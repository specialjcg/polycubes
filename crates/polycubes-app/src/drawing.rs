//! Uniform drawing of polycubes for the frontend.

use std::collections::HashMap;

use polycubes_core::Coordinates;
use polycubes_sampling::RankedPolycubes;
use serde::Serialize;

/// Largest size drawn interactively: beyond it a draw takes over a second.
pub const MAX_INTERACTIVE_CELL_COUNT: usize = 12;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DrawError {
    #[error("cell count must be between 1 and {MAX_INTERACTIVE_CELL_COUNT}, got {0}")]
    CellCountOutOfRange(usize),
    #[error("rank {rank} is not below the number of polycubes {polycube_count}")]
    RankOutOfRange { rank: u64, polycube_count: u64 },
}

/// A polycube size that can be drawn interactively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellCount(usize);

impl CellCount {
    #[must_use]
    pub fn get(self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for CellCount {
    type Error = DrawError;

    fn try_from(cell_count: usize) -> Result<Self, Self::Error> {
        if (1..=MAX_INTERACTIVE_CELL_COUNT).contains(&cell_count) {
            Ok(Self(cell_count))
        } else {
            Err(DrawError::CellCountOutOfRange(cell_count))
        }
    }
}

/// A drawn polycube, as sent to the frontend.
///
/// Counts are sent as text: they may exceed what a JavaScript number holds.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawnPolycube {
    pub rank: String,
    pub polycube_count: String,
    pub cells: Vec<Coordinates>,
}

/// Draws polycubes, keeping what was computed for each size.
#[derive(Default)]
pub struct PolycubeDrawer {
    ranked_by_cell_count: HashMap<CellCount, RankedPolycubes>,
}

impl PolycubeDrawer {
    /// Draws the polycube whose rank `choose_rank` picks below the number of
    /// polycubes it is given.
    ///
    /// # Errors
    ///
    /// Returns [`DrawError::RankOutOfRange`] if the chosen rank is not below
    /// that number.
    pub fn draw(
        &mut self,
        cell_count: CellCount,
        choose_rank: impl FnOnce(u64) -> u64,
    ) -> Result<DrawnPolycube, DrawError> {
        let ranked_polycubes = self
            .ranked_by_cell_count
            .entry(cell_count)
            .or_insert_with(|| RankedPolycubes::new(cell_count.0));
        let polycube_count = ranked_polycubes.count();
        let rank = choose_rank(polycube_count);
        let cells = ranked_polycubes
            .polycube_at(rank)
            .ok_or(DrawError::RankOutOfRange {
                rank,
                polycube_count,
            })?;
        Ok(DrawnPolycube {
            rank: rank.to_string(),
            polycube_count: polycube_count.to_string(),
            cells,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tricube_count() -> CellCount {
        CellCount::try_from(3).expect("three cells can be drawn")
    }

    #[test]
    fn accepts_every_interactive_cell_count() {
        for cell_count in 1..=MAX_INTERACTIVE_CELL_COUNT {
            assert!(CellCount::try_from(cell_count).is_ok());
        }
    }

    #[test]
    fn rejects_an_empty_polycube() {
        assert_eq!(
            CellCount::try_from(0),
            Err(DrawError::CellCountOutOfRange(0))
        );
    }

    #[test]
    fn rejects_a_cell_count_too_slow_to_draw() {
        let too_large = MAX_INTERACTIVE_CELL_COUNT + 1;

        assert_eq!(
            CellCount::try_from(too_large),
            Err(DrawError::CellCountOutOfRange(too_large))
        );
    }

    #[test]
    fn offers_the_number_of_polycubes_to_the_rank_chooser() {
        let mut offered_count = 0;

        let _ = PolycubeDrawer::default().draw(tricube_count(), |polycube_count| {
            offered_count = polycube_count;
            0
        });

        assert_eq!(offered_count, 15);
    }

    #[test]
    fn draws_the_polycube_of_the_chosen_rank() {
        let drawn = PolycubeDrawer::default().draw(tricube_count(), |_| 14);

        let drawn = drawn.expect("rank 14 is below 15");
        assert_eq!(drawn.rank, "14");
        assert_eq!(drawn.polycube_count, "15");
        assert_eq!(drawn.cells.len(), 3);
    }

    #[test]
    fn reports_a_rank_beyond_the_last_polycube() {
        let drawn =
            PolycubeDrawer::default().draw(tricube_count(), |polycube_count| polycube_count);

        assert_eq!(
            drawn,
            Err(DrawError::RankOutOfRange {
                rank: 15,
                polycube_count: 15
            })
        );
    }
}
