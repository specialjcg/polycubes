//! Exceptional polycubes of a size, for the frontend.

use polycubes_sampling::{
    Shape, most_looped_polycubes, most_looped_square_free_polycubes, tightest_tree_polycubes,
};
use serde::Serialize;

use crate::growth::GrownPolycube;

/// Largest size searched: every fixed polycube is visited, and beyond this
/// size the visit takes seconds.
pub const MAX_REMARKABLE_CELL_COUNT: usize = 9;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RemarkableError {
    #[error("cell count must be between 1 and {MAX_REMARKABLE_CELL_COUNT}, got {0}")]
    CellCountOutOfRange(usize),
}

/// Shapes shown per family: each needs its own drawing surface, and a page
/// may only hold a few.
pub const SHOWN_PER_FAMILY: usize = 3;

/// Three families of exceptional polycubes of one size.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemarkablePolycubes {
    /// Those with the most loops.
    pub most_looped: ShapeFamily,
    /// Those with the most loops among the polycubes where no four cubes
    /// form a 2 × 2 square; absent when no such polycube has a loop.
    pub square_free_loops: Option<ShapeFamily>,
    /// Those without loop that fit in the smallest box.
    pub tightest_trees: ShapeFamily,
}

/// Shapes sharing an extreme, each standing for all the fixed polycubes
/// that differ from it by a rotation.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeFamily {
    pub loop_count: usize,
    /// Faces on the outside of each of these polycubes.
    pub exposed_face_count: usize,
    pub shape_count: usize,
    pub fixed_count: u64,
    /// The most symmetric shapes, at most [`SHOWN_PER_FAMILY`].
    pub shown: Vec<RemarkableShape>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemarkableShape {
    pub polycube: GrownPolycube,
    /// Fixed polycubes that are this shape turned around.
    pub orientation_count: u64,
}

/// Searches every fixed polycube of `cell_count` cells.
///
/// # Errors
///
/// Returns [`RemarkableError::CellCountOutOfRange`] if `cell_count` is zero
/// or exceeds [`MAX_REMARKABLE_CELL_COUNT`].
pub fn find_remarkable(cell_count: usize) -> Result<RemarkablePolycubes, RemarkableError> {
    if cell_count == 0 || cell_count > MAX_REMARKABLE_CELL_COUNT {
        return Err(RemarkableError::CellCountOutOfRange(cell_count));
    }
    let most_looped = most_looped_polycubes(cell_count);
    let square_free_loops = most_looped_square_free_polycubes(cell_count);
    let tightest_trees = tightest_tree_polycubes(cell_count);
    Ok(RemarkablePolycubes {
        most_looped: ShapeFamily::of(cell_count, most_looped.loop_count, most_looped.shapes),
        square_free_loops: square_free_loops
            .map(|found| ShapeFamily::of(cell_count, found.loop_count, found.shapes)),
        tightest_trees: ShapeFamily::of(cell_count, 0, tightest_trees.shapes),
    })
}

impl ShapeFamily {
    fn of(cell_count: usize, loop_count: usize, mut shapes: Vec<Shape>) -> Self {
        // The fewer orientations, the more symmetric the shape.
        shapes.sort_by_key(|shape| shape.orientation_count);
        Self {
            loop_count,
            // Without loop a polycube shows `4n + 2` faces; each loop hides two.
            exposed_face_count: 4 * cell_count + 2 - 2 * loop_count,
            shape_count: shapes.len(),
            fixed_count: shapes.iter().map(|shape| shape.orientation_count).sum(),
            shown: shapes
                .into_iter()
                .take(SHOWN_PER_FAMILY)
                .map(|shape| RemarkableShape {
                    polycube: GrownPolycube::from_cells(shape.cells),
                    orientation_count: shape.orientation_count,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_cells_give_the_square() {
        let remarkable = find_remarkable(4).expect("four cells can be searched");

        let most_looped = &remarkable.most_looped;
        assert_eq!(most_looped.loop_count, 1);
        assert_eq!(most_looped.exposed_face_count, 16);
        assert_eq!(most_looped.shape_count, 1);
        assert_eq!(most_looped.fixed_count, 3);
        assert_eq!(most_looped.shown[0].polycube.bounding_box, [1, 2, 2]);
    }

    #[test]
    fn eight_cells_give_the_cube() {
        let remarkable = find_remarkable(8).expect("eight cells can be searched");

        assert_eq!(remarkable.most_looped.loop_count, 5);
        assert_eq!(remarkable.most_looped.exposed_face_count, 24);
        assert_eq!(
            remarkable.most_looped.shown[0].polycube.bounding_box,
            [2, 2, 2]
        );
    }

    #[test]
    fn a_loop_without_square_needs_six_cells() {
        let five_cells = find_remarkable(5).expect("five cells can be searched");
        let six_cells = find_remarkable(6).expect("six cells can be searched");

        assert_eq!(five_cells.square_free_loops, None);
        let loops = six_cells
            .square_free_loops
            .expect("six cells go around a corner");
        assert_eq!(
            (loops.loop_count, loops.shape_count, loops.fixed_count),
            (1, 1, 4)
        );
    }

    #[test]
    fn the_flat_ring_comes_first_among_the_loops_of_eight_cells() {
        let remarkable = find_remarkable(8).expect("eight cells can be searched");

        let loops = remarkable
            .square_free_loops
            .expect("eight cells make a ring");
        assert_eq!(loops.shape_count, 42);
        assert_eq!(loops.shown.len(), SHOWN_PER_FAMILY);
        assert_eq!(loops.shown[0].orientation_count, 3);
        assert_eq!(loops.shown[0].polycube.bounding_box, [1, 3, 3]);
    }

    #[test]
    fn tightest_trees_of_four_cells_fill_half_a_cube() {
        let remarkable = find_remarkable(4).expect("four cells can be searched");

        let trees = &remarkable.tightest_trees;
        assert_eq!((trees.loop_count, trees.exposed_face_count), (0, 18));
        assert_eq!((trees.shape_count, trees.fixed_count), (3, 32));
        assert_eq!(trees.shown[0].polycube.bounding_box, [2, 2, 2]);
    }

    #[test]
    fn refuses_sizes_outside_the_searched_range() {
        for cell_count in [0, MAX_REMARKABLE_CELL_COUNT + 1] {
            assert_eq!(
                find_remarkable(cell_count),
                Err(RemarkableError::CellCountOutOfRange(cell_count))
            );
        }
    }
}
