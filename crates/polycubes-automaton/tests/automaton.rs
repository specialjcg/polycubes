use std::collections::HashSet;

use polycubes_automaton::{
    Census, Fate, Limits, Neighbourhood, Rule, RuleError, census, evolve, next_generation,
};
use polycubes_core::Coordinates;

const LIMITS: Limits = Limits {
    generation_count: 30,
    population: 200,
};

fn rule(neighbourhood: Neighbourhood, survival_counts: &[usize], birth_counts: &[usize]) -> Rule {
    Rule::new(neighbourhood, survival_counts, birth_counts)
        .expect("counts within the neighbourhood")
}

fn unit_cube() -> Vec<Coordinates> {
    (0..8)
        .map(|corner| [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1])
        .collect()
}

#[test]
fn a_lone_cube_gives_birth_on_its_six_faces() {
    let born_beside_one = rule(Neighbourhood::Faces, &[], &[1]);
    let living: HashSet<Coordinates> = HashSet::from([[0, 0, 0]]);

    let next = next_generation(&living, &born_beside_one);

    let expected: HashSet<Coordinates> = HashSet::from([
        [1, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ]);
    assert_eq!(next, expected);
}

#[test]
fn corners_count_only_in_the_wide_neighbourhood() {
    let living: HashSet<Coordinates> = HashSet::from([[0, 0, 0], [1, 1, 1]]);

    let by_faces = next_generation(&living, &rule(Neighbourhood::Faces, &[1], &[6]));
    let by_corners = next_generation(
        &living,
        &rule(Neighbourhood::FacesEdgesCorners, &[1], &[26]),
    );

    assert!(by_faces.is_empty());
    assert_eq!(by_corners, living);
}

#[test]
fn the_cube_stands_still_when_seven_neighbours_keep_alive() {
    let evolution = evolve(
        &unit_cube(),
        &rule(Neighbourhood::FacesEdgesCorners, &[7], &[9]),
        LIMITS,
    );

    assert_eq!(evolution.fate, Fate::Still { generation: 0 });
}

#[test]
fn a_seed_no_rule_keeps_alive_dies_at_once() {
    let evolution = evolve(&unit_cube(), &rule(Neighbourhood::Faces, &[], &[6]), LIMITS);

    assert_eq!(evolution.fate, Fate::Dies { generation: 1 });
    assert_eq!(evolution.generations.len(), 2);
    assert!(evolution.generations[1].is_empty());
}

#[test]
fn two_cubes_touching_by_an_edge_blink_with_their_mirror() {
    let born_between_two = rule(Neighbourhood::Faces, &[], &[2]);

    let evolution = evolve(&[[0, 0, 0], [1, 1, 0]], &born_between_two, LIMITS);

    assert_eq!(
        evolution.fate,
        Fate::Cycles {
            generation: 0,
            period: 2
        }
    );
    assert_eq!(evolution.generations[1], vec![[0, 1, 0], [1, 0, 0]]);
}

#[test]
fn a_seed_that_keeps_growing_is_left_unsettled() {
    let ever_growing = rule(Neighbourhood::Faces, &[0, 1, 2, 3, 4, 5, 6], &[1]);

    let evolution = evolve(&[[0, 0, 0]], &ever_growing, LIMITS);

    assert_eq!(evolution.fate, Fate::Unsettled);
    let last_generation = evolution.generations.last().expect("the seed at least");
    assert!(last_generation.len() <= LIMITS.population);
}

#[test]
fn refuses_counts_beyond_the_neighbourhood_and_birth_from_nothing() {
    assert_eq!(
        Rule::new(Neighbourhood::Faces, &[7], &[1]),
        Err(RuleError::CountOutOfRange {
            count: 7,
            neighbour_count: 6
        })
    );
    assert_eq!(
        Rule::new(Neighbourhood::FacesEdgesCorners, &[], &[0]),
        Err(RuleError::BirthWithoutNeighbour)
    );
}

#[test]
fn the_census_holds_every_fixed_polycube() {
    let found = census(4, &rule(Neighbourhood::Faces, &[], &[6]), LIMITS);

    assert_eq!(
        found,
        Census {
            dying: 86,
            ..Census::default()
        }
    );
}

#[test]
fn the_census_tells_the_square_from_the_other_tetracubes() {
    // Each cube of the square touches two others; no other tetracube has
    // all its cubes in that case.
    let kept_by_two = rule(Neighbourhood::Faces, &[2], &[6]);

    let found = census(4, &kept_by_two, LIMITS);

    assert_eq!(found.still, 3);
    assert_eq!(found.dying, 83);
}

/// Fredkin's parity rule: a cell lives when an odd number of its six
/// neighbours do.
fn parity_rule() -> Rule {
    rule(Neighbourhood::Faces, &[1, 3, 5], &[1, 3, 5])
}

fn six_copies(seed: &[Coordinates], distance: i32) -> Vec<Coordinates> {
    let mut copies = Vec::new();
    for axis in 0..3 {
        for direction in [-1, 1] {
            for cell in seed {
                let mut moved = *cell;
                moved[axis] += direction * distance;
                copies.push(moved);
            }
        }
    }
    copies.sort_unstable();
    copies
}

#[test]
fn the_parity_rule_copies_a_polycube_six_times_at_each_power_of_two() {
    // A tetracube that leaves the plane, narrower than the distances tried.
    let seed = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];
    let limits = Limits {
        generation_count: 8,
        population: 10_000,
    };

    let evolution = evolve(&seed, &parity_rule(), limits);

    assert_eq!(evolution.generations[4], six_copies(&seed, 4));
    assert_eq!(evolution.generations[8], six_copies(&seed, 8));
}

fn twelve_copies(seed: &[Coordinates], distance: i32) -> Vec<Coordinates> {
    let mut copies = Vec::new();
    for still_axis in 0..3 {
        for first_direction in [-1, 1] {
            for second_direction in [-1, 1] {
                for cell in seed {
                    let mut moved = *cell;
                    moved[(still_axis + 1) % 3] += first_direction * distance;
                    moved[(still_axis + 2) % 3] += second_direction * distance;
                    copies.push(moved);
                }
            }
        }
    }
    copies.sort_unstable();
    copies
}

#[test]
fn the_parity_rule_over_edges_copies_a_polycube_twelve_times() {
    let seed = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]];
    let odd_counts = [1, 3, 5, 7, 9, 11];
    let edge_parity_rule = rule(Neighbourhood::Edges, &odd_counts, &odd_counts);
    let limits = Limits {
        generation_count: 4,
        population: 10_000,
    };

    let evolution = evolve(&seed, &edge_parity_rule, limits);

    assert_eq!(evolution.generations[4], twelve_copies(&seed, 4));
}
