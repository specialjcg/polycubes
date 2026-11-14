//! A colony of cells that divide into the free cells beside them.

use std::collections::HashSet;

use polycubes_core::Coordinates;

/// Grows a colony from `seed` for `generation_count` generations and
/// returns them all, the seed first, each with its cells in birth order.
///
/// At each generation every cell with a free face neighbour divides when
/// `divides` says so, into the free neighbour picked by `choose_neighbour`
/// among their number, which must answer an index below that number. A cell hemmed in on all sides no longer divides, and
/// no cell is born once the colony holds `population_limit` cells.
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
            let free_neighbours: Vec<Coordinates> = face_neighbours(cells[parent_index])
                .into_iter()
                .filter(|neighbour| !occupied.contains(neighbour))
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
