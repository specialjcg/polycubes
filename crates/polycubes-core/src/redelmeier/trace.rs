//! Record of the enumeration, node by node, for display.

use super::EnumerationCursor;
use crate::lattice::Coordinates;

/// What the enumeration holds at one node of its tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumerationStep {
    /// Cells of the polycube of the node, in placement order.
    pub placed: Vec<Coordinates>,
    /// Cells waiting to extend the polycube, in the order they are tried.
    pub candidates: Vec<Coordinates>,
    /// Cells already tried higher in the tree, forbidden below this node so
    /// that no polycube is reached twice.
    pub discarded: Vec<Coordinates>,
}

/// Nodes of the enumeration tree of the fixed polycubes of at most
/// `max_cell_count` cells, in the order the enumeration visits them. The
/// empty root is left out, so each step is one polycube, counted once.
///
/// # Panics
///
/// Panics if `max_cell_count` is zero.
#[must_use]
pub fn trace_enumeration(max_cell_count: usize) -> Vec<EnumerationStep> {
    let mut cursor = EnumerationCursor::new(max_cell_count);
    let mut steps = Vec::new();
    record_subtree(&mut cursor, &mut steps);
    steps
}

fn record_subtree(cursor: &mut EnumerationCursor, steps: &mut Vec<EnumerationStep>) {
    for candidate_index in 0..cursor.candidate_count() {
        cursor.descend(candidate_index);
        steps.push(EnumerationStep {
            placed: cursor.cells(),
            candidates: cursor.candidates(),
            discarded: cursor.discarded_cells(),
        });
        record_subtree(cursor, steps);
        cursor.ascend();
    }
}
