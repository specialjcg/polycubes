//! Measures of how compact a polycube is.

use std::collections::HashSet;

use polycubes_core::Coordinates;

const FACES_PER_CUBE: usize = 6;

/// Number of cube faces on the outside of the polycube: the fewer, the more
/// compact. A polycube of `n` cubes without any loop shows `4n + 2` faces.
#[must_use]
pub fn exposed_face_count(cells: &[Coordinates]) -> usize {
    let occupied: HashSet<Coordinates> = cells.iter().copied().collect();
    let hidden_face_count = cells
        .iter()
        .flat_map(|&cell| neighbours(cell))
        .filter(|neighbour| occupied.contains(neighbour))
        .count();
    cells.len() * FACES_PER_CUBE - hidden_face_count
}

fn neighbours([x, y, z]: Coordinates) -> [Coordinates; FACES_PER_CUBE] {
    [
        [x - 1, y, z],
        [x + 1, y, z],
        [x, y - 1, z],
        [x, y + 1, z],
        [x, y, z - 1],
        [x, y, z + 1],
    ]
}
