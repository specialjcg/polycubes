//! The closed formula against a plain visit, on one polycube.

use super::{CLOSED_FORM_CELL_COUNT, EnumerationCursor};
use crate::lattice::Coordinates;

/// Every way to add one to four cells to a polycube met by the
/// enumeration, counted twice: by the closed formula and by visiting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontierExtensions {
    /// Cells of the polycube, in placement order.
    pub placed: Vec<Coordinates>,
    /// Cells waiting to extend the polycube: the frontier the formula reads.
    pub candidates: Vec<Coordinates>,
    /// Formula counts: row `added - 1`, column number of added cells that
    /// are not candidates.
    pub formula_counts: [[u64; CLOSED_FORM_CELL_COUNT]; CLOSED_FORM_CELL_COUNT],
    /// Cells added by each extension the visit finds, in placement order.
    pub extensions: Vec<Vec<Coordinates>>,
}

/// Walks down the enumeration tree to a polycube of `placed_cell_count`
/// cells, then counts its extensions both ways. `choose_candidate` receives
/// the number of children of a node and returns the index of the one to
/// follow.
///
/// # Panics
///
/// Panics if `placed_cell_count` is zero or if `choose_candidate` returns an
/// index that is not below the count it received.
#[must_use]
pub fn frontier_extensions(
    placed_cell_count: usize,
    mut choose_candidate: impl FnMut(usize) -> usize,
) -> FrontierExtensions {
    assert!(placed_cell_count > 0, "a polycube has at least one cell");
    let mut cursor = EnumerationCursor::new(placed_cell_count + CLOSED_FORM_CELL_COUNT);
    for _ in 0..placed_cell_count {
        cursor.descend(choose_candidate(cursor.candidate_count()));
    }
    let mut extensions = Vec::new();
    visit_extensions(&mut cursor, placed_cell_count, &mut extensions);
    FrontierExtensions {
        placed: cursor.cells(),
        candidates: cursor.candidates(),
        formula_counts: cursor.closed_form_extension_counts(),
        extensions,
    }
}

fn visit_extensions(
    cursor: &mut EnumerationCursor,
    placed_cell_count: usize,
    extensions: &mut Vec<Vec<Coordinates>>,
) {
    for candidate_index in 0..cursor.candidate_count() {
        cursor.descend(candidate_index);
        extensions.push(cursor.cells().split_off(placed_cell_count));
        visit_extensions(cursor, placed_cell_count, extensions);
        cursor.ascend();
    }
}
