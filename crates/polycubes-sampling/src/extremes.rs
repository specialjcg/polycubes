//! The polycubes of a size that hold the most loops.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};

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
    let (most_contact_count, shapes) =
        best_shapes(cell_count, |contact_count, _| Some(contact_count), |_| true)
            .unwrap_or_default();
    MostLooped {
        loop_count: loop_count(cell_count, most_contact_count),
        shapes,
    }
}

/// A tree of `cell_count` cubes has one contact less than cubes; every
/// further contact closes a loop.
fn loop_count(cell_count: usize, contact_count: usize) -> usize {
    contact_count + 1 - cell_count
}

/// Visits every fixed polycube of `cell_count` cells and keeps those of the
/// greatest rank, grouped by rotation. `rank_of` receives the number of
/// contacts and the cells, and leaves a polycube out by returning `None`;
/// `is_kept` is a costlier filter, only asked about polycubes that would
/// otherwise be kept.
fn best_shapes<Rank: Ord>(
    cell_count: usize,
    mut rank_of: impl FnMut(usize, &HashSet<Coordinates>) -> Option<Rank>,
    mut is_kept: impl FnMut(&HashSet<Coordinates>) -> bool,
) -> Option<(Rank, Vec<Shape>)> {
    let mut best_rank: Option<Rank> = None;
    let mut orientation_counts: BTreeMap<Vec<Coordinates>, u64> = BTreeMap::new();
    visit_with_contact_counts(cell_count, |contact_count, cells| {
        let Some(rank) = rank_of(contact_count, cells) else {
            return;
        };
        if best_rank
            .as_ref()
            .is_some_and(|best_rank| rank < *best_rank)
            || !is_kept(cells)
        {
            return;
        }
        if best_rank.as_ref().is_none_or(|best_rank| rank > *best_rank) {
            best_rank = Some(rank);
            orientation_counts.clear();
        }
        let cells: Vec<Coordinates> = cells.iter().copied().collect();
        *orientation_counts
            .entry(smallest_rotation(&cells))
            .or_insert(0) += 1;
    });
    let shapes = orientation_counts
        .into_iter()
        .map(|(cells, orientation_count)| Shape {
            cells,
            orientation_count,
        })
        .collect();
    best_rank.map(|best_rank| (best_rank, shapes))
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

/// The trees of a size that fit in the smallest box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TightestTrees {
    /// Side lengths of that box, from the shortest to the longest.
    pub bounding_box: [usize; 3],
    pub shapes: Vec<Shape>,
}

/// Finds the polycubes with the most loops among those where no four cubes
/// form a 2 × 2 square: their loops go around something. `None` when no
/// such polycube of `cell_count` cells has a loop.
///
/// # Panics
///
/// Panics if `cell_count` is zero.
#[must_use]
pub fn most_looped_square_free_polycubes(cell_count: usize) -> Option<MostLooped> {
    let (most_contact_count, shapes) = best_shapes(
        cell_count,
        |contact_count, _| (contact_count >= cell_count).then_some(contact_count),
        |cells| !holds_a_square(cells),
    )?;
    Some(MostLooped {
        loop_count: loop_count(cell_count, most_contact_count),
        shapes,
    })
}

/// Pairs of axes spanning the three planes a square may lie in.
const PLANES: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];

fn holds_a_square(cells: &HashSet<Coordinates>) -> bool {
    cells.iter().any(|&cell| {
        PLANES.iter().any(|&(first_axis, second_axis)| {
            let mut beside = cell;
            beside[first_axis] += 1;
            let mut above = cell;
            above[second_axis] += 1;
            let mut across = beside;
            across[second_axis] += 1;
            [beside, above, across]
                .iter()
                .all(|corner| cells.contains(corner))
        })
    })
}

/// Finds the polycubes without loop whose box has the shortest longest
/// side, and among those the smallest volume.
///
/// # Panics
///
/// Panics if `cell_count` is zero.
#[must_use]
pub fn tightest_tree_polycubes(cell_count: usize) -> TightestTrees {
    let found = best_shapes(
        cell_count,
        |contact_count, cells| {
            if contact_count + 1 != cell_count {
                return None;
            }
            let sorted_sides = sorted_box_sides(cells);
            let volume: usize = sorted_sides.iter().product();
            Some(Reverse((sorted_sides[2], volume, sorted_sides)))
        },
        |_| true,
    );
    match found {
        Some((Reverse((_, _, bounding_box)), shapes)) => TightestTrees {
            bounding_box,
            shapes,
        },
        None => TightestTrees {
            bounding_box: [0; 3],
            shapes: Vec::new(),
        },
    }
}

/// Side lengths of the smallest box holding the cells, shortest first.
fn sorted_box_sides(cells: &HashSet<Coordinates>) -> [usize; 3] {
    let mut sides = [0, 1, 2].map(|axis| {
        let coordinates = cells.iter().map(|cell| cell[axis]);
        let lowest = coordinates.clone().min().unwrap_or(0);
        let highest = coordinates.max().unwrap_or(0);
        (highest - lowest).unsigned_abs() as usize + 1
    });
    sides.sort_unstable();
    sides
}
