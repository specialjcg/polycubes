//! Polycubes that grow generation after generation, for the frontend.

use polycubes_automaton::{
    Limits, Neighbourhood, Rule, evolve, grow_by_budding, grow_by_mutating_buds, grow_colony,
};
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

/// Over the 12 edge neighbours the copies show at generations 1, 2 and 4;
/// generation 7 would hold 1728 of them.
const EDGE_REPLICATOR_LIMITS: Limits = Limits {
    generation_count: 4,
    population: 3000,
};

/// Cells a soup aims at: a lone small polycube dies at once under the life
/// rule, some twenty cells thrown together live on.
const SOUP_CELL_COUNT: usize = 24;

/// Side of the cube the polycubes of a soup are shifted within: close
/// enough for them to meet.
const SOUP_SIDE: usize = 3;

const LIFE_LIMITS: Limits = Limits {
    generation_count: 32,
    population: 2000,
};

const COLONY_GENERATION_COUNT: usize = 24;

/// Cells drawn at most: beyond, the 3D view slows down.
const COLONY_POPULATION_LIMIT: usize = 1000;

/// Share of the cells with room beside them that divide at each generation.
pub const COLONY_DIVISION_CHANCE: f64 = 0.4;

const BUDDING_GENERATION_COUNT: usize = 16;

/// Mutating polycubes drawn at most: beyond, the branches merge into a
/// lump where shapes can no longer be told apart.
const MUTATION_POLYCUBE_LIMIT: usize = 60;

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
    /// The same parity rule over the twelve edge neighbours: twelve copies.
    EdgeReplicator,
    /// Cells divide at random into the free cells beside them.
    Colony,
    /// Whole polycubes bud turned copies of themselves against them.
    Budding,
    /// A rule of the family of Conway's game of life, over the 26
    /// neighbours, started from several polycubes thrown together.
    Life,
    /// Like budding, but each bud is its parent with one cell moved.
    Mutation,
}

/// The generations of one seed, the seed first, and the box holding them
/// all, so that the view may be framed once. `parent_of_copy` is filled
/// when the generations list polycubes that bore one another: the birth
/// rank of the parent of each, the seed being its own.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomatonRun {
    pub generations: Vec<Vec<Coordinates>>,
    pub lowest_corner: Coordinates,
    pub highest_corner: Coordinates,
    pub parent_of_copy: Vec<usize>,
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

/// Runs the parity rule over the six face neighbours from `seed`.
#[must_use]
pub fn replicate(seed: &[Coordinates]) -> AutomatonRun {
    run_parity_rule(seed, Neighbourhood::Faces, REPLICATOR_LIMITS)
}

/// Runs the parity rule over the twelve edge neighbours from `seed`.
#[must_use]
pub fn replicate_over_edges(seed: &[Coordinates]) -> AutomatonRun {
    run_parity_rule(seed, Neighbourhood::Edges, EDGE_REPLICATOR_LIMITS)
}

fn run_parity_rule(
    seed: &[Coordinates],
    neighbourhood: Neighbourhood,
    limits: Limits,
) -> AutomatonRun {
    let odd_counts: Vec<usize> = (1..=neighbourhood.neighbour_count()).step_by(2).collect();
    let parity_rule = Rule::new(neighbourhood, &odd_counts, &odd_counts)
        .expect("odd counts up to the number of neighbours fit it");
    AutomatonRun::of(evolve(seed, &parity_rule, limits).generations)
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

/// Polycubes of `cell_count` cells a soup takes to hold about
/// [`SOUP_CELL_COUNT`] cells.
#[must_use]
pub fn soup_polycube_count(cell_count: usize) -> usize {
    SOUP_CELL_COUNT.div_ceil(cell_count.max(1))
}

/// Throws `polycubes` together and runs a rule of the family of Conway's
/// game of life: a cell with 3 to 6 living cells among its 26 neighbours
/// lives on, a free cell with exactly 5 is born. `choose_shift` gives how
/// far each polycube is moved along each axis, among [`SOUP_SIDE`] steps,
/// and must answer an index below that number.
///
/// Measured on soups of four to eight hexacubes: this rule kept all of
/// them alive and growing slowly, where the two rules known for their
/// gliders (survival 4-5 birth 5, survival 5-7 birth 6) died out.
#[must_use]
pub fn live(
    polycubes: &[Vec<Coordinates>],
    mut choose_shift: impl FnMut(usize) -> usize,
) -> AutomatonRun {
    let mut soup = Vec::new();
    for polycube in polycubes {
        let shift = [0, 1, 2].map(|_| step_count(choose_shift(SOUP_SIDE)));
        soup.extend(
            polycube
                .iter()
                .map(|cell| [0, 1, 2].map(|axis| cell[axis] + shift[axis])),
        );
    }
    soup.sort_unstable();
    soup.dedup();
    let life_rule = Rule::new(Neighbourhood::FacesEdgesCorners, &[3, 4, 5, 6], &[5])
        .expect("counts below 26 fit the wide neighbourhood");
    AutomatonRun::of(evolve(&soup, &life_rule, LIFE_LIMITS).generations)
}

/// A shift below [`SOUP_SIDE`] always fits a coordinate.
fn step_count(shift: usize) -> i32 {
    i32::try_from(shift).unwrap_or(0)
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
    fn the_edge_replicator_ends_on_twelve_copies_of_the_seed() {
        let run = replicate_over_edges(&SEED);

        assert_eq!(run.generations.len(), 5);
        assert_eq!(run.generations[4].len(), 12 * SEED.len());
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
    fn mutant_buds_come_with_their_parents() {
        let run = bud_mutants(&SEED, || true, |_| 0);

        let last_copy_count = run.generations.last().map_or(0, Vec::len) / SEED.len();
        assert_eq!(last_copy_count, MUTATION_POLYCUBE_LIMIT);
        assert_eq!(run.parent_of_copy.len(), last_copy_count);
    }

    #[test]
    fn life_starts_from_every_polycube_shifted_into_one_soup() {
        let mut shifts = [0, 0, 0, 2, 1, 0].into_iter();
        let run = live(&[SEED.to_vec(), SEED.to_vec()], |_| {
            shifts.next().expect("three shifts per polycube")
        });

        let mut soup: Vec<Coordinates> = SEED
            .iter()
            .flat_map(|&[x, y, z]| [[x, y, z], [x + 2, y + 1, z]])
            .collect();
        soup.sort_unstable();
        soup.dedup();
        assert_eq!(run.generations[0], soup);
    }

    #[test]
    fn a_soup_holds_about_as_many_cells_whatever_the_polycubes() {
        assert_eq!(soup_polycube_count(1), 24);
        assert_eq!(soup_polycube_count(5), 5);
        assert_eq!(soup_polycube_count(6), 4);
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
