//! The polycubes of a size that hold the most loops.

use std::collections::BTreeMap;

use polycubes_core::Coordinates;

use crate::measure::visit_with_contact_counts;

/// A polycube up to rotation, with the number of fixed polycubes it stands
/// for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    /// Cells of one orientation, sorted, the smallest coordinates at zero.
    pub cells: Vec<Coordinates>,
    pub orientation_count: u64,
}

/// The shapes of a size with the most loops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MostLooped {
    /// Independent loops: contacts between cubes beyond those of a tree.
    pub loop_count: usize,
    pub shapes: Vec<Shape>,
}

/// Finds the polycubes of `cell_count` cells with the most loops by visiting
/// every fixed polycube, and groups those that differ by a rotation.
///
/// # Panics
///
/// Panics if `cell_count` is zero.
#[must_use]
pub fn most_looped_polycubes(cell_count: usize) -> MostLooped {
    let mut most_contact_count = 0;
    let mut orientation_counts: BTreeMap<Vec<Coordinates>, u64> = BTreeMap::new();
    visit_with_contact_counts(cell_count, |contact_count, cells| {
        if contact_count < most_contact_count {
            return;
        }
        if contact_count > most_contact_count {
            most_contact_count = contact_count;
            orientation_counts.clear();
        }
        let cells: Vec<Coordinates> = cells.iter().copied().collect();
        *orientation_counts
            .entry(smallest_rotation(&cells))
            .or_insert(0) += 1;
    });
    MostLooped {
        // A tree of `cell_count` cubes has one contact less than cubes.
        loop_count: most_contact_count + 1 - cell_count,
        shapes: orientation_counts
            .into_iter()
            .map(|(cells, orientation_count)| Shape {
                cells,
                orientation_count,
            })
            .collect(),
    }
}

/// An axis of the turned polycube: the axis it comes from and whether it is
/// reversed.
type TurnedAxis = (usize, i32);

const AXIS_ORDERS: [([usize; 3], i32); 6] = [
    ([0, 1, 2], 1),
    ([1, 2, 0], 1),
    ([2, 0, 1], 1),
    ([0, 2, 1], -1),
    ([2, 1, 0], -1),
    ([1, 0, 2], -1),
];

/// The 24 rotations of the cube: axes are permuted and reversed, keeping
/// those moves that do not mirror the polycube.
fn rotations() -> Vec<[TurnedAxis; 3]> {
    let mut rotations = Vec::with_capacity(24);
    for (axis_order, order_handedness) in AXIS_ORDERS {
        for reversed_axes in 0..8 {
            let directions = [0, 1, 2].map(|axis| {
                if reversed_axes >> axis & 1 == 1 {
                    -1
                } else {
                    1
                }
            });
            let handedness: i32 = order_handedness * directions.iter().product::<i32>();
            if handedness == 1 {
                rotations.push([0, 1, 2].map(|axis| (axis_order[axis], directions[axis])));
            }
        }
    }
    rotations
}

/// The same polycube for every polycube that differs by a rotation and a
/// translation: among all its orientations, moved against the origin and
/// sorted, the first in lexicographic order.
fn smallest_rotation(cells: &[Coordinates]) -> Vec<Coordinates> {
    rotations()
        .into_iter()
        .map(|rotation| {
            let turned: Vec<Coordinates> = cells
                .iter()
                .map(|cell| rotation.map(|(source_axis, direction)| direction * cell[source_axis]))
                .collect();
            moved_against_origin(turned)
        })
        .min()
        .unwrap_or_default()
}

fn moved_against_origin(mut cells: Vec<Coordinates>) -> Vec<Coordinates> {
    let lowest_corner =
        [0, 1, 2].map(|axis| cells.iter().map(|cell| cell[axis]).min().unwrap_or(0));
    for cell in &mut cells {
        for axis in 0..3 {
            cell[axis] -= lowest_corner[axis];
        }
    }
    cells.sort_unstable();
    cells
}
