//! Biased growth against uniform draw, side by side, for the frontend.

use polycubes_core::Coordinates;
use polycubes_sampling::{bounding_box, grow_by_random_neighbour};
use serde::Serialize;

use crate::drawing::{CellCount, DrawError, PolycubeDrawer};

/// A polycube with its cells in the order they are shown appearing.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrownPolycube {
    pub cells: Vec<Coordinates>,
    /// Side lengths of the smallest box holding the polycube.
    pub bounding_box: [usize; 3],
}

/// Two polycubes of the same size: one grown cell by cell by picking a free
/// neighbour at random, one drawn uniformly among all fixed polycubes.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthComparison {
    pub neighbour_growth: GrownPolycube,
    pub uniform_draw: GrownPolycube,
}

/// Builds both polycubes; `choose_rank` and `choose_candidate` stand for the
/// random choices of the uniform draw and of the neighbour growth.
///
/// # Errors
///
/// Returns [`DrawError::RankOutOfRange`] if the chosen rank is not below the
/// number of polycubes.
pub fn compare_growths(
    drawer: &mut PolycubeDrawer,
    cell_count: CellCount,
    choose_rank: impl FnOnce(u64) -> u64,
    choose_candidate: impl FnMut(usize) -> usize,
) -> Result<GrowthComparison, DrawError> {
    let uniform_cells = drawer.draw(cell_count, choose_rank)?.cells;
    let grown_cells = grow_by_random_neighbour(cell_count.get(), choose_candidate);
    Ok(GrowthComparison {
        neighbour_growth: GrownPolycube::from_cells(grown_cells),
        uniform_draw: GrownPolycube::from_cells(uniform_cells),
    })
}

impl GrownPolycube {
    pub(crate) fn from_cells(cells: Vec<Coordinates>) -> Self {
        Self {
            bounding_box: bounding_box(&cells),
            cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPARED_CELL_COUNT: usize = 5;

    fn compare_with_first_choices() -> GrowthComparison {
        let cell_count = CellCount::try_from(COMPARED_CELL_COUNT).expect("five cells can be drawn");
        compare_growths(&mut PolycubeDrawer::default(), cell_count, |_| 0, |_| 0)
            .expect("rank 0 always exists")
    }

    #[test]
    fn both_polycubes_have_the_requested_size() {
        let comparison = compare_with_first_choices();

        assert_eq!(comparison.neighbour_growth.cells.len(), COMPARED_CELL_COUNT);
        assert_eq!(comparison.uniform_draw.cells.len(), COMPARED_CELL_COUNT);
    }

    #[test]
    fn bounding_boxes_hold_exactly_the_extent_of_each_polycube() {
        let comparison = compare_with_first_choices();

        for grown in [comparison.neighbour_growth, comparison.uniform_draw] {
            assert_eq!(
                grown.bounding_box,
                polycubes_sampling::bounding_box(&grown.cells)
            );
            assert!(grown.bounding_box.iter().product::<usize>() >= COMPARED_CELL_COUNT);
        }
    }

    #[test]
    fn reports_a_rank_beyond_the_last_polycube() {
        let cell_count = CellCount::try_from(COMPARED_CELL_COUNT).expect("five cells can be drawn");

        let comparison = compare_growths(
            &mut PolycubeDrawer::default(),
            cell_count,
            |polycube_count| polycube_count,
            |_| 0,
        );

        assert_eq!(
            comparison,
            Err(DrawError::RankOutOfRange {
                rank: 534,
                polycube_count: 534
            })
        );
    }
}
