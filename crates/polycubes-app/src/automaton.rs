//! Cellular automata seeded with polycubes, for the frontend.

use polycubes_automaton::{Fate, Limits, Neighbourhood, Rule, census, evolve};
use polycubes_core::Coordinates;
use serde::{Deserialize, Serialize};

/// Largest seed: the most looped polycube is found by visiting every fixed
/// polycube, which takes seconds beyond this size.
pub const MAX_SEED_CELL_COUNT: usize = 9;

/// Largest size whose polycubes are all run: beyond it the census takes
/// over a second.
pub const MAX_CENSUS_CELL_COUNT: usize = 6;

/// A seed that has neither died nor repeated by then is left unsettled.
const LIMITS: Limits = Limits {
    generation_count: 60,
    population: 300,
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AutomatonError {
    #[error("cell count must be between 1 and {largest}, got {cell_count}")]
    CellCountOutOfRange { cell_count: usize, largest: usize },
}

/// How the seed is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SeedKind {
    Uniform,
    Growth,
    MostLooped,
}

/// The rules offered. Those on 26 neighbours are the two "lives" Carter
/// Bays singled out in three dimensions; the one on 6 neighbours gave the
/// most cycles among the rules tried on seeds of five cubes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RulePreset {
    /// 26 neighbours: alive with 4 or 5, born with 5.
    Bays4555,
    /// 26 neighbours: alive with 5 to 7, born with 6.
    Bays5766,
    /// 6 neighbours: alive with 2 or 3, born with 2 or 3.
    Faces2323,
}

impl RulePreset {
    fn rule(self) -> Rule {
        let (neighbourhood, survival_counts, birth_counts): (_, &[usize], &[usize]) = match self {
            Self::Bays4555 => (Neighbourhood::FacesEdgesCorners, &[4, 5], &[5]),
            Self::Bays5766 => (Neighbourhood::FacesEdgesCorners, &[5, 6, 7], &[6]),
            Self::Faces2323 => (Neighbourhood::Faces, &[2, 3], &[2, 3]),
        };
        Rule::new(neighbourhood, survival_counts, birth_counts)
            .expect("the counts of a preset fit its neighbourhood")
    }
}

/// The generations of one seed, the seed first, and what becomes of it.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomatonRun {
    pub generations: Vec<Vec<Coordinates>>,
    pub fate: FateReport,
}

/// What becomes of a seed; `generation` is the first one of the final
/// behaviour.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FateReport {
    Dies {
        generation: usize,
    },
    Still {
        generation: usize,
    },
    Cycles {
        generation: usize,
        period: usize,
    },
    Travels {
        generation: usize,
        period: usize,
        displacement: Coordinates,
    },
    Unsettled,
}

impl From<Fate> for FateReport {
    fn from(fate: Fate) -> Self {
        match fate {
            Fate::Dies { generation } => Self::Dies { generation },
            Fate::Still { generation } => Self::Still { generation },
            Fate::Cycles { generation, period } => Self::Cycles { generation, period },
            Fate::Travels {
                generation,
                period,
                displacement,
            } => Self::Travels {
                generation,
                period,
                displacement,
            },
            Fate::Unsettled => Self::Unsettled,
        }
    }
}

/// Fixed polycubes of a size per fate.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CensusReport {
    pub dying: u64,
    pub still: u64,
    pub cycling: u64,
    pub travelling: u64,
    pub unsettled: u64,
}

/// Checks that a seed of `cell_count` cells may be asked for.
///
/// # Errors
///
/// Returns [`AutomatonError::CellCountOutOfRange`] if `cell_count` is zero
/// or exceeds [`MAX_SEED_CELL_COUNT`].
pub fn check_seed_cell_count(cell_count: usize) -> Result<(), AutomatonError> {
    check_cell_count(cell_count, MAX_SEED_CELL_COUNT)
}

fn check_cell_count(cell_count: usize, largest: usize) -> Result<(), AutomatonError> {
    if cell_count == 0 || cell_count > largest {
        return Err(AutomatonError::CellCountOutOfRange {
            cell_count,
            largest,
        });
    }
    Ok(())
}

/// Runs the automaton from `seed`.
#[must_use]
pub fn run_automaton(seed: &[Coordinates], rule: RulePreset) -> AutomatonRun {
    let evolution = evolve(seed, &rule.rule(), LIMITS);
    AutomatonRun {
        generations: evolution.generations,
        fate: evolution.fate.into(),
    }
}

/// Runs the automaton from every fixed polycube of `cell_count` cells.
///
/// # Errors
///
/// Returns [`AutomatonError::CellCountOutOfRange`] if `cell_count` is zero
/// or exceeds [`MAX_CENSUS_CELL_COUNT`].
pub fn take_census(cell_count: usize, rule: RulePreset) -> Result<CensusReport, AutomatonError> {
    check_cell_count(cell_count, MAX_CENSUS_CELL_COUNT)?;
    let found = census(cell_count, &rule.rule(), LIMITS);
    Ok(CensusReport {
        dying: found.dying,
        still: found.still,
        cycling: found.cycling,
        travelling: found.travelling,
        unsettled: found.unsettled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube() -> Vec<Coordinates> {
        (0..8)
            .map(|corner| [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1])
            .collect()
    }

    #[test]
    fn the_cube_stands_still_when_five_to_seven_neighbours_keep_alive() {
        // Each cube of the 2 × 2 × 2 cube touches the seven others.
        let run = run_automaton(&unit_cube(), RulePreset::Bays5766);

        assert_eq!(run.fate, FateReport::Still { generation: 0 });
        assert_eq!(run.generations.len(), 2);
    }

    #[test]
    fn the_cube_dies_when_seven_neighbours_are_too_many() {
        let run = run_automaton(&unit_cube(), RulePreset::Bays4555);

        assert_eq!(run.fate, FateReport::Dies { generation: 1 });
    }

    #[test]
    fn the_census_holds_every_fixed_polycube() {
        let found = take_census(5, RulePreset::Bays4555).expect("five cells can be counted");

        let total = found.dying + found.still + found.cycling + found.travelling + found.unsettled;
        assert_eq!(total, 534);
    }

    #[test]
    fn refuses_sizes_outside_the_ranges() {
        assert_eq!(
            take_census(MAX_CENSUS_CELL_COUNT + 1, RulePreset::Bays4555),
            Err(AutomatonError::CellCountOutOfRange {
                cell_count: MAX_CENSUS_CELL_COUNT + 1,
                largest: MAX_CENSUS_CELL_COUNT
            })
        );
        assert_eq!(
            check_seed_cell_count(0),
            Err(AutomatonError::CellCountOutOfRange {
                cell_count: 0,
                largest: MAX_SEED_CELL_COUNT
            })
        );
    }
}
