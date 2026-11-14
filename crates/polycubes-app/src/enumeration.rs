//! The enumeration of fixed polycubes, node by node, for the frontend.

use polycubes_core::{Coordinates, EnumerationStep};
use serde::Serialize;

/// Largest size traced: beyond it the trace has thousands of nodes, too
/// many to watch one by one.
pub const MAX_TRACED_CELL_COUNT: usize = 5;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TraceError {
    #[error("cell count must be between 1 and {MAX_TRACED_CELL_COUNT}, got {0}")]
    CellCountOutOfRange(usize),
}

/// One node of the enumeration tree, with the tally of the polycubes found
/// up to it.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TracedStep {
    pub placed: Vec<Coordinates>,
    pub candidates: Vec<Coordinates>,
    pub discarded: Vec<Coordinates>,
    /// Polycubes found so far, this one included: entry `size - 1` counts
    /// those of `size` cells.
    pub counts_by_size: Vec<u64>,
}

/// Traces the enumeration of the fixed polycubes of at most
/// `max_cell_count` cells.
///
/// # Errors
///
/// Returns [`TraceError::CellCountOutOfRange`] if `max_cell_count` is zero
/// or exceeds [`MAX_TRACED_CELL_COUNT`].
pub fn trace_enumeration(max_cell_count: usize) -> Result<Vec<TracedStep>, TraceError> {
    if !(1..=MAX_TRACED_CELL_COUNT).contains(&max_cell_count) {
        return Err(TraceError::CellCountOutOfRange(max_cell_count));
    }
    let mut counts_by_size = vec![0; max_cell_count];
    let steps = polycubes_core::trace_enumeration(max_cell_count)
        .into_iter()
        .map(|step| {
            counts_by_size[step.placed.len() - 1] += 1;
            TracedStep::new(step, counts_by_size.clone())
        })
        .collect();
    Ok(steps)
}

impl TracedStep {
    fn new(step: EnumerationStep, counts_by_size: Vec<u64>) -> Self {
        Self {
            placed: step.placed,
            candidates: step.candidates,
            discarded: step.discarded,
            counts_by_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_step_counts_every_polycube_of_each_size() {
        let steps = trace_enumeration(MAX_TRACED_CELL_COUNT).expect("largest traced size");

        let last_step = steps.last().expect("the trace is not empty");
        assert_eq!(last_step.counts_by_size, [1, 3, 15, 86, 534]);
    }

    #[test]
    fn each_step_adds_one_to_the_count_of_its_size() {
        let steps = trace_enumeration(4).expect("four cells can be traced");

        let mut expected_counts = vec![0; 4];
        for step in steps {
            expected_counts[step.placed.len() - 1] += 1;
            assert_eq!(step.counts_by_size, expected_counts);
        }
    }

    #[test]
    fn refuses_sizes_outside_the_traced_range() {
        for cell_count in [0, MAX_TRACED_CELL_COUNT + 1] {
            assert_eq!(
                trace_enumeration(cell_count),
                Err(TraceError::CellCountOutOfRange(cell_count))
            );
        }
    }
}
