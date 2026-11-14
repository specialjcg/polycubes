//! A game of life whose living unit is a whole polycube: it dies when too
//! many others lean on it, and new ones are born against the living where
//! few do. A polycube stays whole from its birth to its death.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use polycubes_core::Coordinates;

use crate::budding::{moved_onto, orientations};
use crate::colony::face_neighbours;
use crate::evolution::Limits;

/// Neighbours of a polycube are the other polycubes it shares a face with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolycubeLifeRule {
    /// A polycube with more neighbours dies of crowding.
    pub most_neighbours_to_survive: usize,
    /// A polycube is born only where it would have at least that many
    /// neighbours, and at most the next field.
    pub fewest_neighbours_to_be_born: usize,
    pub most_neighbours_to_be_born: usize,
}

/// One polycube of a generation: `identity` is its birth rank over the
/// whole game, the same as long as it lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivingPolycube {
    pub identity: usize,
    pub cells: Vec<Coordinates>,
}

/// Plays `limits.generation_count` generations from `seed` alone and
/// returns them all, the seed first.
///
/// All polycubes change at once, from the generation before. One with more
/// neighbours than the rule allows dies. Each one, dying or not, bears a
/// child when `buds` says so: the seed turned any way, never mirrored,
/// against its parent, over no cell taken before or during the generation,
/// and with a number of neighbours among the generation before that the
/// rule allows. `choose` picks the child among the number of such places,
/// and must answer an index below that number. None is born once the
/// generation holds `limits.population` cells.
#[must_use]
pub fn live_as_polycubes(
    seed: &[Coordinates],
    rule: PolycubeLifeRule,
    limits: Limits,
    mut buds: impl FnMut() -> bool,
    mut choose: impl FnMut(usize) -> usize,
) -> Vec<Vec<LivingPolycube>> {
    let mut living = vec![LivingPolycube {
        identity: 0,
        cells: seed.to_vec(),
    }];
    let mut generations = vec![living.clone()];
    if seed.is_empty() {
        return generations;
    }
    let orientations = orientations(seed);
    let mut next_identity = 1;
    for _ in 0..limits.generation_count {
        let before = Generation::of(&living);
        let mut next: Vec<LivingPolycube> = living
            .iter()
            .enumerate()
            .filter(|(rank, _)| before.neighbour_count(*rank) <= rule.most_neighbours_to_survive)
            .map(|(_, polycube)| polycube.clone())
            .collect();
        let mut taken: HashSet<Coordinates> = before.rank_by_cell.keys().copied().collect();
        for parent in &living {
            if (next.len() + 1) * seed.len() > limits.population {
                break;
            }
            if !buds() {
                continue;
            }
            let mut birthplaces = before.birthplaces(&parent.cells, &orientations, &taken, rule);
            if birthplaces.is_empty() {
                continue;
            }
            let cells = birthplaces.swap_remove(choose(birthplaces.len()));
            taken.extend(cells.iter().copied());
            next.push(LivingPolycube {
                identity: next_identity,
                cells,
            });
            next_identity += 1;
        }
        living = next;
        generations.push(living.clone());
    }
    generations
}

/// The polycubes of one generation, found by their cells.
struct Generation<'living> {
    living: &'living [LivingPolycube],
    /// Rank in `living` of the polycube holding each cell.
    rank_by_cell: HashMap<Coordinates, usize>,
}

impl<'living> Generation<'living> {
    fn of(living: &'living [LivingPolycube]) -> Self {
        let rank_by_cell = living
            .iter()
            .enumerate()
            .flat_map(|(rank, polycube)| polycube.cells.iter().map(move |&cell| (cell, rank)))
            .collect();
        Self {
            living,
            rank_by_cell,
        }
    }

    fn neighbour_count(&self, rank: usize) -> usize {
        self.ranks_leant_on(&self.living[rank].cells)
            .into_iter()
            .filter(|&other_rank| other_rank != rank)
            .count()
    }

    /// Ranks of the polycubes sharing a face with one of `cells`.
    fn ranks_leant_on(&self, cells: &[Coordinates]) -> BTreeSet<usize> {
        cells
            .iter()
            .copied()
            .flat_map(face_neighbours)
            .filter_map(|neighbour| self.rank_by_cell.get(&neighbour).copied())
            .collect()
    }

    /// Places against `parent` where a turned seed fits over no `taken`
    /// cell and with a number of neighbours the rule allows, each set of
    /// cells once, in an order that depends on the cells alone.
    fn birthplaces(
        &self,
        parent: &[Coordinates],
        orientations: &[Vec<Coordinates>],
        taken: &HashSet<Coordinates>,
        rule: PolycubeLifeRule,
    ) -> Vec<Vec<Coordinates>> {
        let landings: BTreeSet<Coordinates> = parent
            .iter()
            .copied()
            .flat_map(face_neighbours)
            .filter(|neighbour| !taken.contains(neighbour))
            .collect();
        let allowed_neighbour_counts =
            rule.fewest_neighbours_to_be_born..=rule.most_neighbours_to_be_born;
        let mut birthplace_by_cells = BTreeMap::new();
        for orientation in orientations {
            for &anchor in orientation {
                for &landing in &landings {
                    let birthplace = moved_onto(orientation, anchor, landing);
                    if birthplace.iter().any(|cell| taken.contains(cell))
                        || !allowed_neighbour_counts
                            .contains(&self.ranks_leant_on(&birthplace).len())
                    {
                        continue;
                    }
                    let mut sorted_cells = birthplace.clone();
                    sorted_cells.sort_unstable();
                    birthplace_by_cells
                        .entry(sorted_cells)
                        .or_insert(birthplace);
                }
            }
        }
        birthplace_by_cells.into_values().collect()
    }
}
