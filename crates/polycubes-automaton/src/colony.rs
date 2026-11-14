//! A colony of cells that divide into the free cells beside them, branching
//! out: a newborn touches no cell but its mother.

use std::collections::HashSet;

use polycubes_core::Coordinates;

/// Grows a colony from `seed` for `generation_count` generations and
/// returns them all, the seed first, each with its cells in birth order.
///
/// At each generation every cell with room beside it divides when
/// `divides` says so. Room is a free face neighbour that touches no other
/// cell of the colony; `choose_neighbour` picks one among their number, and
/// must answer an index below that number. A cell with no room no longer
/// divides, and no cell is born once the colony holds `population_limit`
/// cells.
#[must_use]
pub fn grow_colony(
    seed: &[Coordinates],
    generation_count: usize,
    population_limit: usize,
    mut divides: impl FnMut() -> bool,
    mut choose_neighbour: impl FnMut(usize) -> usize,
) -> Vec<Vec<Coordinates>> {
    let mut cells = seed.to_vec();
    let mut occupied: HashSet<Coordinates> = seed.iter().copied().collect();
    let mut generations = vec![cells.clone()];
    for _ in 0..generation_count {
        // Cells born in this generation wait for the next one to divide.
        for parent_index in 0..cells.len() {
            if cells.len() >= population_limit {
                break;
            }
            let mother = cells[parent_index];
            let free_neighbours: Vec<Coordinates> = face_neighbours(mother)
                .into_iter()
                .filter(|&neighbour| is_clear_of_all_but(neighbour, mother, &occupied))
                .collect();
            if free_neighbours.is_empty() || !divides() {
                continue;
            }
            let newborn = free_neighbours[choose_neighbour(free_neighbours.len())];
            occupied.insert(newborn);
            cells.push(newborn);
        }
        generations.push(cells.clone());
    }
    generations
}

/// Whether `cell` is free and touches no cell but `mother`: newborns that
/// lean on any neighbour fill the colony into a ball, these branch out.
fn is_clear_of_all_but(
    cell: Coordinates,
    mother: Coordinates,
    occupied: &HashSet<Coordinates>,
) -> bool {
    !occupied.contains(&cell)
        && face_neighbours(cell)
            .into_iter()
            .all(|neighbour| neighbour == mother || !occupied.contains(&neighbour))
}

pub(crate) fn face_neighbours([x, y, z]: Coordinates) -> [Coordinates; 6] {
    [
        [x + 1, y, z],
        [x - 1, y, z],
        [x, y + 1, z],
        [x, y - 1, z],
        [x, y, z + 1],
        [x, y, z - 1],
    ]
}
