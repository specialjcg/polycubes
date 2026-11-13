use std::collections::BTreeSet;

use polycubes_core::Coordinates;
use polycubes_sampling::{bounding_box, grow_by_random_neighbour};

const GROWN_CELL_COUNT: usize = 40;

/// Deterministic stand-in for a random choice: cycles through the candidates.
fn cycling_chooser() -> impl FnMut(usize) -> usize {
    let mut call_count = 0;
    move |candidate_count| {
        call_count += 7;
        call_count % candidate_count
    }
}

fn are_face_neighbours(left: Coordinates, right: Coordinates) -> bool {
    let distance: i32 = left
        .iter()
        .zip(right)
        .map(|(left_coordinate, right_coordinate)| (left_coordinate - right_coordinate).abs())
        .sum();
    distance == 1
}

#[test]
fn grows_the_requested_number_of_distinct_cells() {
    let cells = grow_by_random_neighbour(GROWN_CELL_COUNT, cycling_chooser());

    let distinct_cells: BTreeSet<Coordinates> = cells.iter().copied().collect();
    assert_eq!(cells.len(), GROWN_CELL_COUNT);
    assert_eq!(distinct_cells.len(), GROWN_CELL_COUNT);
}

#[test]
fn every_added_cell_touches_an_earlier_one() {
    let cells = grow_by_random_neighbour(GROWN_CELL_COUNT, cycling_chooser());

    for (cell_index, &cell) in cells.iter().enumerate().skip(1) {
        let earlier_cells = &cells[..cell_index];
        assert!(
            earlier_cells
                .iter()
                .any(|&earlier_cell| are_face_neighbours(cell, earlier_cell))
        );
    }
}

#[test]
fn offers_the_six_neighbours_of_the_first_cell() {
    let mut offered_candidate_counts = Vec::new();

    let _ = grow_by_random_neighbour(2, |candidate_count| {
        offered_candidate_counts.push(candidate_count);
        0
    });

    assert_eq!(offered_candidate_counts, [6]);
}

#[test]
fn grows_nothing_when_no_cell_is_requested() {
    let cells = grow_by_random_neighbour(0, cycling_chooser());

    assert!(cells.is_empty());
}

#[test]
fn bounding_box_spans_the_extreme_cells_on_each_axis() {
    let cells = [[0, 0, 0], [1, 0, 0], [2, 0, 0], [2, -1, 0]];

    let side_lengths = bounding_box(&cells);

    assert_eq!(side_lengths, [3, 2, 1]);
}

#[test]
fn bounding_box_of_no_cell_is_empty() {
    let side_lengths = bounding_box(&[]);

    assert_eq!(side_lengths, [0, 0, 0]);
}
