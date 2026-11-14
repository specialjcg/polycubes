use std::collections::HashSet;

use polycubes_core::{Coordinates, EnumerationStep, trace_enumeration};

/// OEIS A001931, sizes 1 to 5.
const KNOWN_FIXED_COUNTS: [usize; 5] = [1, 3, 15, 86, 534];

fn is_adjacent(first: Coordinates, second: Coordinates) -> bool {
    let distance: i32 = (0..3).map(|axis| (first[axis] - second[axis]).abs()).sum();
    distance == 1
}

fn touches(cell: Coordinates, polycube: &[Coordinates]) -> bool {
    polycube.iter().any(|&placed| is_adjacent(cell, placed))
}

#[test]
fn visits_every_fixed_polycube_exactly_once() {
    let steps = trace_enumeration(KNOWN_FIXED_COUNTS.len());

    for (size_index, &known_count) in KNOWN_FIXED_COUNTS.iter().enumerate() {
        let polycubes_of_size: HashSet<Vec<Coordinates>> = steps
            .iter()
            .filter(|step| step.placed.len() == size_index + 1)
            .map(|step| {
                let mut cells = step.placed.clone();
                cells.sort_unstable();
                cells
            })
            .collect();
        assert_eq!(polycubes_of_size.len(), known_count);
    }
    assert_eq!(steps.len(), KNOWN_FIXED_COUNTS.iter().sum::<usize>());
}

#[test]
fn starts_with_the_single_cube_and_its_three_candidates() {
    let steps = trace_enumeration(3);

    assert_eq!(
        steps[0],
        EnumerationStep {
            placed: vec![[0, 0, 0]],
            candidates: vec![[1, 0, 0], [0, 1, 0], [0, 0, 1]],
            discarded: vec![],
        }
    );
}

#[test]
fn candidates_and_discarded_cells_touch_the_polycube_without_overlapping() {
    for step in trace_enumeration(4) {
        let mut seen_cells = HashSet::new();
        let all_cells = step
            .placed
            .iter()
            .chain(&step.candidates)
            .chain(&step.discarded);
        for &cell in all_cells {
            assert!(seen_cells.insert(cell), "{cell:?} listed twice in {step:?}");
        }
        for &cell in step.candidates.iter().chain(&step.discarded) {
            assert!(
                touches(cell, &step.placed),
                "{cell:?} is detached in {step:?}"
            );
        }
    }
}

#[test]
fn a_polycube_of_the_largest_size_has_no_candidate() {
    let largest_size = 4;

    for step in trace_enumeration(largest_size) {
        assert_eq!(
            step.candidates.is_empty(),
            step.placed.len() == largest_size,
            "{step:?}"
        );
    }
}

#[test]
fn a_cell_discarded_at_a_node_stays_discarded_below_it() {
    let steps = trace_enumeration(4);

    for pair in steps.windows(2) {
        let (parent, child) = (&pair[0], &pair[1]);
        if child.placed.len() == parent.placed.len() + 1 {
            for cell in &parent.discarded {
                assert!(
                    child.discarded.contains(cell),
                    "{cell:?} came back in {child:?}"
                );
            }
        }
    }
}
