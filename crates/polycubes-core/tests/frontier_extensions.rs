use std::collections::HashSet;

use polycubes_core::{Coordinates, FrontierExtensions, frontier_extensions};

const LARGEST_CHECKED_SIZE: usize = 6;
const CHOICE_SEQUENCE_COUNT: u64 = 12;

/// Deterministic stand-in for random choices, different for each seed.
fn scrambled_choices(seed: u64) -> impl FnMut(usize) -> usize {
    let mut state = seed;
    move |candidate_count| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).expect("31 bits fit") % candidate_count
    }
}

/// Visited extensions sorted like the formula: row `added - 1`, column
/// number of added cells that were not candidates of the frontier.
fn visited_counts(frontier: &FrontierExtensions) -> [[u64; 4]; 4] {
    let candidates: HashSet<Coordinates> = frontier.candidates.iter().copied().collect();
    let mut counts = [[0; 4]; 4];
    for extension in &frontier.extensions {
        let deeper_cell_count = extension
            .iter()
            .filter(|cell| !candidates.contains(*cell))
            .count();
        counts[extension.len() - 1][deeper_cell_count] += 1;
    }
    counts
}

#[test]
fn formula_counts_what_the_visit_finds_term_by_term() {
    for placed_cell_count in 1..=LARGEST_CHECKED_SIZE {
        for seed in 0..CHOICE_SEQUENCE_COUNT {
            let frontier = frontier_extensions(placed_cell_count, scrambled_choices(seed));

            assert_eq!(
                frontier.formula_counts,
                visited_counts(&frontier),
                "polycube {:?}",
                frontier.placed
            );
        }
    }
}

#[test]
fn single_cube_has_three_candidates_and_known_extension_totals() {
    let frontier = frontier_extensions(1, |_| 0);

    assert_eq!(frontier.placed, [[0, 0, 0]]);
    assert_eq!(frontier.candidates.len(), 3);
    // Polycubes of 2 to 5 cells all extend the single cube: OEIS A001931.
    let totals = frontier
        .formula_counts
        .map(|terms| terms.iter().sum::<u64>());
    assert_eq!(totals, [3, 15, 86, 534]);
}

#[test]
fn visited_extensions_are_distinct_and_disjoint_from_the_polycube() {
    let frontier = frontier_extensions(4, scrambled_choices(7));

    let mut seen_extensions = HashSet::new();
    for extension in &frontier.extensions {
        assert!((1..=4).contains(&extension.len()));
        assert!(extension.iter().all(|cell| !frontier.placed.contains(cell)));
        let mut cells = extension.clone();
        cells.sort_unstable();
        assert!(seen_extensions.insert(cells), "{extension:?} visited twice");
    }
}
