//! The polycubes of a size with the most loops, for the frontend.

use polycubes_sampling::most_looped_polycubes;
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

/// The shapes of a size that hold the most loops, each standing for all the
/// fixed polycubes that differ from it by a rotation.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MostLoopedPolycubes {
    pub loop_count: usize,
    /// Faces on the outside of each of these polycubes, the fewest of their
    /// size.
    pub exposed_face_count: usize,
    pub shapes: Vec<RemarkableShape>,
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
pub fn find_most_looped(cell_count: usize) -> Result<MostLoopedPolycubes, RemarkableError> {
    if cell_count == 0 || cell_count > MAX_REMARKABLE_CELL_COUNT {
        return Err(RemarkableError::CellCountOutOfRange(cell_count));
    }
    let most_looped = most_looped_polycubes(cell_count);
    Ok(MostLoopedPolycubes {
        loop_count: most_looped.loop_count,
        // Without loop a polycube shows `4n + 2` faces; each loop hides two.
        exposed_face_count: 4 * cell_count + 2 - 2 * most_looped.loop_count,
        shapes: most_looped
            .shapes
            .into_iter()
            .map(|shape| RemarkableShape {
                polycube: GrownPolycube::from_cells(shape.cells),
                orientation_count: shape.orientation_count,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_cells_give_the_square() {
        let most_looped = find_most_looped(4).expect("four cells can be searched");

        assert_eq!(most_looped.loop_count, 1);
        assert_eq!(most_looped.exposed_face_count, 16);
        assert_eq!(most_looped.shapes.len(), 1);
        assert_eq!(most_looped.shapes[0].orientation_count, 3);
        assert_eq!(most_looped.shapes[0].polycube.bounding_box, [1, 2, 2]);
    }

    #[test]
    fn eight_cells_give_the_cube() {
        let most_looped = find_most_looped(8).expect("eight cells can be searched");

        assert_eq!(most_looped.loop_count, 5);
        assert_eq!(most_looped.exposed_face_count, 24);
        assert_eq!(most_looped.shapes[0].polycube.bounding_box, [2, 2, 2]);
    }

    #[test]
    fn refuses_sizes_outside_the_searched_range() {
        for cell_count in [0, MAX_REMARKABLE_CELL_COUNT + 1] {
            assert_eq!(
                find_most_looped(cell_count),
                Err(RemarkableError::CellCountOutOfRange(cell_count))
            );
        }
    }
}
