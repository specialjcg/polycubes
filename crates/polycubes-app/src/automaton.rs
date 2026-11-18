//! Polycubes that grow generation after generation, for the frontend.

use polycubes_automaton::{
    ConwayRule, DOMINO, DOMINO_GLIDER, LivingPolycube, PolycubeLifeRule, PolycubeWorld,
    SCREW_GLIDER, SQUARE_GLIDER, Soup, THICK_SQUARE_GLIDER, Torus, aggregate, canonical_shape,
    grow_by_budding, grow_by_mutating_buds, live_as_polycubes, play_conway, soup,
};
use polycubes_core::Coordinates;
use serde::{Deserialize, Serialize};

/// Largest seed: the 3D view slows down beyond a thousand cells or so, and
/// larger polycubes would leave too few of them to watch.
pub const MAX_SEED_CELL_COUNT: usize = 6;

/// Cells drawn at most: beyond, the 3D view slows down.
const POPULATION_LIMIT: usize = 1000;

/// Largest aggregate: it takes some 45 seconds to grow, and the 3D view
/// draws it as one mesh of that many cubes.
pub const MAX_AGGREGATE_CELL_COUNT: usize = 10_000;

/// Generations the arrivals of an aggregate are spread over.
const AGGREGATE_GENERATION_COUNT: usize = 24;

const BUDDING_GENERATION_COUNT: usize = 16;

/// Mutating polycubes drawn at most: beyond, the branches merge into a
/// lump where shapes can no longer be told apart.
const MUTATION_POLYCUBE_LIMIT: usize = 60;

/// Share of the polycubes that try to bud at each generation.
pub const BUDDING_CHANCE: f64 = 0.5;

/// Measured on seeds of five cells, against seven other settings: this one
/// had the most deaths, some 200 over 30 generations. One child in five
/// mutates: often enough for the 29 shapes of five cells to show up, seldom
/// enough for a shape that fares well to spread.
const LIFE_RULE: PolycubeLifeRule = PolycubeLifeRule {
    most_neighbours_to_survive: 2,
    fewest_neighbours_to_be_born: 1,
    most_neighbours_to_be_born: 2,
    mutation_one_in: 5,
};

const LIFE_GENERATION_COUNT: usize = 200;

/// The world of the game reaches at most that far from the seed along each
/// axis: a cube of 21 cells a side, where 200 generations take some 45
/// seconds and 700 polycubes live at once.
pub const MAX_LIFE_WORLD_REACH: i32 = 10;

/// Found by sweeping rules over soups of dominoes, with births favouring no
/// direction: the one where a glider of four dominoes showed up on its own.
const CONWAY_RULE: ConwayRule = DOMINO_GLIDER.rule;

/// Large enough for a glider to be watched on its way, which takes it 62
/// generations to cross.
const CONWAY_TORUS: Torus = Torus { side: 31 };

const CONWAY_GENERATION_COUNT: usize = 200;

/// Poured in the middle of the torus, as densely as the soups where
/// gliders were found.
const CONWAY_SOUP: Soup = Soup {
    side: 16,
    attempt_count: 320,
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AutomatonError {
    #[error("cell count must be between 1 and {MAX_SEED_CELL_COUNT}, got {0}")]
    CellCountOutOfRange(usize),
    #[error("an aggregate holds between 1 and {MAX_AGGREGATE_CELL_COUNT} cells, got {0}")]
    AggregateCellCountOutOfRange(usize),
    #[error("the world of life reaches between 1 and {MAX_LIFE_WORLD_REACH} cells, got {0}")]
    LifeWorldReachOutOfRange(i32),
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
    /// A game of life after Conway's, with dominoes, where gliders arise.
    Gliders,
}

/// What a Conway-like game of dominoes starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConwayStart {
    /// Dominoes poured at random, from which gliders may or may not arise.
    Soup,
    /// Four dominoes gliding in one plane.
    DominoGlider,
    /// Six squares flying along a diagonal of the cube, under their own rule.
    SquareGlider,
    /// Six squares under the same rule, thicker and quicker to come back.
    ThickSquareGlider,
    /// Three screws running along an axis, under their own rule.
    ScrewGlider,
}

/// The generations of one seed, the seed first, and the box holding them
/// all, so that the view may be framed once. A generation lists polycubes
/// one after the other, each as many cells as the seed.
///
/// `parent_of_copy` is filled when the polycubes bore one another: the
/// birth rank of the parent of each, the seed being its own.
/// `identities_by_generation` is filled when polycubes die: for each
/// generation, what tells each of its polycubes from all others.
/// `shapes_by_generation` comes with it: for each polycube, a number shared
/// by all those of its shape, however turned, over the whole run.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomatonRun {
    pub generations: Vec<Vec<Coordinates>>,
    pub lowest_corner: Coordinates,
    pub highest_corner: Coordinates,
    pub parent_of_copy: Vec<usize>,
    pub identities_by_generation: Vec<Vec<usize>>,
    pub shapes_by_generation: Vec<Vec<usize>>,
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
            shapes_by_generation: Vec::new(),
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

/// Checks that an aggregate of `cell_count` cells may be asked for.
///
/// # Errors
///
/// Returns [`AutomatonError::AggregateCellCountOutOfRange`] if `cell_count`
/// is zero or exceeds [`MAX_AGGREGATE_CELL_COUNT`].
pub fn check_aggregate_cell_count(cell_count: usize) -> Result<(), AutomatonError> {
    if cell_count == 0 || cell_count > MAX_AGGREGATE_CELL_COUNT {
        return Err(AutomatonError::AggregateCellCountOutOfRange(cell_count));
    }
    Ok(())
}

/// Checks that a world of life reaching `reach` cells may be asked for.
///
/// # Errors
///
/// Returns [`AutomatonError::LifeWorldReachOutOfRange`] if `reach` is below
/// one or exceeds [`MAX_LIFE_WORLD_REACH`].
pub fn check_life_world_reach(reach: i32) -> Result<(), AutomatonError> {
    if !(1..=MAX_LIFE_WORLD_REACH).contains(&reach) {
        return Err(AutomatonError::LifeWorldReachOutOfRange(reach));
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

/// Grows an aggregate of about `cell_count` cells from `seed`: copies of
/// it, turned at random, wander in and stick where they first touch.
/// `choose` stands for every random choice. A generation lists the polycubes stuck so far, in the order
/// they came.
#[must_use]
pub fn aggregate_wanderers(
    seed: &[Coordinates],
    cell_count: usize,
    choose: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let polycube_count = cell_count / seed.len().max(1);
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
/// living, with the shape of its parent or, one time in five, that shape
/// with one cell moved. The world is a cube reaching `world_reach` cells
/// from the seed along each axis: births go on wherever room is left in it,
/// so the game never freezes. `buds` and `choose` stand for the random
/// choices of each.
#[must_use]
pub fn live(
    seed: &[Coordinates],
    world_reach: i32,
    buds: impl FnMut() -> bool,
    choose: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let world = PolycubeWorld {
        generation_count: LIFE_GENERATION_COUNT,
        reach: world_reach,
    };
    live_in(world, seed, buds, choose)
}

fn live_in(
    world: PolycubeWorld,
    seed: &[Coordinates],
    buds: impl FnMut() -> bool,
    choose: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let generations = live_as_polycubes(seed, LIFE_RULE, world, buds, choose);
    AutomatonRun::told_apart(generations)
}

/// Plays a game of life after Conway's on a torus, framed on the whole
/// torus: nothing buds, a polycube is born where it fits snugly against the
/// living and dies when too few or too many lean on it. A soup of dominoes
/// plays under the rule of the domino glider, each glider under its own.
/// `choose` stands for the random choices of a soup.
#[must_use]
pub fn play_gliders(start: ConwayStart, choose: impl FnMut(usize) -> usize) -> AutomatonRun {
    play_gliders_for(CONWAY_GENERATION_COUNT, start, choose)
}

fn play_gliders_for(
    generation_count: usize,
    start: ConwayStart,
    choose: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let middle = (CONWAY_TORUS.side - CONWAY_SOUP.side) / 2;
    let (start_cells, rule) = match start {
        ConwayStart::Soup => (soup(&DOMINO, CONWAY_SOUP, choose), CONWAY_RULE),
        ConwayStart::DominoGlider => (DOMINO_GLIDER.start(), DOMINO_GLIDER.rule),
        ConwayStart::SquareGlider => (SQUARE_GLIDER.start(), SQUARE_GLIDER.rule),
        ConwayStart::ThickSquareGlider => (THICK_SQUARE_GLIDER.start(), THICK_SQUARE_GLIDER.rule),
        ConwayStart::ScrewGlider => (SCREW_GLIDER.start(), SCREW_GLIDER.rule),
    };
    let centred: Vec<Vec<Coordinates>> = start_cells
        .iter()
        .map(|cells| {
            cells
                .iter()
                .map(|cell| cell.map(|coordinate| coordinate + middle))
                .collect()
        })
        .collect();
    let generations = play_conway(&centred, rule, CONWAY_TORUS, generation_count);
    AutomatonRun {
        lowest_corner: [0, 0, 0],
        highest_corner: [CONWAY_TORUS.side - 1; 3],
        ..AutomatonRun::told_apart(generations)
    }
}

impl AutomatonRun {
    /// Generations of polycubes that come and go, each told apart by its
    /// identity and its shape.
    fn told_apart(generations: Vec<Vec<LivingPolycube>>) -> Self {
        let identities_by_generation = generations
            .iter()
            .map(|living| living.iter().map(|polycube| polycube.identity).collect())
            .collect();
        let mut shape_ranks = ShapeRanks::default();
        let shapes_by_generation = generations
            .iter()
            .map(|living| {
                living
                    .iter()
                    .map(|polycube| shape_ranks.rank_of(&polycube.cells))
                    .collect()
            })
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
            shapes_by_generation,
            ..AutomatonRun::of(cells_by_generation)
        }
    }
}

/// Numbers the shapes in the order they are first met.
#[derive(Default)]
struct ShapeRanks {
    rank_by_shape: std::collections::HashMap<Vec<Coordinates>, usize>,
}

impl ShapeRanks {
    fn rank_of(&mut self, cells: &[Coordinates]) -> usize {
        let next_rank = self.rank_by_shape.len();
        *self
            .rank_by_shape
            .entry(canonical_shape(cells))
            .or_insert(next_rank)
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
        let run = aggregate_wanderers(&SEED, 200, varied_choice());

        assert_eq!(run.generations.len(), AGGREGATE_GENERATION_COUNT + 1);
        assert_eq!(run.generations[0], SEED);
        let populations: Vec<usize> = run.generations.iter().map(Vec::len).collect();
        assert!(populations.is_sorted(), "{populations:?}");
        assert!(
            populations
                .iter()
                .all(|population| population % SEED.len() == 0)
        );
        assert_eq!(populations.last(), Some(&200));
    }

    #[test]
    fn refuses_life_worlds_outside_the_range() {
        for reach in [0, MAX_LIFE_WORLD_REACH + 1] {
            assert_eq!(
                check_life_world_reach(reach),
                Err(AutomatonError::LifeWorldReachOutOfRange(reach))
            );
        }
        assert_eq!(check_life_world_reach(MAX_LIFE_WORLD_REACH), Ok(()));
    }

    #[test]
    fn refuses_aggregates_outside_the_range() {
        for cell_count in [0, MAX_AGGREGATE_CELL_COUNT + 1] {
            assert_eq!(
                check_aggregate_cell_count(cell_count),
                Err(AutomatonError::AggregateCellCountOutOfRange(cell_count))
            );
        }
        assert_eq!(check_aggregate_cell_count(MAX_AGGREGATE_CELL_COUNT), Ok(()));
    }

    #[test]
    fn life_tells_every_polycube_of_every_generation_apart() {
        // A small world played briefly: the real one takes minutes unoptimised.
        let world = PolycubeWorld {
            generation_count: 14,
            reach: 3,
        };
        let run = live_in(world, &SEED, || true, varied_choice());

        assert_eq!(run.generations.len(), world.generation_count + 1);
        assert_eq!(run.identities_by_generation.len(), run.generations.len());
        for (cells, identities) in run.generations.iter().zip(&run.identities_by_generation) {
            assert_eq!(cells.len(), identities.len() * SEED.len());
            assert!(identities.is_sorted_by(|earlier, later| earlier < later));
        }
        assert_eq!(run.shapes_by_generation[0], [0]);
        let shape_count_by_generation: Vec<usize> =
            run.shapes_by_generation.iter().map(Vec::len).collect();
        let polycube_count_by_generation: Vec<usize> =
            run.identities_by_generation.iter().map(Vec::len).collect();
        assert_eq!(shape_count_by_generation, polycube_count_by_generation);
        let last_shapes: std::collections::HashSet<&usize> = run
            .shapes_by_generation
            .last()
            .into_iter()
            .flatten()
            .collect();
        assert!(last_shapes.len() > 1, "no shape but the seed's");
    }

    #[test]
    fn a_glider_glides_round_the_torus_without_end() {
        // Played briefly: the real game takes a minute unoptimised.
        let run = play_gliders_for(30, ConwayStart::DominoGlider, varied_choice());

        assert_eq!(run.generations.len(), 31);
        let polycube_counts: Vec<usize> =
            run.identities_by_generation.iter().map(Vec::len).collect();
        assert!(
            polycube_counts.iter().all(|count| (4..=6).contains(count)),
            "{polycube_counts:?}"
        );
        assert_eq!(run.lowest_corner, [0, 0, 0]);
        assert_eq!(run.highest_corner, [CONWAY_TORUS.side - 1; 3]);
    }

    #[test]
    fn the_square_and_screw_gliders_keep_on_living_as_tetracubes() {
        for start in [
            ConwayStart::SquareGlider,
            ConwayStart::ThickSquareGlider,
            ConwayStart::ScrewGlider,
        ] {
            let run = play_gliders_for(30, start, varied_choice());

            for (cells, identities) in run.generations.iter().zip(&run.identities_by_generation) {
                assert!(!identities.is_empty(), "{start:?} died out");
                assert_eq!(cells.len(), identities.len() * 4, "{start:?}");
            }
        }
    }

    #[test]
    fn a_soup_of_dominoes_is_poured_in_the_middle_of_the_torus() {
        let run = play_gliders_for(10, ConwayStart::Soup, varied_choice());

        let first = &run.generations[0];
        assert!(first.len() > 40, "only {} cells", first.len());
        let (low, high) = (CONWAY_TORUS.side / 4, CONWAY_TORUS.side * 3 / 4);
        assert!(
            first
                .iter()
                .flatten()
                .all(|coordinate| (low..=high).contains(coordinate))
        );
        for (cells, identities) in run.generations.iter().zip(&run.identities_by_generation) {
            assert_eq!(cells.len(), identities.len() * DOMINO.len());
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
