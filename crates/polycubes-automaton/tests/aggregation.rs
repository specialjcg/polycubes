use std::collections::HashSet;

use polycubes_automaton::aggregate;
use polycubes_core::Coordinates;

/// A tetracube that differs from its mirror image.
const SCREW: [Coordinates; 4] = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];

fn step(from: Coordinates, to: Coordinates) -> Coordinates {
    [0, 1, 2].map(|axis| to[axis] - from[axis])
}

fn touch(cell: Coordinates, other: Coordinates) -> bool {
    step(cell, other).iter().map(|gap| gap.abs()).sum::<i32>() == 1
}

fn lean_on_each_other(copy: &[Coordinates], other: &[Coordinates]) -> bool {
    copy.iter()
        .any(|&cell| other.iter().any(|&other_cell| touch(cell, other_cell)))
}

/// Signed volume spanned by the three steps of a screw: a rotation keeps
/// it, a mirror image reverses it.
fn handedness(screw: &[Coordinates]) -> i32 {
    let [first, second, third] = [0, 1, 2].map(|index| step(screw[index], screw[index + 1]));
    first[0] * (second[1] * third[2] - second[2] * third[1])
        - first[1] * (second[0] * third[2] - second[2] * third[0])
        + first[2] * (second[0] * third[1] - second[1] * third[0])
}

/// Choices that look random but are the same at every run.
fn wandering_choice() -> impl FnMut(usize) -> usize {
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    move |choice_count| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).expect("31 bits fit") % choice_count
    }
}

#[test]
fn wanderers_stick_until_the_aggregate_holds_the_polycubes_asked_for() {
    let polycubes = aggregate(&SCREW, 12, wandering_choice());

    assert_eq!(polycubes.len(), 12);
    assert_eq!(polycubes[0], SCREW);
}

#[test]
fn every_wanderer_is_the_seed_turned_and_sticks_where_it_first_touches() {
    let polycubes = aggregate(&SCREW, 12, wandering_choice());

    let cells: Vec<Coordinates> = polycubes.concat();
    let distinct: HashSet<&Coordinates> = cells.iter().collect();
    assert_eq!(distinct.len(), cells.len(), "two polycubes overlap");
    for (arrival_rank, polycube) in polycubes.iter().enumerate().skip(1) {
        assert_eq!(handedness(polycube), handedness(&SCREW), "{polycube:?}");
        assert!(
            polycubes[..arrival_rank]
                .iter()
                .any(|elder| lean_on_each_other(polycube, elder)),
            "polycube {arrival_rank} floats apart"
        );
    }
}

#[test]
fn an_aggregate_of_lone_cells_spreads_far_wider_than_a_ball() {
    let cell_count = 80;
    let polycubes = aggregate(&[[0, 0, 0]], cell_count, wandering_choice());

    let box_volume: i32 = (0..3)
        .map(|axis| {
            let along_axis = polycubes.iter().flatten().map(|cell| cell[axis]);
            along_axis.clone().max().unwrap_or(0) - along_axis.min().unwrap_or(0) + 1
        })
        .product();
    assert_eq!(polycubes.len(), cell_count);
    assert!(box_volume > 8 * 80, "box of {box_volume} cells only");
}

#[test]
fn wanderers_that_never_turn_get_lost_and_leave_the_seed_alone() {
    let polycubes = aggregate(&SCREW, 5, |_| 0);

    assert_eq!(polycubes, [SCREW.to_vec()]);
}
