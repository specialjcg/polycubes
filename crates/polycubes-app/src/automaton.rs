//! Polycubes that grow generation after generation, for the frontend.

use polycubes_automaton::{Limits, Neighbourhood, Rule, evolve, grow_by_budding, grow_colony};
use polycubes_core::Coordinates;
use serde::{Deserialize, Serialize};

/// Largest seed: the replicator shows up to 216 copies of it at once, and
/// the 3D view slows down beyond a thousand cells or so.
pub const MAX_SEED_CELL_COUNT: usize = 6;

/// The replicator shows its copies at generations 1, 2, 4 and 8; the next
/// ones, at 16, would stand too far apart to be seen together.
const REPLICATOR_LIMITS: Limits = Limits {
    generation_count: 8,
    population: 3000,
};

const COLONY_GENERATION_COUNT: usize = 24;

/// Cells drawn at most: beyond, the 3D view slows down.
const COLONY_POPULATION_LIMIT: usize = 1000;

/// Share of the cells with room beside them that divide at each generation.
pub const COLONY_DIVISION_CHANCE: f64 = 0.4;

const BUDDING_GENERATION_COUNT: usize = 16;

/// Share of the polycubes that try to bud at each generation.
pub const BUDDING_CHANCE: f64 = 0.5;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AutomatonError {
    #[error("cell count must be between 1 and {MAX_SEED_CELL_COUNT}, got {0}")]
    CellCountOutOfRange(usize),
}

/// How the seed is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SeedKind {
    Uniform,
    Growth,
    MostLooped,
}

/// How the seed grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GrowthMode {
    /// Fredkin's parity rule: a cell lives when an odd number of its six
    /// face neighbours do. Any seed comes back as six copies of itself at
    /// each power of two.
    Replicator,
    /// Cells divide at random into the free cells beside them.
    Colony,
    /// Whole polycubes bud turned copies of themselves against them.
    Budding,
}

/// The generations of one seed, the seed first, and the box holding them
/// all, so that the view may be framed once.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomatonRun {
    pub generations: Vec<Vec<Coordinates>>,
    pub lowest_corner: Coordinates,
    pub highest_corner: Coordinates,
}

impl AutomatonRun {
    fn of(generations: Vec<Vec<Coordinates>>) -> Self {
        let coordinates_along =
            |axis: usize| generations.iter().flatten().map(move |cell| cell[axis]);
        Self {
            lowest_corner: [0, 1, 2].map(|axis| coordinates_along(axis).min().unwrap_or(0)),
            highest_corner: [0, 1, 2].map(|axis| coordinates_along(axis).max().unwrap_or(0)),
            generations,
        }
    }
}

/// Checks that a seed of `cell_count` cells may be asked for.
///
/// # Errors
///
/// Returns [`AutomatonError::CellCountOutOfRange`] if `cell_count` is zero
/// or exceeds [`MAX_SEED_CELL_COUNT`].
pub fn check_seed_cell_count(cell_count: usize) -> Result<(), AutomatonError> {
    if cell_count == 0 || cell_count > MAX_SEED_CELL_COUNT {
        return Err(AutomatonError::CellCountOutOfRange(cell_count));
    }
    Ok(())
}

/// Runs the parity rule from `seed`.
#[must_use]
pub fn replicate(seed: &[Coordinates]) -> AutomatonRun {
    let parity_rule = Rule::new(Neighbourhood::Faces, &[1, 3, 5], &[1, 3, 5])
        .expect("odd counts fit six neighbours");
    AutomatonRun::of(evolve(seed, &parity_rule, REPLICATOR_LIMITS).generations)
}

/// Grows a colony from `seed`; `divides` and `choose_neighbour` stand for
/// the random choices of each cell.
#[must_use]
pub fn grow(
    seed: &[Coordinates],
    divides: impl FnMut() -> bool,
    choose_neighbour: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    AutomatonRun::of(grow_colony(
        seed,
        COLONY_GENERATION_COUNT,
        COLONY_POPULATION_LIMIT,
        divides,
        choose_neighbour,
    ))
}

/// Grows a colony of copies of `seed`; `buds` and `choose_placement` stand
/// for the random choices of each copy. A generation lists its copies one
/// after the other, each as many cells as the seed.
#[must_use]
pub fn bud(
    seed: &[Coordinates],
    buds: impl FnMut() -> bool,
    choose_placement: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    AutomatonRun::of(grow_by_budding(
        seed,
        BUDDING_GENERATION_COUNT,
        COLONY_POPULATION_LIMIT,
        buds,
        choose_placement,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [Coordinates; 4] = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];

    #[test]
    fn the_replicator_ends_on_six_copies_of_the_seed() {
        let run = replicate(&SEED);

        assert_eq!(run.generations.len(), 9);
        assert_eq!(run.generations[0].len(), SEED.len());
        assert_eq!(run.generations[8].len(), 6 * SEED.len());
    }

    #[test]
    fn the_box_holds_the_farthest_copies() {
        let run = replicate(&SEED);

        // Copies stand eight cells away along each axis, both ways.
        assert_eq!(run.lowest_corner, [-8, -8, -8]);
        assert_eq!(run.highest_corner, [9, 9, 9]);
    }

    #[test]
    fn a_colony_grows_up_to_the_population_limit() {
        let run = grow(&SEED, || true, |_| 0);

        assert_eq!(run.generations.len(), COLONY_GENERATION_COUNT + 1);
        let populations: Vec<usize> = run.generations.iter().map(Vec::len).collect();
        assert!(populations.is_sorted(), "{populations:?}");
        assert_eq!(populations.last(), Some(&COLONY_POPULATION_LIMIT));
    }

    #[test]
    fn budding_fills_the_population_limit_with_whole_copies() {
        let run = bud(&SEED, || true, |_| 0);

        assert_eq!(run.generations.len(), BUDDING_GENERATION_COUNT + 1);
        assert_eq!(
            run.generations.last().map(Vec::len),
            Some(COLONY_POPULATION_LIMIT / SEED.len() * SEED.len())
        );
    }

    #[test]
    fn refuses_seeds_outside_the_range() {
        for cell_count in [0, MAX_SEED_CELL_COUNT + 1] {
            assert_eq!(
                check_seed_cell_count(cell_count),
                Err(AutomatonError::CellCountOutOfRange(cell_count))
            );
        }
    }
}
