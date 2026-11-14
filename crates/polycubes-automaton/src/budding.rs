//! A colony of whole polycubes: each one buds turned copies of itself.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use polycubes_core::Coordinates;

use crate::colony::face_neighbours;

/// Grows a colony of copies of `seed` for `generation_count` generations
/// and returns them all, the seed first. A generation lists its copies in
/// birth order, each as `seed.len()` cells in the order of the seed's.
///
/// At each generation every copy buds when `buds` says so: a copy of the
/// seed, turned any way but never mirrored, is born against it where no
/// cell is taken and where it touches no other copy. `choose_placement` picks it among the number of such
/// places, and must answer an index below that number. A copy with no room
/// around it no longer buds, and none is born that would bring the colony
/// beyond `population_limit` cells.
#[must_use]
pub fn grow_by_budding(
    seed: &[Coordinates],
    generation_count: usize,
    population_limit: usize,
    mut buds: impl FnMut() -> bool,
    mut choose_placement: impl FnMut(usize) -> usize,
) -> Vec<Vec<Coordinates>> {
    if seed.is_empty() {
        return vec![Vec::new(); generation_count + 1];
    }
    let orientations = orientations(seed);
    let mut copies = vec![seed.to_vec()];
    let mut occupied: HashSet<Coordinates> = seed.iter().copied().collect();
    let mut generations = vec![seed.to_vec()];
    for _ in 0..generation_count {
        // Copies born in this generation wait for the next one to bud.
        for parent_index in 0..copies.len() {
            if (copies.len() + 1) * seed.len() > population_limit {
                break;
            }
            if !buds() {
                continue;
            }
            let mut placements = free_placements(&copies[parent_index], &orientations, &occupied);
            if placements.is_empty() {
                continue;
            }
            let newborn = placements.swap_remove(choose_placement(placements.len()));
            occupied.extend(newborn.iter().copied());
            copies.push(newborn);
        }
        generations.push(copies.concat());
    }
    generations
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
