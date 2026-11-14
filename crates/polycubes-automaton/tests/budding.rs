use std::collections::HashSet;

use polycubes_automaton::grow_by_budding;
use polycubes_core::Coordinates;

const POPULATION_LIMIT: usize = 200;

/// A tetracube that differs from its mirror image.
const SCREW: [Coordinates; 4] = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];

fn copies_of(generation: &[Coordinates]) -> Vec<&[Coordinates]> {
    generation.chunks(SCREW.len()).collect()
}

fn step(from: Coordinates, to: Coordinates) -> Coordinates {
    [0, 1, 2].map(|axis| to[axis] - from[axis])
}

/// Signed volume spanned by the three steps of a screw: a rotation keeps
/// it, a mirror image reverses it.
fn handedness(screw: &[Coordinates]) -> i32 {
    let [first, second, third] = [0, 1, 2].map(|index| step(screw[index], screw[index + 1]));
    first[0] * (second[1] * third[2] - second[2] * third[1])
        - first[1] * (second[0] * third[2] - second[2] * third[0])
        + first[2] * (second[0] * third[1] - second[1] * third[0])
}

fn touch(cell: Coordinates, other: Coordinates) -> bool {
    step(cell, other).iter().map(|gap| gap.abs()).sum::<i32>() == 1
}

fn always_first(_placement_count: usize) -> usize {
    0
}

#[test]
fn a_polycube_that_never_buds_stays_alone() {
    let generations = grow_by_budding(&SCREW, 4, POPULATION_LIMIT, || false, always_first);

    assert_eq!(generations.len(), 5);
    assert!(generations.iter().all(|generation| generation == &SCREW));
}

#[test]
fn a_polycube_that_buds_makes_a_second_one_against_itself() {
    let generations = grow_by_budding(&SCREW, 1, POPULATION_LIMIT, || true, always_first);

    let copies = copies_of(&generations[1]);
    assert_eq!(copies.len(), 2);
    assert_eq!(copies[0], &SCREW);
    assert!(
        copies[1]
            .iter()
            .any(|&cell| SCREW.iter().any(|&parent_cell| touch(cell, parent_cell)))
    );
}

#[test]
fn every_copy_is_the_seed_turned_never_mirrored_and_none_overlap() {
    let mut choice_index = 0;
    let generations = grow_by_budding(
        &SCREW,
        5,
        POPULATION_LIMIT,
        || true,
        |placement_count| {
            choice_index += 7;
            choice_index % placement_count
        },
    );

    for pair in generations.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        assert!(after.len() > before.len(), "a free colony keeps budding");
        assert!(after.len() <= 2 * before.len());
        assert_eq!(&after[..before.len()], &before[..]);
        let distinct: HashSet<&Coordinates> = after.iter().collect();
        assert_eq!(distinct.len(), after.len(), "two copies overlap");
        for copy in copies_of(after) {
            assert_eq!(handedness(copy), handedness(&SCREW), "{copy:?}");
            assert!(
                copy.windows(2).all(|cells| touch(cells[0], cells[1])),
                "{copy:?} is not a screw"
            );
        }
    }
}

#[test]
fn no_copy_is_born_beyond_the_population_limit() {
    let generations = grow_by_budding(&SCREW, 8, 30, || true, always_first);

    // Seven screws hold 28 cells; an eighth would exceed 30.
    assert_eq!(generations.last().map(Vec::len), Some(28));
}

#[test]
fn a_bud_touches_no_copy_but_its_parent() {
    let mut choice_index = 0;
    let generations = grow_by_budding(
        &SCREW,
        6,
        POPULATION_LIMIT,
        || true,
        |placement_count| {
            choice_index += 5;
            choice_index % placement_count
        },
    );

    let colony = generations.last().expect("the seed at least");
    let copies = copies_of(colony);
    for (copy_index, copy) in copies.iter().enumerate().skip(1) {
        let touched_elders = copies[..copy_index]
            .iter()
            .filter(|elder| {
                copy.iter()
                    .any(|&cell| elder.iter().any(|&elder_cell| touch(cell, elder_cell)))
            })
            .count();
        assert_eq!(touched_elders, 1, "copy {copy_index} leans on several");
    }
}
