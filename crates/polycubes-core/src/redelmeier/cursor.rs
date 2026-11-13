//! Step-by-step walk of the enumeration tree.

use super::Enumerator;
use crate::lattice::{Coordinates, Lattice};

/// A position in the enumeration tree of the fixed polycubes of at most
/// `max_cell_count` cells.
///
/// Each node is a polycube; its children extend it with one of its
/// candidate cells. Every fixed polycube is the node of exactly one path
/// from the root, which holds no cell.
pub struct EnumerationCursor {
    enumerator: Enumerator<'static>,
    lattice: Lattice,
    max_cell_count: usize,
    /// Start of the candidates of the current node in the untried queue.
    first_untried: usize,
    placed_cells: Vec<usize>,
    /// For each placed cell, the candidates of the node it was chosen from.
    parent_candidates: Vec<std::ops::Range<usize>>,
}

impl EnumerationCursor {
    /// Cursor at the root of the tree.
    ///
    /// # Panics
    ///
    /// Panics if `max_cell_count` is zero.
    #[must_use]
    pub fn new(max_cell_count: usize) -> Self {
        assert!(max_cell_count > 0, "a polycube has at least one cell");
        Self {
            enumerator: Enumerator::new(max_cell_count, None),
            lattice: Lattice::new(max_cell_count),
            max_cell_count,
            first_untried: 0,
            placed_cells: Vec::new(),
            parent_candidates: Vec::new(),
        }
    }

    /// Number of children of the current node.
    #[must_use]
    pub fn candidate_count(&self) -> usize {
        if self.placed_cells.len() == self.max_cell_count {
            return 0;
        }
        self.enumerator.untried_cells.len() - self.first_untried
    }

    /// Moves to the child that adds the candidate of the given index.
    ///
    /// # Panics
    ///
    /// Panics if `candidate_index` is not below [`Self::candidate_count`].
    pub fn descend(&mut self, candidate_index: usize) {
        assert!(
            candidate_index < self.candidate_count(),
            "candidate index out of range"
        );
        let untried_end = self.enumerator.untried_cells.len();
        let chosen_untried = self.first_untried + candidate_index;
        // Earlier candidates are closed too: they belong to earlier subtrees.
        for untried_index in self.first_untried..=chosen_untried {
            self.enumerator
                .close(self.enumerator.untried_cells[untried_index]);
        }
        let chosen_cell = self.enumerator.untried_cells[chosen_untried];
        self.enumerator.queue_free_neighbours(chosen_cell);
        self.placed_cells.push(chosen_cell);
        self.parent_candidates.push(self.first_untried..untried_end);
        self.first_untried = chosen_untried + 1;
    }

    /// Moves back to the parent node; does nothing at the root.
    pub fn ascend(&mut self) {
        let Some(parent_candidates) = self.parent_candidates.pop() else {
            return;
        };
        self.placed_cells.pop();
        self.enumerator
            .free_cells_queued_after(parent_candidates.end);
        for untried_index in (parent_candidates.start..self.first_untried).rev() {
            self.enumerator
                .reopen(self.enumerator.untried_cells[untried_index]);
        }
        self.first_untried = parent_candidates.start;
    }

    /// Number of polycubes of exactly `max_cell_count` cells in the subtree
    /// of the current node, itself included.
    pub fn completion_count(&mut self) -> u64 {
        let placed_cell_count = self.placed_cells.len();
        if placed_cell_count == self.max_cell_count {
            return 1;
        }
        self.enumerator.counts.fill(0);
        self.enumerator
            .extend(self.first_untried, placed_cell_count);
        self.enumerator.counts[self.max_cell_count - 1]
    }

    /// Cells of the polycube of the current node, in placement order.
    #[must_use]
    pub fn cells(&self) -> Vec<Coordinates> {
        self.placed_cells
            .iter()
            .map(|&cell| self.lattice.coordinates(cell))
            .collect()
    }
}
