use std::collections::HashSet;

use polycubes_automaton::grow_colony;
use polycubes_core::Coordinates;

const POPULATION_LIMIT: usize = 500;

fn touches(cell: Coordinates, others: &[Coordinates]) -> bool {
    others.iter().any(|other| {
        (0..3)
            .map(|axis| (cell[axis] - other[axis]).abs())
            .sum::<i32>()
            == 1
    })
}

#[test]
fn a_colony_where_no_cell_divides_stays_as_it_is() {
    let seed = [[0, 0, 0], [1, 0, 0]];

    let generations = grow_colony(&seed, 5, POPULATION_LIMIT, || false, |_| 0);

    assert_eq!(generations.len(), 6);
    assert!(generations.iter().all(|generation| generation == &seed));
}

#[test]
fn a_lone_cell_that_divides_makes_two() {
    let generations = grow_colony(&[[0, 0, 0]], 1, POPULATION_LIMIT, || true, |_| 0);

    assert_eq!(generations[1].len(), 2);
    assert_eq!(generations[1][0], [0, 0, 0]);
    assert!(touches(generations[1][1], &generations[0]));
}

#[test]
fn a_colony_at_most_doubles_and_keeps_its_cells_in_birth_order() {
    let mut division_index = 0;
    let generations = grow_colony(
        &[[0, 0, 0]],
        6,
        POPULATION_LIMIT,
        || true,
        |free_neighbour_count| {
            division_index += 1;
            division_index % free_neighbour_count
        },
    );

    for pair in generations.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        assert!(after.len() <= 2 * before.len());
        assert!(after.len() > before.len(), "a free colony keeps growing");
        assert_eq!(&after[..before.len()], &before[..]);
        let distinct: HashSet<&Coordinates> = after.iter().collect();
        assert_eq!(distinct.len(), after.len(), "a cell was born twice");
        for &newborn in &after[before.len()..] {
            assert!(touches(newborn, after), "{newborn:?} is detached");
        }
    }
}

#[test]
fn a_colony_stops_growing_at_the_population_limit() {
    let generations = grow_colony(&[[0, 0, 0]], 12, 40, || true, |_| 0);

    assert_eq!(generations.len(), 13);
    assert!(generations.iter().all(|generation| generation.len() <= 40));
    assert_eq!(generations[12].len(), 40);
}

#[test]
fn a_newborn_touches_no_cell_but_its_mother() {
    let mut division_index = 0;
    let generations = grow_colony(
        &[[0, 0, 0]],
        8,
        POPULATION_LIMIT,
        || true,
        |free_neighbour_count| {
            division_index += 3;
            division_index % free_neighbour_count
        },
    );

    let colony = generations.last().expect("the seed at least");
    for (birth_rank, &newborn) in colony.iter().enumerate().skip(1) {
        let touched_elders = colony[..birth_rank]
            .iter()
            .filter(|&&elder| touches(newborn, &[elder]))
            .count();
        assert_eq!(touched_elders, 1, "cell {birth_rank} leans on several");
    }
}
