//! Measures of how compact a polycube is.

use std::collections::{BTreeMap, HashSet};

use polycubes_core::{Coordinates, EnumerationCursor};

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

/// Number of fixed polycubes of `cell_count` cells for each number of
/// exposed faces, by visiting them all: exact, where a sample only
/// estimates.
///
/// # Panics
///
/// Panics if `cell_count` is zero.
#[must_use]
pub fn exposed_face_distribution(cell_count: usize) -> BTreeMap<usize, u64> {
    let mut visit = FaceCountingVisit {
        cursor: EnumerationCursor::new(cell_count),
        cell_count,
        occupied: HashSet::new(),
        contact_count: 0,
        distribution: BTreeMap::new(),
    };
    visit.visit_subtree();
    visit.distribution
}

/// Walk of the enumeration tree that keeps the number of contacts between
/// cubes up to date, so that a polycube costs no more than its last cube.
struct FaceCountingVisit {
    cursor: EnumerationCursor,
    cell_count: usize,
    occupied: HashSet<Coordinates>,
    contact_count: usize,
    distribution: BTreeMap<usize, u64>,
}

impl FaceCountingVisit {
    fn visit_subtree(&mut self) {
        if self.occupied.len() == self.cell_count {
            // Each contact hides one face of each of its two cubes.
            let exposed_face_count = self.cell_count * FACES_PER_CUBE - 2 * self.contact_count;
            *self.distribution.entry(exposed_face_count).or_insert(0) += 1;
            return;
        }
        for (candidate_index, candidate) in self.cursor.candidates().into_iter().enumerate() {
            let new_contact_count = neighbours(candidate)
                .iter()
                .filter(|neighbour| self.occupied.contains(*neighbour))
                .count();
            self.cursor.descend(candidate_index);
            self.occupied.insert(candidate);
            self.contact_count += new_contact_count;
            self.visit_subtree();
            self.contact_count -= new_contact_count;
            self.occupied.remove(&candidate);
            self.cursor.ascend();
        }
    }
}
