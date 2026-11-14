//! Diffusion-limited aggregation: polycubes wander in from afar and stick
//! to the aggregate where they first touch it. Tips catch the wanderers
//! before the hollows do, so the aggregate branches like frost or coral.

use std::collections::HashSet;

use polycubes_core::Coordinates;

use crate::budding::orientations;
use crate::colony::face_neighbours;

/// Free cells kept between the aggregate and the box wanderers start from.
const LAUNCH_MARGIN: i32 = 2;

/// Times a wanderer may start again after straying too far, before it is
/// given up: a walk that never turns would otherwise never end.
const LAUNCHES_PER_WANDERER: usize = 100;

const DIRECTION_COUNT: usize = 6;

/// Grows an aggregate from `seed` and returns its polycubes in the order
/// they stuck, the seed first, each as `seed.len()` cells.
///
/// Each wanderer is the seed turned at random, never mirrored. It starts
/// on a face of a box around the aggregate, then moves one cell at a time
/// until one of its cells touches the aggregate, where it stays. One that
/// strays twice as far as the box starts again. `choose` picks the turn,
/// the starting place and each step among a number of choices, and must
/// answer an index below that number.
///
/// Fewer than `polycube_count` polycubes come back when wanderers get
/// lost, which a `choose` that is not random enough brings about.
#[must_use]
pub fn aggregate(
    seed: &[Coordinates],
    polycube_count: usize,
    mut choose: impl FnMut(usize) -> usize,
) -> Vec<Vec<Coordinates>> {
    let mut polycubes = vec![seed.to_vec()];
    if seed.is_empty() {
        return polycubes;
    }
    let orientations = orientations(seed);
    let mut aggregate = Aggregate::of(seed);
    for _ in 1..polycube_count {
        let orientation = &orientations[choose(orientations.len())];
        if let Some(stuck) = aggregate.catch_wanderer(orientation, &mut choose) {
            aggregate.hold(&stuck);
            polycubes.push(stuck);
        }
    }
    polycubes
}

struct Aggregate {
    occupied: HashSet<Coordinates>,
    /// Farthest coordinate of a cell from the origin, along any axis.
    reach: i32,
}

impl Aggregate {
    fn of(seed: &[Coordinates]) -> Self {
        let mut aggregate = Self {
            occupied: HashSet::new(),
            reach: 0,
        };
        aggregate.hold(seed);
        aggregate
    }

    fn hold(&mut self, polycube: &[Coordinates]) {
        for &cell in polycube {
            self.occupied.insert(cell);
            self.reach = self.reach.max(farthest_coordinate(cell));
        }
    }

    /// Walks `shape` in from the launch box until it touches the aggregate;
    /// `None` when it got lost at every launch.
    fn catch_wanderer(
        &self,
        shape: &[Coordinates],
        choose: &mut dyn FnMut(usize) -> usize,
    ) -> Option<Vec<Coordinates>> {
        let shape_reach = shape.iter().copied().map(farthest_coordinate).max()?;
        let launch_distance = self.reach + shape_reach + LAUNCH_MARGIN;
        for _ in 0..LAUNCHES_PER_WANDERER {
            let mut position = launch_position(launch_distance, choose);
            while farthest_coordinate(position) <= 2 * launch_distance {
                let wanderer = moved_by(shape, position);
                if self.is_touched_by(&wanderer) {
                    return Some(wanderer);
                }
                position = stepped(position, choose(DIRECTION_COUNT));
            }
        }
        None
    }

    fn is_touched_by(&self, wanderer: &[Coordinates]) -> bool {
        wanderer
            .iter()
            .copied()
            .flat_map(face_neighbours)
            .any(|neighbour| self.occupied.contains(&neighbour))
    }
}

fn farthest_coordinate(cell: Coordinates) -> i32 {
    cell.iter()
        .map(|coordinate| coordinate.abs())
        .max()
        .unwrap_or(0)
}

/// A place on one of the six faces of the cube that reaches `distance`
/// from the origin along each axis.
fn launch_position(distance: i32, choose: &mut dyn FnMut(usize) -> usize) -> Coordinates {
    let places_along_a_side = usize::try_from(2 * distance + 1).unwrap_or(1);
    let mut place_on_side = || index_as_coordinate(choose(places_along_a_side)) - distance;
    let (first, second) = (place_on_side(), place_on_side());
    match choose(DIRECTION_COUNT) {
        0 => [distance, first, second],
        1 => [-distance, first, second],
        2 => [first, distance, second],
        3 => [first, -distance, second],
        4 => [first, second, distance],
        _ => [first, second, -distance],
    }
}

fn stepped([x, y, z]: Coordinates, direction: usize) -> Coordinates {
    match direction {
        0 => [x + 1, y, z],
        1 => [x - 1, y, z],
        2 => [x, y + 1, z],
        3 => [x, y - 1, z],
        4 => [x, y, z + 1],
        _ => [x, y, z - 1],
    }
}

fn moved_by(shape: &[Coordinates], shift: Coordinates) -> Vec<Coordinates> {
    shape
        .iter()
        .map(|cell| [0, 1, 2].map(|axis| cell[axis] + shift[axis]))
        .collect()
}

/// An index along a side of the launch box always fits a coordinate.
fn index_as_coordinate(index: usize) -> i32 {
    i32::try_from(index).unwrap_or(0)
}
