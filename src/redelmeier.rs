//! Redelmeier's algorithm, generalised to the cubic lattice.
//!
//! Every fixed polycube is generated exactly once by growing it from its
//! lexicographically smallest cell, placed at the origin. Cells that precede
//! the origin in (z, y, x) order are forbidden, which removes translations.

const NEIGHBOUR_COUNT: usize = 6;
/// Number of final cells counted in closed form instead of being enumerated.
const CLOSED_FORM_CELL_COUNT: usize = 3;

/// Counts fixed polycubes (distinct up to translation only) of every size
/// from 1 to `max_cell_count`.
///
/// Element `size - 1` of the result is the number of fixed polycubes made of
/// `size` cubes (OEIS A001931).
#[must_use]
pub fn count_fixed_polycubes(max_cell_count: usize) -> Vec<u64> {
    if max_cell_count == 0 {
        return Vec::new();
    }
    let mut enumerator = Enumerator::new(max_cell_count);
    enumerator.extend(0, 0);
    enumerator.counts
}

struct Enumerator {
    /// Cells already in the polycube, already queued, or forbidden.
    is_reached: Vec<bool>,
    /// Append-only queue of candidate cells; each recursion level owns a suffix.
    untried_cells: Vec<usize>,
    /// Scratch board: untried neighbours seen so far for each free cell.
    untried_neighbour_counts: Vec<u8>,
    neighbour_offsets: [isize; NEIGHBOUR_COUNT],
    counts: Vec<u64>,
    max_cell_count: usize,
}

impl Enumerator {
    fn new(max_cell_count: usize) -> Self {
        let lattice = Lattice::new(max_cell_count);
        let origin = lattice.origin();
        let mut is_reached = lattice.forbidden_cells();
        is_reached[origin] = true;
        Self {
            untried_neighbour_counts: vec![0; is_reached.len()],
            is_reached,
            untried_cells: vec![origin],
            neighbour_offsets: lattice.neighbour_offsets(),
            counts: vec![0; max_cell_count],
            max_cell_count,
        }
    }

    /// Extends a polycube of `placed_cell_count` cells with every untried
    /// cell from `first_untried` onwards.
    fn extend(&mut self, first_untried: usize, placed_cell_count: usize) {
        if placed_cell_count + CLOSED_FORM_CELL_COUNT == self.max_cell_count {
            self.count_final_extensions(first_untried, placed_cell_count);
            return;
        }
        let untried_end = self.untried_cells.len();
        let cell_count = placed_cell_count + 1;
        for untried_index in first_untried..untried_end {
            let cell = self.untried_cells[untried_index];
            self.counts[cell_count - 1] += 1;
            // Only reachable when the maximum size is below the closed-form depth.
            if cell_count == self.max_cell_count {
                continue;
            }
            self.queue_unreached_neighbours(cell);
            self.extend(untried_index + 1, cell_count);
            self.forget_cells_queued_after(untried_end);
        }
    }

    /// Counts, without visiting them, every way to add one, two or three
    /// cells to the current polycube.
    ///
    /// A set of added cells is valid when each cell is untried, or free and
    /// linked to an untried cell of the set through free cells of the set.
    /// With `U` the untried cells, `d(u)` the free neighbours of `u`, `m(f)`
    /// the untried neighbours of a free cell `f` and `e(f)` its free
    /// neighbours:
    ///
    /// - one cell: `|U|`
    /// - two cells: `C(|U|, 2) + sum d(u)`
    /// - three cells: `C(|U|, 3)` (all untried)
    ///   `+ (|U| - 1) * sum d(u) - sum C(m(f), 2)` (two untried, one free)
    ///   `+ sum C(d(u), 2) + sum m(f) * e(f)` (one untried, two free; the
    ///   lattice is bipartite, so a free cell two steps away from `u` is
    ///   never also a neighbour of `u`).
    fn count_final_extensions(&mut self, first_untried: usize, placed_cell_count: usize) {
        let untried_end = self.untried_cells.len();
        let untried_count = (untried_end - first_untried) as u64;
        let mut free_neighbour_total = 0;
        let mut free_neighbour_pair_total = 0;
        let mut shared_free_neighbour_pair_total = 0;
        let mut two_step_path_total = 0;
        for untried_index in first_untried..untried_end {
            let untried_cell = self.untried_cells[untried_index];
            let mut free_neighbour_count = 0;
            for offset in self.neighbour_offsets {
                let free_cell = untried_cell.wrapping_add_signed(offset);
                if self.is_reached[free_cell] {
                    continue;
                }
                free_neighbour_count += 1;
                shared_free_neighbour_pair_total +=
                    u64::from(self.untried_neighbour_counts[free_cell]);
                self.untried_neighbour_counts[free_cell] += 1;
                two_step_path_total += self.free_neighbour_count(free_cell);
            }
            free_neighbour_total += free_neighbour_count;
            free_neighbour_pair_total += choose_two(free_neighbour_count);
        }
        self.clear_untried_neighbour_counts(first_untried);

        let two_untried_one_free = untried_count.saturating_sub(1) * free_neighbour_total
            - shared_free_neighbour_pair_total;
        let one_untried_two_free = free_neighbour_pair_total + two_step_path_total;
        self.counts[placed_cell_count] += untried_count;
        self.counts[placed_cell_count + 1] += choose_two(untried_count) + free_neighbour_total;
        self.counts[placed_cell_count + 2] +=
            choose_three(untried_count) + two_untried_one_free + one_untried_two_free;
    }

    fn free_neighbour_count(&self, cell: usize) -> u64 {
        self.neighbour_offsets
            .iter()
            .filter(|&&offset| !self.is_reached[cell.wrapping_add_signed(offset)])
            .count() as u64
    }

    fn clear_untried_neighbour_counts(&mut self, first_untried: usize) {
        for &untried_cell in &self.untried_cells[first_untried..] {
            for offset in self.neighbour_offsets {
                self.untried_neighbour_counts[untried_cell.wrapping_add_signed(offset)] = 0;
            }
        }
    }

    fn queue_unreached_neighbours(&mut self, cell: usize) {
        for offset in self.neighbour_offsets {
            let neighbour = cell.wrapping_add_signed(offset);
            if !self.is_reached[neighbour] {
                self.is_reached[neighbour] = true;
                self.untried_cells.push(neighbour);
            }
        }
    }

    fn forget_cells_queued_after(&mut self, untried_end: usize) {
        for forgotten_cell in self.untried_cells.drain(untried_end..) {
            self.is_reached[forgotten_cell] = false;
        }
    }
}

fn choose_two(item_count: u64) -> u64 {
    item_count * item_count.saturating_sub(1) / 2
}

fn choose_three(item_count: u64) -> u64 {
    choose_two(item_count) * item_count.saturating_sub(2) / 3
}

/// Box of the cubic lattice large enough to hold any polycube grown from the
/// origin, surrounded by one layer of forbidden cells so that neighbour
/// lookups never leave the box.
struct Lattice {
    /// Largest |x| and |y|, border layer included.
    horizontal_radius: usize,
    side: usize,
    layer_count: usize,
}

impl Lattice {
    fn new(max_cell_count: usize) -> Self {
        let horizontal_radius = max_cell_count;
        Self {
            horizontal_radius,
            side: 2 * horizontal_radius + 1,
            // z ranges over -1 (forbidden) ..= max_cell_count (border).
            layer_count: max_cell_count + 2,
        }
    }

    fn cell_index(&self, column: usize, row: usize, layer: usize) -> usize {
        (layer * self.side + row) * self.side + column
    }

    fn origin(&self) -> usize {
        self.cell_index(self.horizontal_radius, self.horizontal_radius, 1)
    }

    fn neighbour_offsets(&self) -> [isize; NEIGHBOUR_COUNT] {
        let row_stride = self.side as isize;
        let layer_stride = row_stride * row_stride;
        [1, -1, row_stride, -row_stride, layer_stride, -layer_stride]
    }

    /// Marks the border and every cell preceding the origin in (z, y, x) order.
    fn forbidden_cells(&self) -> Vec<bool> {
        let origin = self.origin();
        let last = self.side - 1;
        let top_layer = self.layer_count - 1;
        let mut is_forbidden = vec![false; self.side * self.side * self.layer_count];
        for layer in 0..self.layer_count {
            for row in 0..self.side {
                for column in 0..self.side {
                    let cell = self.cell_index(column, row, layer);
                    let is_border = layer == top_layer
                        || row == 0
                        || row == last
                        || column == 0
                        || column == last;
                    is_forbidden[cell] = is_border || cell < origin;
                }
            }
        }
        is_forbidden
    }
}
