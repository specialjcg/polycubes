//! A game of life after Conway's whose living unit is a whole polycube.
//!
//! Unlike `polycube_life`, nobody buds: a polycube is born, with no parent
//! and no chance, wherever it would fit snugly between the right number of
//! living ones, and dies when too few or too many lean on it. Births are
//! scarce, so the population stays bounded and patterns that move appear.

use std::collections::{BTreeSet, HashMap};

use polycubes_core::Coordinates;

use crate::budding::{moved_onto, orientations};
use crate::colony::face_neighbours;
use crate::polycube_life::LivingPolycube;

/// Neighbours of a polycube are the other polycubes it shares a face with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConwayRule {
    pub fewest_neighbours_to_survive: usize,
    pub most_neighbours_to_survive: usize,
    pub fewest_neighbours_to_be_born: usize,
    pub most_neighbours_to_be_born: usize,
    /// A newborn must share at least that many faces with the living: it
    /// fits snugly, where touching would let births happen almost anywhere.
    pub fewest_faces_shared_to_be_born: usize,
}

/// A cube of `side` cells whose opposite faces are glued together: what
/// leaves on one side comes back on the other. Cells stand between 0 and
/// `side - 1` along each axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Torus {
    pub side: i32,
}

/// Where and how densely a soup is poured: `attempt_count` polycubes are
/// tried inside a cube of `side` cells from the origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Soup {
    pub side: i32,
    pub attempt_count: usize,
}

pub const DOMINO: [Coordinates; 2] = [[0, 0, 0], [1, 0, 0]];

/// Four dominoes that, under the rule of the Automaton page, come back as
/// they were every ten generations, five cells further along a diagonal.
pub const DOMINO_GLIDER: [[Coordinates; 2]; 4] = [
    [[0, 0, 0], [0, 1, 0]],
    [[0, 2, 0], [1, 2, 0]],
    [[1, 1, 0], [2, 1, 0]],
    [[2, 0, 0], [3, 0, 0]],
];

/// Plays `generation_count` generations from `start` and returns them all,
/// `start` first.
///
/// All polycubes change at once, from the generation before. One dies
/// unless the number of its neighbours lies within the rule. A polycube of
/// the shape of the first one of `start`, turned any way, never mirrored,
/// is born over free cells where it would have a number of neighbours within
/// the rule and share enough faces with them, unless another such birth
/// would overlap or touch it: no birth is preferred to another, so a turned
/// start plays the same game turned.
#[must_use]
pub fn play_conway(
    start: &[Vec<Coordinates>],
    rule: ConwayRule,
    torus: Torus,
    generation_count: usize,
) -> Vec<Vec<LivingPolycube>> {
    let mut living: Vec<LivingPolycube> = start
        .iter()
        .enumerate()
        .map(|(identity, cells)| LivingPolycube {
            identity,
            cells: cells.iter().map(|&cell| torus.wrapped(cell)).collect(),
        })
        .collect();
    let mut generations = vec![living.clone()];
    let Some(shape) = start.first() else {
        return generations;
    };
    let turns = orientations(shape);
    let mut next_identity = start.len();
    for _ in 0..generation_count {
        let before = Generation::of(&living, torus);
        let mut next: Vec<LivingPolycube> = living
            .iter()
            .enumerate()
            .filter(|(rank, _)| before.survives(*rank, rule))
            .map(|(_, polycube)| polycube.clone())
            .collect();
        for cells in before.births(&turns, rule) {
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

/// Polycubes of `shape`, turned at random, poured into the cube of the
/// soup: each attempt that would overlap one already poured is dropped.
/// `choose` picks each turn and place among a number of choices, and must
/// answer an index below that number.
#[must_use]
pub fn soup(
    shape: &[Coordinates],
    soup: Soup,
    mut choose: impl FnMut(usize) -> usize,
) -> Vec<Vec<Coordinates>> {
    let turns: Vec<Vec<Coordinates>> = orientations(shape)
        .iter()
        .map(|turn| against_origin(turn))
        .collect();
    let mut poured: Vec<Vec<Coordinates>> = Vec::new();
    let mut taken = BTreeSet::new();
    for _ in 0..soup.attempt_count {
        let turn = &turns[choose(turns.len())];
        let reach = turn.iter().flatten().copied().max().unwrap_or(0);
        let room = usize::try_from(soup.side - reach).unwrap_or(0).max(1);
        let corner = [0, 1, 2].map(|_| i32::try_from(choose(room)).unwrap_or(0));
        let cells = moved_onto(turn, [0, 0, 0], corner);
        if cells.iter().all(|cell| !taken.contains(cell)) {
            taken.extend(cells.iter().copied());
            poured.push(cells);
        }
    }
    poured
}

/// `cells` moved so that their lowest corner is the origin.
fn against_origin(cells: &[Coordinates]) -> Vec<Coordinates> {
    let lowest_corner =
        [0, 1, 2].map(|axis| cells.iter().map(|cell| cell[axis]).min().unwrap_or(0));
    moved_onto(cells, lowest_corner, [0, 0, 0])
}

impl Torus {
    fn wrapped(self, cell: Coordinates) -> Coordinates {
        cell.map(|coordinate| coordinate.rem_euclid(self.side))
    }

    fn face_neighbours(self, cell: Coordinates) -> [Coordinates; 6] {
        face_neighbours(cell).map(|neighbour| self.wrapped(neighbour))
    }
}

/// The polycubes of one generation, found by their cells.
struct Generation<'living> {
    living: &'living [LivingPolycube],
    torus: Torus,
    /// Rank in `living` of the polycube holding each cell.
    rank_by_cell: HashMap<Coordinates, usize>,
}

impl<'living> Generation<'living> {
    fn of(living: &'living [LivingPolycube], torus: Torus) -> Self {
        let rank_by_cell = living
            .iter()
            .enumerate()
            .flat_map(|(rank, polycube)| polycube.cells.iter().map(move |&cell| (cell, rank)))
            .collect();
        Self {
            living,
            torus,
            rank_by_cell,
        }
    }

    fn survives(&self, rank: usize, rule: ConwayRule) -> bool {
        let neighbour_count = self
            .ranks_leant_on(&self.living[rank].cells)
            .into_iter()
            .filter(|&other_rank| other_rank != rank)
            .count();
        (rule.fewest_neighbours_to_survive..=rule.most_neighbours_to_survive)
            .contains(&neighbour_count)
    }

    /// Ranks of the polycubes sharing a face with one of `cells`.
    fn ranks_leant_on(&self, cells: &[Coordinates]) -> BTreeSet<usize> {
        self.neighbours_of(cells)
            .filter_map(|neighbour| self.rank_by_cell.get(&neighbour).copied())
            .collect()
    }

    fn neighbours_of<'cells>(
        &self,
        cells: &'cells [Coordinates],
    ) -> impl Iterator<Item = Coordinates> + 'cells {
        let torus = self.torus;
        cells
            .iter()
            .flat_map(move |&cell| torus.face_neighbours(cell))
    }

    /// Births the rule allows that no other overlaps or touches.
    fn births(&self, turns: &[Vec<Coordinates>], rule: ConwayRule) -> Vec<Vec<Coordinates>> {
        let candidates = self.birthplaces(turns, rule);
        let mut claim_count: HashMap<Coordinates, usize> = HashMap::new();
        for cells in &candidates {
            for cell in self.cells_and_neighbours(cells) {
                *claim_count.entry(cell).or_default() += 1;
            }
        }
        candidates
            .into_iter()
            .filter(|cells| cells.iter().all(|cell| claim_count.get(cell) == Some(&1)))
            .collect()
    }

    fn cells_and_neighbours(&self, cells: &[Coordinates]) -> BTreeSet<Coordinates> {
        self.neighbours_of(cells)
            .chain(cells.iter().copied())
            .collect()
    }

    /// Free places where a turned polycube would be born, each set of cells
    /// once, its cells sorted, in an order that depends on the cells alone.
    fn birthplaces(
        &self,
        turns: &[Vec<Coordinates>],
        rule: ConwayRule,
    ) -> BTreeSet<Vec<Coordinates>> {
        let landings: BTreeSet<Coordinates> = self
            .neighbours_of(&self.rank_by_cell.keys().copied().collect::<Vec<_>>())
            .filter(|neighbour| !self.rank_by_cell.contains_key(neighbour))
            .collect();
        let mut birthplaces = BTreeSet::new();
        for &landing in &landings {
            for turn in turns {
                for &anchor in turn {
                    let mut cells: Vec<Coordinates> = moved_onto(turn, anchor, landing)
                        .into_iter()
                        .map(|cell| self.torus.wrapped(cell))
                        .collect();
                    if self.allows_birth(&cells, rule) {
                        cells.sort_unstable();
                        birthplaces.insert(cells);
                    }
                }
            }
        }
        birthplaces
    }

    fn allows_birth(&self, cells: &[Coordinates], rule: ConwayRule) -> bool {
        if cells
            .iter()
            .any(|cell| self.rank_by_cell.contains_key(cell))
        {
            return false;
        }
        let faces_shared = self
            .neighbours_of(cells)
            .filter(|neighbour| self.rank_by_cell.contains_key(neighbour))
            .count();
        faces_shared >= rule.fewest_faces_shared_to_be_born
            && (rule.fewest_neighbours_to_be_born..=rule.most_neighbours_to_be_born)
                .contains(&self.ranks_leant_on(cells).len())
    }
}
