use polycubes_core::{Coordinates, EnumerationCursor};

/// The fixed polycubes of a given size, addressable by rank.
///
/// The rank of a polycube is its position among the polycubes of that size
/// in the order of the enumeration tree. Drawing a rank uniformly in
/// `0..count()` therefore draws a fixed polycube uniformly.
pub struct RankedPolycubes {
    cursor: EnumerationCursor,
    cell_count: usize,
    count: u64,
}

impl RankedPolycubes {
    /// # Panics
    ///
    /// Panics if `cell_count` is zero.
    #[must_use]
    pub fn new(cell_count: usize) -> Self {
        let mut cursor = EnumerationCursor::new(cell_count);
        let count = cursor.completion_count();
        Self {
            cursor,
            cell_count,
            count,
        }
    }

    /// Number of fixed polycubes of the requested size.
    #[must_use]
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Cells of the polycube of the given rank, its smallest cell in
    /// (z, y, x) order at the origin; `None` when the rank is out of range.
    pub fn polycube_at(&mut self, rank: u64) -> Option<Vec<Coordinates>> {
        if rank >= self.count {
            return None;
        }
        let mut rank_in_subtree = rank;
        for _ in 0..self.cell_count {
            let candidate_index = self.candidate_holding(&mut rank_in_subtree);
            self.cursor.descend(candidate_index);
        }
        let cells = self.cursor.cells();
        for _ in 0..self.cell_count {
            self.cursor.ascend();
        }
        Some(cells)
    }

    /// Finds the child whose subtree holds the rank, and makes the rank
    /// relative to that subtree.
    fn candidate_holding(&mut self, rank_in_subtree: &mut u64) -> usize {
        for candidate_index in 0..self.cursor.candidate_count() {
            self.cursor.descend(candidate_index);
            let completion_count = self.cursor.completion_count();
            self.cursor.ascend();
            if *rank_in_subtree < completion_count {
                return candidate_index;
            }
            *rank_in_subtree -= completion_count;
        }
        unreachable!("the rank is below the completion count of the current node")
    }
}
