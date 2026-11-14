use std::collections::HashSet;

use polycubes_automaton::grow_by_mutating_buds;
use polycubes_core::Coordinates;

const POPULATION_LIMIT: usize = 200;

const SCREW: [Coordinates; 4] = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];

fn touch(cell: Coordinates, other: Coordinates) -> bool {
    (0..3)
        .map(|axis| (cell[axis] - other[axis]).abs())
        .sum::<i32>()
        == 1
}

fn lean_on_each_other(copy: &[Coordinates], other: &[Coordinates]) -> bool {
    copy.iter()
        .any(|&cell| other.iter().any(|&other_cell| touch(cell, other_cell)))
}

fn is_in_one_piece(cells: &[Coordinates]) -> bool {
    let mut reached = vec![cells[0]];
    let mut explored_count = 0;
    while explored_count < reached.len() {
        for &cell in cells {
            if !reached.contains(&cell) && touch(cell, reached[explored_count]) {
                reached.push(cell);
            }
        }
        explored_count += 1;
    }
    reached.len() == cells.len()
}

/// Squared distances between the cells, sorted: turning or moving a
/// polycube keeps them.
fn distances_within(cells: &[Coordinates]) -> Vec<i32> {
    let mut distances = Vec::new();
    for (index, cell) in cells.iter().enumerate() {
        for other in &cells[index + 1..] {
            distances.push((0..3).map(|axis| (cell[axis] - other[axis]).pow(2)).sum());
        }
    }
    distances.sort_unstable();
    distances
}

fn varied_choice() -> impl FnMut(usize) -> usize {
    let mut choice_index = 0;
    move |choice_count| {
        choice_index += 7;
        choice_index % choice_count
    }
}

#[test]
fn a_polycube_that_never_buds_stays_alone() {
    let lineage = grow_by_mutating_buds(&SCREW, 4, POPULATION_LIMIT, || false, |_| 0);

    assert_eq!(lineage.generations.len(), 5);
    assert!(lineage.generations.iter().all(|cells| cells == &SCREW));
    assert_eq!(lineage.parent_of_copy, [0]);
}

#[test]
fn every_bud_is_a_polycube_of_the_seed_size_and_none_overlap() {
    let lineage = grow_by_mutating_buds(&SCREW, 6, POPULATION_LIMIT, || true, varied_choice());

    let colony = lineage.generations.last().expect("the seed at least");
    assert!(colony.len() > SCREW.len(), "nothing budded");
    let distinct: HashSet<&Coordinates> = colony.iter().collect();
    assert_eq!(distinct.len(), colony.len(), "two polycubes overlap");
    for copy in colony.chunks(SCREW.len()) {
        assert!(is_in_one_piece(copy), "{copy:?} is in several pieces");
    }
}

#[test]
fn buds_drift_away_from_the_shape_of_the_seed() {
    let lineage = grow_by_mutating_buds(&SCREW, 6, POPULATION_LIMIT, || true, varied_choice());

    let colony = lineage.generations.last().expect("the seed at least");
    let shapes: HashSet<Vec<i32>> = colony.chunks(SCREW.len()).map(distances_within).collect();
    assert!(shapes.len() > 2, "only {} shapes", shapes.len());
}

#[test]
fn a_bud_leans_on_its_parent_and_on_no_other_elder() {
    let lineage = grow_by_mutating_buds(&SCREW, 6, POPULATION_LIMIT, || true, varied_choice());

    let colony = lineage.generations.last().expect("the seed at least");
    let copies: Vec<&[Coordinates]> = colony.chunks(SCREW.len()).collect();
    assert_eq!(lineage.parent_of_copy.len(), copies.len());
    for (copy_index, copy) in copies.iter().enumerate().skip(1) {
        let elders_leant_on: Vec<usize> = (0..copy_index)
            .filter(|&elder_index| lean_on_each_other(copy, copies[elder_index]))
            .collect();
        assert_eq!(elders_leant_on, [lineage.parent_of_copy[copy_index]]);
    }
}

#[test]
fn a_lone_cell_buds_lone_cells() {
    let lineage = grow_by_mutating_buds(&[[0, 0, 0]], 3, POPULATION_LIMIT, || true, |_| 0);

    let populations: Vec<usize> = lineage.generations.iter().map(Vec::len).collect();
    assert!(
        populations.windows(2).all(|pair| pair[1] > pair[0]),
        "{populations:?}"
    );
}
