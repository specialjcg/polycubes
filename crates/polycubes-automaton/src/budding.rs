//! A colony of whole polycubes: each one buds turned copies of itself, true
//! to its shape or with one cell moved.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use polycubes_core::Coordinates;

use crate::colony::face_neighbours;

/// The generations of a colony of polycubes, and which bore which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    /// The seed first. A generation lists its polycubes in birth order,
    /// each as many cells as the seed.
    pub generations: Vec<Vec<Coordinates>>,
    /// Birth rank of the parent of each polycube; the seed is its own.
    pub parent_of_copy: Vec<usize>,
}

/// Picks one among a number of choices: must answer an index below it.
type Choice<'choice> = &'choice mut dyn FnMut(usize) -> usize;

/// Grows a colony of copies of `seed` for `generation_count` generations
/// and returns them all, the seed first. A generation lists its copies in
/// birth order, each as `seed.len()` cells in the order of the seed's.
///
/// At each generation every copy buds when `buds` says so: a copy of the
/// seed, turned any way but never mirrored, is born against it where no
/// cell is taken and where it touches no other copy. `choose_placement`
/// picks it among the number of such places, and must answer an index
/// below that number. A copy with no room around it no longer buds, and
/// none is born that would bring the colony beyond `population_limit`
/// cells.
#[must_use]
pub fn grow_by_budding(
    seed: &[Coordinates],
    generation_count: usize,
    population_limit: usize,
    mut buds: impl FnMut() -> bool,
    mut choose_placement: impl FnMut(usize) -> usize,
) -> Vec<Vec<Coordinates>> {
    let limits = ColonyLimits {
        generation_count,
        population: population_limit,
    };
    grow_lineage(seed, limits, &mut buds, &mut choose_placement, same_shape).generations
}

/// Grows a colony like [`grow_by_budding`], but each bud is its parent with
/// one cell moved, so that shapes drift along the branches. `choose` picks
/// both the moved cell and the place of the bud, each among a number of
/// choices, and must answer an index below that number.
#[must_use]
pub fn grow_by_mutating_buds(
    seed: &[Coordinates],
    generation_count: usize,
    population_limit: usize,
    mut buds: impl FnMut() -> bool,
    mut choose: impl FnMut(usize) -> usize,
) -> Lineage {
    let limits = ColonyLimits {
        generation_count,
        population: population_limit,
    };
    grow_lineage(seed, limits, &mut buds, &mut choose, one_cell_moved)
}

#[derive(Clone, Copy)]
struct ColonyLimits {
    generation_count: usize,
    /// In cells.
    population: usize,
}

fn grow_lineage(
    seed: &[Coordinates],
    limits: ColonyLimits,
    buds: &mut dyn FnMut() -> bool,
    choose: Choice,
    shape_of_bud: fn(&[Coordinates], Choice) -> Vec<Coordinates>,
) -> Lineage {
    if seed.is_empty() {
        return Lineage {
            generations: vec![Vec::new(); limits.generation_count + 1],
            parent_of_copy: vec![0],
        };
    }
    let mut copies = vec![seed.to_vec()];
    let mut parent_of_copy = vec![0];
    let mut occupied: HashSet<Coordinates> = seed.iter().copied().collect();
    let mut generations = vec![seed.to_vec()];
    for _ in 0..limits.generation_count {
        // Copies born in this generation wait for the next one to bud.
        for parent_index in 0..copies.len() {
            if (copies.len() + 1) * seed.len() > limits.population {
                break;
            }
            if !buds() {
                continue;
            }
            let parent = &copies[parent_index];
            let bud_orientations = orientations(&shape_of_bud(parent, choose));
            let mut placements = free_placements(parent, &bud_orientations, &occupied);
            if placements.is_empty() {
                continue;
            }
            let newborn = placements.swap_remove(choose(placements.len()));
            occupied.extend(newborn.iter().copied());
            copies.push(newborn);
            parent_of_copy.push(parent_index);
        }
        generations.push(copies.concat());
    }
    Lineage {
        generations,
        parent_of_copy,
    }
}

fn same_shape(parent: &[Coordinates], _choose: Choice) -> Vec<Coordinates> {
    parent.to_vec()
}

/// `parent` with one cell taken off and put back elsewhere against the
/// others, still in one piece; `parent` itself when no cell can move.
fn one_cell_moved(parent: &[Coordinates], choose: Choice) -> Vec<Coordinates> {
    let mut mutants = BTreeSet::new();
    for moved_index in 0..parent.len() {
        let mut kept = parent.to_vec();
        let moved = kept.remove(moved_index);
        if kept.is_empty() || !is_in_one_piece(&kept) {
            continue;
        }
        for landing in kept.iter().copied().flat_map(face_neighbours) {
            if landing != moved && !kept.contains(&landing) {
                let mut mutant = kept.clone();
                mutant.push(landing);
                mutant.sort_unstable();
                mutants.insert(mutant);
            }
        }
    }
    if mutants.is_empty() {
        return parent.to_vec();
    }
    let chosen_index = choose(mutants.len());
    mutants
        .into_iter()
        .nth(chosen_index)
        .unwrap_or_else(|| parent.to_vec())
}

fn is_in_one_piece(cells: &[Coordinates]) -> bool {
    let Some(&first_cell) = cells.first() else {
        return true;
    };
    let mut reached = vec![first_cell];
    let mut explored_count = 0;
    while explored_count < reached.len() {
        for neighbour in face_neighbours(reached[explored_count]) {
            if cells.contains(&neighbour) && !reached.contains(&neighbour) {
                reached.push(neighbour);
            }
        }
        explored_count += 1;
    }
    reached.len() == cells.len()
}

/// Every way to turn `seed`, each once: quarter turns about two axes reach
/// all 24 rotations of the cube, and no mirror image.
fn orientations(seed: &[Coordinates]) -> Vec<Vec<Coordinates>> {
    let quarter_turns: [fn(Coordinates) -> Coordinates; 2] =
        [|[x, y, z]| [x, -z, y], |[x, y, z]| [z, y, -x]];
    let mut orientations = vec![seed.to_vec()];
    let mut turned_count = 0;
    while turned_count < orientations.len() {
        for quarter_turn in quarter_turns {
            let turned: Vec<Coordinates> = orientations[turned_count]
                .iter()
                .copied()
                .map(quarter_turn)
                .collect();
            if !orientations.contains(&turned) {
                orientations.push(turned);
            }
        }
        turned_count += 1;
    }
    orientations
}

/// Copies that touch `parent`, overlap nothing and touch no other copy, each
/// set of cells once,
/// in an order that depends on the cells alone.
fn free_placements(
    parent: &[Coordinates],
    orientations: &[Vec<Coordinates>],
    occupied: &HashSet<Coordinates>,
) -> Vec<Vec<Coordinates>> {
    let landings: BTreeSet<Coordinates> = parent
        .iter()
        .copied()
        .flat_map(face_neighbours)
        .filter(|neighbour| !occupied.contains(neighbour))
        .collect();
    // A symmetric seed reaches the same cells through several turns.
    let mut placement_by_cells = BTreeMap::new();
    for orientation in orientations {
        for &anchor in orientation {
            for &landing in &landings {
                let placement = moved_onto(orientation, anchor, landing);
                if !has_room(&placement, parent, occupied) {
                    continue;
                }
                let mut sorted_cells = placement.clone();
                sorted_cells.sort_unstable();
                placement_by_cells.entry(sorted_cells).or_insert(placement);
            }
        }
    }
    placement_by_cells.into_values().collect()
}

/// Whether `placement` overlaps nothing and leans on no copy but `parent`:
/// buds that pack against every neighbour would hide the copies in a lump.
fn has_room(
    placement: &[Coordinates],
    parent: &[Coordinates],
    occupied: &HashSet<Coordinates>,
) -> bool {
    placement.iter().all(|&cell| {
        !occupied.contains(&cell)
            && face_neighbours(cell)
                .iter()
                .all(|neighbour| !occupied.contains(neighbour) || parent.contains(neighbour))
    })
}

/// `cells` shifted so that its cell `anchor` lands on `landing`.
fn moved_onto(
    cells: &[Coordinates],
    anchor: Coordinates,
    landing: Coordinates,
) -> Vec<Coordinates> {
    cells
        .iter()
        .map(|cell| [0, 1, 2].map(|axis| cell[axis] - anchor[axis] + landing[axis]))
        .collect()
}
