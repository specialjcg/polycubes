//! The closed formula against a plain visit, for the frontend.

use std::collections::HashSet;

use polycubes_core::{Coordinates, FrontierExtensions};
use serde::Serialize;

/// Largest polycube extended: beyond it the visit runs through tens of
/// thousands of extensions.
pub const MAX_EXTENDED_CELL_COUNT: usize = 6;

/// Extensions of each size sent as examples: enough to watch the visit, few
/// enough to keep the answer small.
pub const MAX_EXAMPLE_COUNT: usize = 40;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ClosedFormError {
    #[error("cell count must be between 1 and {MAX_EXTENDED_CELL_COUNT}, got {0}")]
    CellCountOutOfRange(usize),
}

/// A polycube, its frontier, and its extensions sorted by size.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedFormComparison {
    pub placed: Vec<Coordinates>,
    pub candidates: Vec<Coordinates>,
    /// Entry `added - 1` holds the extensions of `added` cells.
    pub extension_sizes: Vec<ExtensionSize>,
}

/// The extensions that add a given number of cells, counted by the formula
/// and by the visit. Both counts are split by how many of the added cells
/// lie beyond the frontier: entry 0 uses candidates only.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSize {
    pub formula_counts: Vec<u64>,
    pub visited_counts: Vec<u64>,
    /// Cells added by some of the visited extensions, spread over the visit.
    pub examples: Vec<Vec<Coordinates>>,
}

/// Picks a polycube of `placed_cell_count` cells by following
/// `choose_candidate` down the enumeration tree, then compares both counts
/// of its extensions.
///
/// # Errors
///
/// Returns [`ClosedFormError::CellCountOutOfRange`] if `placed_cell_count`
/// is zero or exceeds [`MAX_EXTENDED_CELL_COUNT`].
pub fn compare_closed_form(
    placed_cell_count: usize,
    choose_candidate: impl FnMut(usize) -> usize,
) -> Result<ClosedFormComparison, ClosedFormError> {
    if !(1..=MAX_EXTENDED_CELL_COUNT).contains(&placed_cell_count) {
        return Err(ClosedFormError::CellCountOutOfRange(placed_cell_count));
    }
    let frontier = polycubes_core::frontier_extensions(placed_cell_count, choose_candidate);
    let extension_sizes = (1..=frontier.formula_counts.len())
        .map(|added_cell_count| ExtensionSize::of(&frontier, added_cell_count))
        .collect();
    Ok(ClosedFormComparison {
        placed: frontier.placed,
        candidates: frontier.candidates,
        extension_sizes,
    })
}

impl ExtensionSize {
    fn of(frontier: &FrontierExtensions, added_cell_count: usize) -> Self {
        let candidates: HashSet<Coordinates> = frontier.candidates.iter().copied().collect();
        let extensions: Vec<&Vec<Coordinates>> = frontier
            .extensions
            .iter()
            .filter(|extension| extension.len() == added_cell_count)
            .collect();
        let mut visited_counts = vec![0; added_cell_count];
        for extension in &extensions {
            let cells_beyond_frontier = extension
                .iter()
                .filter(|cell| !candidates.contains(*cell))
                .count();
            visited_counts[cells_beyond_frontier] += 1;
        }
        Self {
            formula_counts: frontier.formula_counts[added_cell_count - 1][..added_cell_count]
                .to_vec(),
            visited_counts,
            examples: spread_examples(&extensions),
        }
    }
}

/// At most [`MAX_EXAMPLE_COUNT`] extensions, evenly spaced in visit order so
/// that every kind of extension shows up, not only the first ones visited.
fn spread_examples(extensions: &[&Vec<Coordinates>]) -> Vec<Vec<Coordinates>> {
    let stride = extensions.len().div_ceil(MAX_EXAMPLE_COUNT).max(1);
    extensions
        .iter()
        .step_by(stride)
        .map(|extension| (*extension).clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_following_last_candidates(placed_cell_count: usize) -> ClosedFormComparison {
        compare_closed_form(placed_cell_count, |candidate_count| candidate_count - 1)
            .expect("size within range")
    }

    #[test]
    fn formula_and_visit_agree_for_every_extension_size() {
        let comparison = compare_following_last_candidates(MAX_EXTENDED_CELL_COUNT);

        assert_eq!(comparison.placed.len(), MAX_EXTENDED_CELL_COUNT);
        assert_eq!(comparison.extension_sizes.len(), 4);
        for (size_index, extension_size) in comparison.extension_sizes.iter().enumerate() {
            assert_eq!(extension_size.formula_counts.len(), size_index + 1);
            assert_eq!(extension_size.formula_counts, extension_size.visited_counts);
        }
    }

    #[test]
    fn single_cube_extends_into_every_polycube_of_two_to_five_cells() {
        let comparison = compare_following_last_candidates(1);

        let totals: Vec<u64> = comparison
            .extension_sizes
            .iter()
            .map(|extension_size| extension_size.visited_counts.iter().sum())
            .collect();
        assert_eq!(totals, [3, 15, 86, 534]);
    }

    #[test]
    fn examples_are_capped_and_have_the_size_they_illustrate() {
        let comparison = compare_following_last_candidates(4);

        for (size_index, extension_size) in comparison.extension_sizes.iter().enumerate() {
            let visited_count: u64 = extension_size.visited_counts.iter().sum();
            let expected_example_count = usize::try_from(visited_count)
                .expect("small count")
                .min(MAX_EXAMPLE_COUNT);
            assert!(extension_size.examples.len() <= MAX_EXAMPLE_COUNT);
            assert!(extension_size.examples.len() * 2 > expected_example_count);
            for example in &extension_size.examples {
                assert_eq!(example.len(), size_index + 1);
            }
        }
    }

    #[test]
    fn refuses_sizes_outside_the_extended_range() {
        for cell_count in [0, MAX_EXTENDED_CELL_COUNT + 1] {
            assert_eq!(
                compare_closed_form(cell_count, |_| 0),
                Err(ClosedFormError::CellCountOutOfRange(cell_count))
            );
        }
    }
}
