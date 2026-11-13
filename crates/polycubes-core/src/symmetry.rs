//! Polycubes counted up to rotation, with Burnside's lemma.
//!
//! The number of one-sided polycubes is the average, over the 24 rotations of
//! the cube, of the number of fixed polycubes that the rotation maps onto a
//! translate of themselves. The identity contributes every fixed polycube.
//!
//! For another rotation, take each polycube with its smallest cell at the
//! origin: it is counted when some rigid motion "rotation then shift" maps it
//! onto itself. Polycubes are therefore enumerated shift by shift.

use std::num::NonZeroUsize;

use crate::lattice::{Coordinates, Lattice, NEIGHBOUR_COUNT};
use crate::redelmeier::count_fixed_polycubes_in_parallel;

const ROTATION_GROUP_ORDER: u64 = 24;

/// A non-trivial rotation of the cube, standing for its whole conjugacy
/// class: conjugate rotations leave the same number of polycubes unchanged.
struct RotationClass {
    /// Rows of the rotation matrix.
    matrix: [Coordinates; 3],
    axis: Coordinates,
    class_size: u64,
}

const ROTATION_CLASSES: [RotationClass; 4] = [
    // Half turn about a face axis: (x, y, z) -> (-x, -y, z).
    RotationClass {
        matrix: [[-1, 0, 0], [0, -1, 0], [0, 0, 1]],
        axis: [0, 0, 1],
        class_size: 3,
    },
    // Quarter turn about a face axis: (x, y, z) -> (-y, x, z).
    RotationClass {
        matrix: [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
        axis: [0, 0, 1],
        class_size: 6,
    },
    // Half turn about an edge axis: (x, y, z) -> (y, x, -z).
    RotationClass {
        matrix: [[0, 1, 0], [1, 0, 0], [0, 0, -1]],
        axis: [1, 1, 0],
        class_size: 6,
    },
    // Third of a turn about a vertex axis: (x, y, z) -> (y, z, x).
    RotationClass {
        matrix: [[0, 1, 0], [0, 0, 1], [1, 0, 0]],
        axis: [1, 1, 1],
        class_size: 8,
    },
];

/// Counts one-sided polycubes (distinct up to translation and rotation,
/// mirror images counted separately) of every size from 1 to `max_cell_count`.
///
/// Element `size - 1` of the result is OEIS A000162 at `size`.
#[must_use]
pub fn count_one_sided_polycubes(max_cell_count: usize, thread_count: NonZeroUsize) -> Vec<u64> {
    let fixed_counts = count_fixed_polycubes_in_parallel(max_cell_count, thread_count);
    one_sided_counts_from_fixed(&fixed_counts)
}

/// Derives the one-sided counts from the fixed counts of sizes 1 to
/// `fixed_counts.len()`, by enumerating the rotation-symmetric polycubes.
///
/// # Panics
///
/// Panics if `fixed_counts` are not the numbers of fixed polycubes: the
/// Burnside totals would then not be multiples of the group order.
#[must_use]
pub fn one_sided_counts_from_fixed(fixed_counts: &[u64]) -> Vec<u64> {
    let max_cell_count = fixed_counts.len();
    if max_cell_count == 0 {
        return Vec::new();
    }
    let mut fixed_point_totals = fixed_counts.to_vec();
    for rotation_class in &ROTATION_CLASSES {
        let symmetric_counts = count_symmetric_polycubes(rotation_class, max_cell_count);
        for (fixed_point_total, symmetric_count) in
            fixed_point_totals.iter_mut().zip(symmetric_counts)
        {
            *fixed_point_total += rotation_class.class_size * symmetric_count;
        }
    }
    fixed_point_totals
        .into_iter()
        .map(|fixed_point_total| {
            assert_eq!(
                fixed_point_total % ROTATION_GROUP_ORDER,
                0,
                "Burnside total must be a multiple of the group order"
            );
            fixed_point_total / ROTATION_GROUP_ORDER
        })
        .collect()
}

/// Counts, for every size, the fixed polycubes mapped onto a translate of
/// themselves by the rotation.
fn count_symmetric_polycubes(rotation_class: &RotationClass, max_cell_count: usize) -> Vec<u64> {
    let lattice = Lattice::new(max_cell_count);
    let mut symmetric_counts = vec![0; max_cell_count];
    for shift in candidate_shifts(&lattice, rotation_class.axis, max_cell_count) {
        let motion = RigidMotion {
            matrix: rotation_class.matrix,
            shift,
        };
        let mut enumerator = SymmetricEnumerator::new(&lattice, motion, max_cell_count);
        enumerator.extend(0, 0);
        for (symmetric_count, invariant_count) in symmetric_counts.iter_mut().zip(enumerator.counts)
        {
            *symmetric_count += invariant_count;
        }
    }
    symmetric_counts
}

/// Shifts for which an invariant polycube may exist.
///
/// The shift is the image of the origin, hence a cell of the polycube; and
/// the motion must fix the centroid, which forces the shift to be
/// perpendicular to the rotation axis.
fn candidate_shifts(
    lattice: &Lattice,
    axis: Coordinates,
    max_cell_count: usize,
) -> Vec<Coordinates> {
    let farthest_cell_distance = max_cell_count as i32 - 1;
    let origin = lattice.origin();
    lattice
        .forbidden_cells()
        .iter()
        .enumerate()
        .filter(|&(cell, &is_forbidden)| cell == origin || !is_forbidden)
        .map(|(cell, _)| lattice.coordinates(cell))
        .filter(|&shift| dot_product(shift, axis) == 0)
        .filter(|shift| {
            shift.iter().map(|coordinate| coordinate.abs()).sum::<i32>() <= farthest_cell_distance
        })
        .collect()
}

fn dot_product(left: Coordinates, right: Coordinates) -> i32 {
    left.iter()
        .zip(right)
        .map(|(left_coordinate, right_coordinate)| left_coordinate * right_coordinate)
        .sum()
}

/// A rotation followed by a translation.
struct RigidMotion {
    matrix: [Coordinates; 3],
    shift: Coordinates,
}

impl RigidMotion {
    fn image(&self, point: Coordinates) -> Coordinates {
        let rotated = self.matrix.map(|matrix_row| dot_product(matrix_row, point));
        [0, 1, 2].map(|axis_index| rotated[axis_index] + self.shift[axis_index])
    }

    fn preimage(&self, point: Coordinates) -> Coordinates {
        let unshifted = [0, 1, 2].map(|axis_index| point[axis_index] - self.shift[axis_index]);
        // The inverse of a rotation matrix is its transpose.
        [0, 1, 2].map(|column_index| {
            let matrix_column = self.matrix.map(|matrix_row| matrix_row[column_index]);
            dot_product(matrix_column, unshifted)
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellState {
    Free,
    Untried,
    Placed,
    /// Discarded by an ancestor, or forbidden.
    Closed,
}

/// Redelmeier enumeration restricted to polycubes invariant under a rigid
/// motion: a branch is abandoned as soon as the images still to be placed
/// cannot fit, or one of them has been discarded.
struct SymmetricEnumerator<'lattice> {
    lattice: &'lattice Lattice,
    motion: RigidMotion,
    states: Vec<CellState>,
    untried_cells: Vec<usize>,
    neighbour_offsets: [isize; NEIGHBOUR_COUNT],
    /// Placed cells whose image is not placed yet.
    pending_image_count: usize,
    counts: Vec<u64>,
    max_cell_count: usize,
}

impl<'lattice> SymmetricEnumerator<'lattice> {
    fn new(lattice: &'lattice Lattice, motion: RigidMotion, max_cell_count: usize) -> Self {
        let origin = lattice.origin();
        let mut states: Vec<CellState> = lattice
            .forbidden_cells()
            .iter()
            .map(|&is_forbidden| {
                if is_forbidden {
                    CellState::Closed
                } else {
                    CellState::Free
                }
            })
            .collect();
        states[origin] = CellState::Untried;
        Self {
            lattice,
            motion,
            states,
            untried_cells: vec![origin],
            neighbour_offsets: lattice.neighbour_offsets(),
            pending_image_count: 0,
            counts: vec![0; max_cell_count],
            max_cell_count,
        }
    }

    fn extend(&mut self, first_untried: usize, placed_cell_count: usize) {
        let untried_end = self.untried_cells.len();
        let cell_count = placed_cell_count + 1;
        for untried_index in first_untried..untried_end {
            let cell = self.untried_cells[untried_index];
            let pending_image_count_before = self.pending_image_count;
            if self.place(cell) {
                self.count_and_extend(untried_index, cell, cell_count);
            }
            self.states[cell] = CellState::Closed;
            self.pending_image_count = pending_image_count_before;
            // A placed cell maps onto this one: no later sibling can succeed.
            if self.is_image_of_placed_cell(cell) {
                break;
            }
        }
        for &cell in &self.untried_cells[first_untried..untried_end] {
            self.states[cell] = CellState::Untried;
        }
    }

    fn count_and_extend(&mut self, untried_index: usize, cell: usize, cell_count: usize) {
        if self.pending_image_count == 0 {
            self.counts[cell_count - 1] += 1;
        }
        let can_grow = cell_count < self.max_cell_count
            && cell_count + self.pending_image_count <= self.max_cell_count;
        if !can_grow {
            return;
        }
        let untried_end = self.untried_cells.len();
        self.queue_free_neighbours(cell);
        self.extend(untried_index + 1, cell_count);
        for freed_cell in self.untried_cells.drain(untried_end..) {
            self.states[freed_cell] = CellState::Free;
        }
    }

    /// Places the cell unless its image can never be placed.
    fn place(&mut self, cell: usize) -> bool {
        let Some(image) = self.image_cell(cell) else {
            return false;
        };
        if image != cell {
            match self.states[image] {
                CellState::Closed => return false,
                CellState::Placed => {}
                CellState::Free | CellState::Untried => self.pending_image_count += 1,
            }
            if self.is_image_of_placed_cell(cell) {
                self.pending_image_count -= 1;
            }
        }
        self.states[cell] = CellState::Placed;
        true
    }

    fn image_cell(&self, cell: usize) -> Option<usize> {
        self.lattice
            .cell_at(self.motion.image(self.lattice.coordinates(cell)))
    }

    fn is_image_of_placed_cell(&self, cell: usize) -> bool {
        let preimage_coordinates = self.motion.preimage(self.lattice.coordinates(cell));
        self.lattice
            .cell_at(preimage_coordinates)
            .is_some_and(|preimage| preimage != cell && self.states[preimage] == CellState::Placed)
    }

    fn queue_free_neighbours(&mut self, cell: usize) {
        for offset in self.neighbour_offsets {
            let neighbour = cell.wrapping_add_signed(offset);
            if self.states[neighbour] == CellState::Free {
                self.states[neighbour] = CellState::Untried;
                self.untried_cells.push(neighbour);
            }
        }
    }
}
