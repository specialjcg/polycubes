//! The gallery of patterns of the Conway-like life of whole polycubes:
//! found once by `examples/pattern_gallery.rs` and embedded in the binary,
//! so that nothing is searched when the page opens.

use std::sync::LazyLock;

use polycubes_automaton::{ConwayRule, Torus, play_conway};
use polycubes_core::Coordinates;
use serde::{Deserialize, Serialize};

use crate::automaton::AutomatonRun;

const CATALOGUE_JSON: &str = include_str!("../../../docs/vie/galerie.json");

/// Wide enough that a glider played a few periods never comes back round.
const PATTERN_TORUS: Torus = Torus { side: 200 };

/// A pattern is played that many periods, within the bounds below: enough
/// to see it come back, not so many that a glider flies out of sight.
const PERIODS_PLAYED: usize = 3;
const FEWEST_GENERATIONS: usize = 24;
const MOST_GENERATIONS: usize = 120;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PatternsError {
    #[error("the embedded catalogue of patterns is unreadable: {0}")]
    UnreadableCatalogue(String),
    #[error("no pattern bears the number {0}")]
    UnknownPattern(usize),
}

/// How a pattern behaves once it comes back as it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PatternNature {
    Still,
    Oscillator,
    Glider,
}

/// The rule a pattern lives under, as the catalogue spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CataloguedRule {
    pub fewest_neighbours_to_survive: usize,
    pub most_neighbours_to_survive: usize,
    pub fewest_neighbours_to_be_born: usize,
    pub most_neighbours_to_be_born: usize,
    pub fewest_faces_shared_to_be_born: usize,
}

impl From<CataloguedRule> for ConwayRule {
    fn from(rule: CataloguedRule) -> Self {
        Self {
            fewest_neighbours_to_survive: rule.fewest_neighbours_to_survive,
            most_neighbours_to_survive: rule.most_neighbours_to_survive,
            fewest_neighbours_to_be_born: rule.fewest_neighbours_to_be_born,
            most_neighbours_to_be_born: rule.most_neighbours_to_be_born,
            fewest_faces_shared_to_be_born: rule.fewest_faces_shared_to_be_born,
        }
    }
}

/// One pattern of the gallery, its lowest corner at the origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pattern {
    pub number: usize,
    pub shape: String,
    pub nature: PatternNature,
    pub period: usize,
    pub shift: Coordinates,
    pub layers: [usize; 3],
    pub symmetry: usize,
    pub rule: CataloguedRule,
    pub polycubes: Vec<Vec<Coordinates>>,
}

#[derive(Deserialize)]
struct Catalogue {
    patterns: Vec<Pattern>,
}

static CATALOGUE: LazyLock<Result<Vec<Pattern>, String>> = LazyLock::new(|| {
    serde_json::from_str::<Catalogue>(CATALOGUE_JSON)
        .map(|catalogue| catalogue.patterns)
        .map_err(|error| error.to_string())
});

/// The patterns of the gallery, in their order.
///
/// # Errors
///
/// Returns [`PatternsError::UnreadableCatalogue`] if the embedded catalogue
/// cannot be read.
pub fn patterns() -> Result<&'static [Pattern], PatternsError> {
    CATALOGUE
        .as_deref()
        .map_err(|reason| PatternsError::UnreadableCatalogue(reason.clone()))
}

/// Plays the pattern bearing `number` under its own rule, a few periods,
/// framed on every cell it reaches.
///
/// # Errors
///
/// Returns [`PatternsError::UnknownPattern`] if no pattern bears `number`,
/// or [`PatternsError::UnreadableCatalogue`] if the catalogue cannot be read.
pub fn play_pattern(number: usize) -> Result<AutomatonRun, PatternsError> {
    let pattern = patterns()?
        .iter()
        .find(|pattern| pattern.number == number)
        .ok_or(PatternsError::UnknownPattern(number))?;
    let middle = PATTERN_TORUS.side / 2;
    let centred: Vec<Vec<Coordinates>> = pattern
        .polycubes
        .iter()
        .map(|cells| {
            cells
                .iter()
                .map(|cell| cell.map(|coordinate| coordinate + middle))
                .collect()
        })
        .collect();
    let generation_count =
        (PERIODS_PLAYED * pattern.period).clamp(FEWEST_GENERATIONS, MOST_GENERATIONS);
    let generations = play_conway(
        &centred,
        pattern.rule.into(),
        PATTERN_TORUS,
        generation_count,
    );
    Ok(AutomatonRun::told_apart(generations))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_of(nature: PatternNature) -> &'static Pattern {
        patterns()
            .ok()
            .and_then(|all| all.iter().find(|pattern| pattern.nature == nature))
            .expect("the catalogue holds a pattern of each nature")
    }

    #[test]
    fn the_catalogue_numbers_its_patterns_from_one_in_order() {
        let numbers: Vec<usize> = patterns()
            .map(|all| all.iter().map(|pattern| pattern.number).collect())
            .unwrap_or_default();

        let expected: Vec<usize> = (1..=numbers.len()).collect();
        assert!(numbers.len() >= 12 && numbers == expected);
    }

    #[test]
    fn the_catalogue_holds_every_nature() {
        let natures = [
            PatternNature::Still,
            PatternNature::Oscillator,
            PatternNature::Glider,
        ];

        assert!(natures.iter().all(|&nature| {
            patterns().is_ok_and(|all| all.iter().any(|pattern| pattern.nature == nature))
        }));
    }

    #[test]
    fn a_still_pattern_plays_unchanged() {
        let pattern = first_of(PatternNature::Still);

        let run = play_pattern(pattern.number).expect("a catalogued pattern plays");

        assert!(
            run.generations.len() == FEWEST_GENERATIONS + 1
                && run
                    .generations
                    .iter()
                    .all(|cells| cells == &run.generations[0])
        );
    }

    #[test]
    fn a_glider_comes_back_moved_after_its_period() {
        let pattern = first_of(PatternNature::Glider);

        let run = play_pattern(pattern.number).expect("a catalogued pattern plays");

        let moved: Vec<Coordinates> = run.generations[0]
            .iter()
            .map(|cell| [0, 1, 2].map(|axis| cell[axis] + pattern.shift[axis]))
            .collect();
        let mut later = run.generations[pattern.period].clone();
        let mut expected = moved;
        later.sort_unstable();
        expected.sort_unstable();
        assert_eq!(later, expected);
    }

    #[test]
    fn an_unknown_number_is_refused() {
        assert_eq!(
            play_pattern(0).err(),
            Some(PatternsError::UnknownPattern(0))
        );
    }
}
