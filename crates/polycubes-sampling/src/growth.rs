//! Growth processes that build a polycube one cell at a time.

use std::collections::HashSet;

use polycubes_core::Coordinates;

/// Grows a polycube from a single cell by repeatedly adding one of the free
/// cells next to it, each being equally likely (Eden growth).
///
/// `choose_candidate` receives the number of such cells and returns the index
/// of the one to add. The result lists the cells in the order they were
/// added. This process is biased: compact shapes come out far more often
/// than under a uniform draw.
///
/// # Panics
///
/// Panics if `choose_candidate` returns an index that is not below the number
/// it was given.
pub fn grow_by_random_neighbour(
    cell_count: usize,
    mut choose_candidate: impl FnMut(usize) -> usize,
) -> Vec<Coordinates> {
    if cell_count == 0 {
        return Vec::new();
    }
    let seed_cell = [0, 0, 0];
    let mut cells = vec![seed_cell];
    // Cells in the polycube or already listed as candidates.
    let mut known_cells = HashSet::from([seed_cell]);
    let mut candidates = Vec::new();
    add_new_neighbours(seed_cell, &mut known_cells, &mut candidates);
    while cells.len() < cell_count {
        let added_cell = candidates.swap_remove(choose_candidate(candidates.len()));
        cells.push(added_cell);
        add_new_neighbours(added_cell, &mut known_cells, &mut candidates);
    }
    cells
}

fn add_new_neighbours(
    [x, y, z]: Coordinates,
    known_cells: &mut HashSet<Coordinates>,
    candidates: &mut Vec<Coordinates>,
) {
    let neighbours = [
        [x + 1, y, z],
        [x - 1, y, z],
        [x, y + 1, z],
        [x, y - 1, z],
        [x, y, z + 1],
        [x, y, z - 1],
    ];
    for neighbour in neighbours {
        if known_cells.insert(neighbour) {
            candidates.push(neighbour);
        }
    }
}

/// Side lengths, along x, y and z, of the smallest box holding the cells.
#[must_use]
pub fn bounding_box(cells: &[Coordinates]) -> [usize; 3] {
    [0, 1, 2].map(|axis_index| {
        let coordinates = cells.iter().map(|cell| cell[axis_index]);
        match (coordinates.clone().min(), coordinates.max()) {
            (Some(smallest), Some(largest)) => (largest - smallest) as usize + 1,
            _ => 0,
        }
    })
}
