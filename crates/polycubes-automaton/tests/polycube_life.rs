use std::collections::HashSet;

use polycubes_automaton::{LivingPolycube, PolycubeLifeRule, PolycubeWorld, live_as_polycubes};
use polycubes_core::Coordinates;

/// A tetracube that differs from its mirror image.
const SCREW: [Coordinates; 4] = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];

const RULE: PolycubeLifeRule = PolycubeLifeRule {
    most_neighbours_to_survive: 2,
    fewest_neighbours_to_be_born: 1,
    most_neighbours_to_be_born: 2,
};

const WORLD: PolycubeWorld = PolycubeWorld {
    generation_count: 14,
    reach: 3,
};

fn step(from: Coordinates, to: Coordinates) -> Coordinates {
    [0, 1, 2].map(|axis| to[axis] - from[axis])
}

fn touch(cell: Coordinates, other: Coordinates) -> bool {
    step(cell, other).iter().map(|gap| gap.abs()).sum::<i32>() == 1
}

fn lean_on_each_other(polycube: &LivingPolycube, other: &LivingPolycube) -> bool {
    polycube.cells.iter().any(|&cell| {
        other
            .cells
            .iter()
            .any(|&other_cell| touch(cell, other_cell))
    })
}

fn neighbour_count(polycube: &LivingPolycube, generation: &[LivingPolycube]) -> usize {
    generation
        .iter()
        .filter(|other| other.identity != polycube.identity && lean_on_each_other(polycube, other))
        .count()
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
fn varied_choice() -> impl FnMut(usize) -> usize {
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    move |choice_count| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).expect("31 bits fit") % choice_count
    }
}

fn played() -> Vec<Vec<LivingPolycube>> {
    live_as_polycubes(&SCREW, RULE, WORLD, || true, varied_choice())
}

#[test]
fn life_starts_from_the_seed_alone() {
    let generations = played();

    assert_eq!(generations.len(), WORLD.generation_count + 1);
    assert_eq!(generations[0].len(), 1);
    assert_eq!(generations[0][0].cells, SCREW);
}

#[test]
fn every_living_polycube_is_the_seed_whole_and_none_overlap() {
    for generation in played() {
        let cells: Vec<Coordinates> = generation
            .iter()
            .flat_map(|polycube| polycube.cells.iter().copied())
            .collect();
        let distinct: HashSet<&Coordinates> = cells.iter().collect();
        assert_eq!(distinct.len(), cells.len(), "two polycubes overlap");
        for polycube in &generation {
            assert_eq!(handedness(&polycube.cells), handedness(&SCREW));
        }
    }
}

#[test]
fn a_polycube_lives_on_unchanged_unless_too_many_lean_on_it() {
    let generations = played();

    let mut death_count = 0;
    for pair in generations.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        for polycube in before {
            let is_crowded = neighbour_count(polycube, before) > RULE.most_neighbours_to_survive;
            let lives_on = after.contains(polycube);
            assert_eq!(lives_on, !is_crowded, "polycube {}", polycube.identity);
            death_count += usize::from(is_crowded);
        }
    }
    assert!(death_count > 0, "nothing ever died");
}

#[test]
fn a_polycube_is_born_against_one_or_two_of_the_living() {
    let generations = played();

    for pair in generations.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        let known: HashSet<usize> = before.iter().map(|polycube| polycube.identity).collect();
        for newborn in after
            .iter()
            .filter(|polycube| !known.contains(&polycube.identity))
        {
            let leant_on_count = before
                .iter()
                .filter(|elder| lean_on_each_other(newborn, elder))
                .count();
            assert!(
                (RULE.fewest_neighbours_to_be_born..=RULE.most_neighbours_to_be_born)
                    .contains(&leant_on_count),
                "polycube {} born against {leant_on_count}",
                newborn.identity
            );
        }
    }
}

#[test]
fn no_polycube_leaves_the_world_yet_births_go_on_to_the_end() {
    let generations = played();

    for cell in generations
        .iter()
        .flatten()
        .flat_map(|polycube| &polycube.cells)
    {
        assert!(
            cell.iter()
                .all(|coordinate| coordinate.abs() <= WORLD.reach)
        );
    }
    let last_two = &generations[generations.len() - 2..];
    assert!(
        last_two[1]
            .iter()
            .any(|polycube| !last_two[0].contains(polycube)),
        "the world froze"
    );
}

#[test]
fn no_identity_is_given_twice() {
    let generations = played();

    let mut cells_by_identity = std::collections::HashMap::new();
    for polycube in generations.iter().flatten() {
        let known_cells = cells_by_identity
            .entry(polycube.identity)
            .or_insert(&polycube.cells);
        assert_eq!(*known_cells, &polycube.cells);
    }
}
