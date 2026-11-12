//! Box of the cubic lattice in which polycubes are grown.

pub(crate) const NEIGHBOUR_COUNT: usize = 6;

pub(crate) type Coordinates = [i32; 3];

/// Box large enough to hold any polycube grown from the origin, surrounded by
/// one layer of forbidden cells so that neighbour lookups never leave the box.
pub(crate) struct Lattice {
    /// Largest |x| and |y|, border layer included.
    horizontal_radius: usize,
    side: usize,
    layer_count: usize,
}

impl Lattice {
    pub(crate) fn new(max_cell_count: usize) -> Self {
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

    pub(crate) fn origin(&self) -> usize {
        self.cell_index(self.horizontal_radius, self.horizontal_radius, 1)
    }

    pub(crate) fn neighbour_offsets(&self) -> [isize; NEIGHBOUR_COUNT] {
        let row_stride = self.side as isize;
        let layer_stride = row_stride * row_stride;
        [1, -1, row_stride, -row_stride, layer_stride, -layer_stride]
    }

    /// Marks the border and every cell preceding the origin in (z, y, x) order.
    pub(crate) fn forbidden_cells(&self) -> Vec<bool> {
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

    /// Number of allowed neighbours of every cell of the box.
    pub(crate) fn allowed_neighbour_counts(&self, is_forbidden: &[bool]) -> Vec<u8> {
        let mut allowed_neighbour_counts = vec![0; is_forbidden.len()];
        let allowed_cells = is_forbidden
            .iter()
            .enumerate()
            .filter(|&(_, &is_cell_forbidden)| !is_cell_forbidden);
        // Allowed cells are never on the border, so their neighbours are in the box.
        for (allowed_cell, _) in allowed_cells {
            for offset in self.neighbour_offsets() {
                allowed_neighbour_counts[allowed_cell.wrapping_add_signed(offset)] += 1;
            }
        }
        allowed_neighbour_counts
    }

    /// Coordinates (x, y, z) of a cell relative to the origin.
    pub(crate) fn coordinates(&self, cell: usize) -> Coordinates {
        let radius = self.horizontal_radius as i32;
        let column = cell % self.side;
        let row = cell / self.side % self.side;
        let layer = cell / (self.side * self.side);
        [
            column as i32 - radius,
            row as i32 - radius,
            layer as i32 - 1,
        ]
    }

    /// Cell at the given coordinates, if it lies inside the box.
    pub(crate) fn cell_at(&self, [x, y, z]: Coordinates) -> Option<usize> {
        let radius = self.horizontal_radius as i32;
        let column = usize::try_from(x + radius).ok()?;
        let row = usize::try_from(y + radius).ok()?;
        let layer = usize::try_from(z + 1).ok()?;
        let is_inside = column < self.side && row < self.side && layer < self.layer_count;
        is_inside.then(|| self.cell_index(column, row, layer))
    }
}
