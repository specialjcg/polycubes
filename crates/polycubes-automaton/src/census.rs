//! The fates of all fixed polycubes of a size, taken as seeds.

use polycubes_core::{Coordinates, EnumerationCursor};

use crate::evolution::{Fate, Limits, evolve};
use crate::rule::Rule;

/// Fixed polycubes of a size per fate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Census {
    pub dying: u64,
    pub still: u64,
    pub cycling: u64,
    pub travelling: u64,
    pub unsettled: u64,
}

/// Runs the automaton from every fixed polycube of `cell_count` cells.
///
/// # Panics
///
/// Panics if `cell_count` is zero.
#[must_use]
pub fn census(cell_count: usize, rule: &Rule, limits: Limits) -> Census {
    let mut found = Census::default();
    let mut cursor = EnumerationCursor::new(cell_count);
    visit_seeds(&mut cursor, cell_count, 0, &mut |seed| {
        let fate_count = match evolve(seed, rule, limits).fate {
            Fate::Dies { .. } => &mut found.dying,
            Fate::Still { .. } => &mut found.still,
            Fate::Cycles { .. } => &mut found.cycling,
            Fate::Travels { .. } => &mut found.travelling,
            Fate::Unsettled => &mut found.unsettled,
        };
        *fate_count += 1;
    });
    found
}

/// Hands every fixed polycube of `cell_count` cells below the cursor to
/// `on_seed`; the cursor stands on a polycube of `placed_cell_count` cells.
fn visit_seeds(
    cursor: &mut EnumerationCursor,
    cell_count: usize,
    placed_cell_count: usize,
    on_seed: &mut impl FnMut(&[Coordinates]),
) {
    if placed_cell_count == cell_count {
        on_seed(&cursor.cells());
        return;
    }
    for candidate_index in 0..cursor.candidate_count() {
        cursor.descend(candidate_index);
        visit_seeds(cursor, cell_count, placed_cell_count + 1, on_seed);
        cursor.ascend();
    }
}
