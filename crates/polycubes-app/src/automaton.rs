//! Polycubes that grow generation after generation, for the frontend.

use polycubes_automaton::{
    PolycubeLifeRule, PolycubeWorld, aggregate, grow_by_budding, grow_by_mutating_buds,
    live_as_polycubes,
};
use polycubes_core::Coordinates;
use serde::{Deserialize, Serialize};

/// Largest seed: the 3D view slows down beyond a thousand cells or so, and
/// larger polycubes would leave too few of them to watch.
pub const MAX_SEED_CELL_COUNT: usize = 6;

/// Cells drawn at most: beyond, the 3D view slows down.
const POPULATION_LIMIT: usize = 1000;

/// Cells an aggregate aims at, whatever the size of its polycubes.
const AGGREGATE_CELL_COUNT: usize = 600;

/// Generations the arrivals of an aggregate are spread over.
const AGGREGATE_GENERATION_COUNT: usize = 24;

const BUDDING_GENERATION_COUNT: usize = 16;

/// Mutating polycubes drawn at most: beyond, the branches merge into a
/// lump where shapes can no longer be told apart.
const MUTATION_POLYCUBE_LIMIT: usize = 60;

/// Share of the polycubes that try to bud at each generation.
pub const BUDDING_CHANCE: f64 = 0.5;

/// Measured on seeds of five cells, against seven other settings: this one
/// had the most deaths, some 200 over 30 generations.
const LIFE_RULE: PolycubeLifeRule = PolycubeLifeRule {
    most_neighbours_to_survive: 2,
    fewest_neighbours_to_be_born: 1,
    most_neighbours_to_be_born: 2,
};

/// The world is a cube of 11 cells a side around the seed: births go on
/// wherever room is left in it, so the game never freezes.
const LIFE_WORLD: PolycubeWorld = PolycubeWorld {
    generation_count: 30,
    reach: 5,
};

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
    /// Polycubes wander in from afar and stick where they first touch.
    Aggregation,
    /// Like budding, but each bud is its parent with one cell moved.
    Mutation,
    /// Whole polycubes bud turned copies of themselves against them.
    Budding,
    /// A game of life whose living unit is a whole polycube.
    Life,
}

/// The generations of one seed, the seed first, and the box holding them
/// all, so that the view may be framed once. A generation lists polycubes
/// one after the other, each as many cells as the seed.
///
/// `parent_of_copy` is filled when the polycubes bore one another: the
/// birth rank of the parent of each, the seed being its own.
/// `identities_by_generation` is filled when polycubes die: for each
/// generation, what tells each of its polycubes from all others.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomatonRun {
    pub generations: Vec<Vec<Coordinates>>,
    pub lowest_corner: Coordinates,
    pub highest_corner: Coordinates,
    pub parent_of_copy: Vec<usize>,
    pub identities_by_generation: Vec<Vec<usize>>,
}

impl AutomatonRun {
    fn of(generations: Vec<Vec<Coordinates>>) -> Self {
        let coordinates_along =
            |axis: usize| generations.iter().flatten().map(move |cell| cell[axis]);
        Self {
            lowest_corner: [0, 1, 2].map(|axis| coordinates_along(axis).min().unwrap_or(0)),
            highest_corner: [0, 1, 2].map(|axis| coordinates_along(axis).max().unwrap_or(0)),
            generations,
            parent_of_copy: Vec::new(),
            identities_by_generation: Vec::new(),
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

/// Grows a colony of copies of `seed`; `buds` and `choose_placement` stand
/// for the random choices of each copy.
#[must_use]
pub fn bud(
    seed: &[Coordinates],
    buds: impl FnMut() -> bool,
    choose_placement: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    AutomatonRun::of(grow_by_budding(
        seed,
        BUDDING_GENERATION_COUNT,
        POPULATION_LIMIT,
        buds,
        choose_placement,
    ))
}

/// Grows a colony of polycubes where each bud is its parent with one cell
/// moved; `buds` and `choose` stand for the random choices of each.
#[must_use]
pub fn bud_mutants(
    seed: &[Coordinates],
    buds: impl FnMut() -> bool,
    choose: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let lineage = grow_by_mutating_buds(
        seed,
        BUDDING_GENERATION_COUNT,
        MUTATION_POLYCUBE_LIMIT * seed.len(),
        buds,
        choose,
    );
    AutomatonRun {
        parent_of_copy: lineage.parent_of_copy,
        ..AutomatonRun::of(lineage.generations)
    }
}

/// Grows an aggregate from `seed`: copies of it, turned at random, wander
/// in and stick where they first touch. `choose` stands for every random
/// choice. A generation lists the polycubes stuck so far, in the order
/// they came.
#[must_use]
pub fn aggregate_wanderers(
    seed: &[Coordinates],
    choose: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let polycube_count = AGGREGATE_CELL_COUNT / seed.len().max(1);
    let polycubes = aggregate(seed, polycube_count, choose);
    let arrivals_per_generation = polycube_count.div_ceil(AGGREGATE_GENERATION_COUNT).max(1);
    let generations = (0..=AGGREGATE_GENERATION_COUNT)
        .map(|generation| {
            let stuck_count = (1 + generation * arrivals_per_generation).min(polycubes.len());
            polycubes[..stuck_count].concat()
        })
        .collect();
    AutomatonRun::of(generations)
}

/// Plays the game of life of whole polycubes from `seed`: one dies when
/// more than two others lean on it, one is born against one or two of the
/// living. `buds` and `choose` stand for the random choices of each.
#[must_use]
pub fn live(
    seed: &[Coordinates],
    buds: impl FnMut() -> bool,
    choose: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let generations = live_as_polycubes(seed, LIFE_RULE, LIFE_WORLD, buds, choose);
    let identities_by_generation = generations
        .iter()
        .map(|living| living.iter().map(|polycube| polycube.identity).collect())
        .collect();
    let cells_by_generation = generations
        .into_iter()
        .map(|living| {
            living
                .into_iter()
                .flat_map(|polycube| polycube.cells)
                .collect()
        })
        .collect();
    AutomatonRun {
        identities_by_generation,
        ..AutomatonRun::of(cells_by_generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [Coordinates; 4] = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];

    /// Choices that look random but are the same at every run.
    fn varied_choice() -> impl FnMut(usize) -> usize {
        let mut state: u64 = 0x2545_f491_4f6c_dd1d;
        move |choice_count| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            usize::try_from(state >> 33).expect("31 bits fit") % choice_count
        }
    }

    #[test]
    fn the_box_holds_every_generation() {
        let run = bud(&SEED, || true, |_| 0);

        for cell in run.generations.iter().flatten() {
            for (axis, coordinate) in cell.iter().enumerate() {
                assert!((run.lowest_corner[axis]..=run.highest_corner[axis]).contains(coordinate));
            }
        }
    }

    #[test]
    fn budding_fills_the_population_limit_with_whole_copies() {
        let run = bud(&SEED, || true, |_| 0);

        assert_eq!(run.generations.len(), BUDDING_GENERATION_COUNT + 1);
        assert_eq!(
            run.generations.last().map(Vec::len),
            Some(POPULATION_LIMIT / SEED.len() * SEED.len())
        );
    }

    #[test]
    fn mutant_buds_come_with_their_parents() {
        let run = bud_mutants(&SEED, || true, |_| 0);

        let last_copy_count = run.generations.last().map_or(0, Vec::len) / SEED.len();
        assert_eq!(last_copy_count, MUTATION_POLYCUBE_LIMIT);
        assert_eq!(run.parent_of_copy.len(), last_copy_count);
    }

    #[test]
    fn an_aggregate_gains_whole_polycubes_generation_after_generation() {
        let run = aggregate_wanderers(&SEED, varied_choice());

        assert_eq!(run.generations.len(), AGGREGATE_GENERATION_COUNT + 1);
        assert_eq!(run.generations[0], SEED);
        let populations: Vec<usize> = run.generations.iter().map(Vec::len).collect();
        assert!(populations.is_sorted(), "{populations:?}");
        assert!(
            populations
                .iter()
                .all(|population| population % SEED.len() == 0)
        );
        assert_eq!(populations.last(), Some(&AGGREGATE_CELL_COUNT));
    }

    #[test]
    fn life_tells_every_polycube_of_every_generation_apart() {
        let run = live(&SEED, || true, varied_choice());

        assert_eq!(run.generations.len(), LIFE_WORLD.generation_count + 1);
        assert_eq!(run.identities_by_generation.len(), run.generations.len());
        for (cells, identities) in run.generations.iter().zip(&run.identities_by_generation) {
            assert_eq!(cells.len(), identities.len() * SEED.len());
            assert!(identities.is_sorted_by(|earlier, later| earlier < later));
        }
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
