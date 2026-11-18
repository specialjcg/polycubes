use polycubes_automaton::{
    ConwayRule, DOMINO_GLIDER, Glider, Nature, SCREW_GLIDER, SQUARE_GLIDER, THICK_SQUARE_GLIDER,
    Watch, canonical_form, classify, symmetry,
};
use polycubes_core::Coordinates;

const WATCH: Watch = Watch {
    generation_count: 60,
    population_cap: 60,
};

/// Nobody is ever born and nobody dies, whatever the neighbours.
const FROZEN: ConwayRule = ConwayRule {
    fewest_neighbours_to_survive: 0,
    most_neighbours_to_survive: 26,
    fewest_neighbours_to_be_born: 1,
    most_neighbours_to_be_born: 1,
    fewest_faces_shared_to_be_born: 100,
};

/// A quarter turn about the z axis, then one about the x axis, then a move.
fn turned_and_moved(polycubes: &[Vec<Coordinates>]) -> Vec<Vec<Coordinates>> {
    polycubes
        .iter()
        .map(|cells| {
            cells
                .iter()
                .map(|&[x, y, z]| [-y, x, z])
                .map(|[x, y, z]| [x + 7, -z - 3, y + 11])
                .collect()
        })
        .collect()
}

#[test]
fn a_turned_and_moved_pattern_has_the_same_canonical_form() {
    let pattern = SQUARE_GLIDER.start();

    assert_eq!(
        canonical_form(&turned_and_moved(&pattern)),
        canonical_form(&pattern)
    );
}

#[test]
fn a_mirrored_screw_has_another_canonical_form() {
    let screw = vec![vec![[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]]];
    let mirror = vec![vec![[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, -1]]];

    assert_ne!(canonical_form(&screw), canonical_form(&mirror));
}

#[test]
fn known_gliders_are_classed_as_gliders_with_their_period() {
    let gliders = [
        DOMINO_GLIDER,
        SQUARE_GLIDER,
        THICK_SQUARE_GLIDER,
        SCREW_GLIDER,
    ];

    let found: Vec<Option<(Nature, usize)>> = gliders
        .iter()
        .map(|glider: &Glider| {
            classify(&glider.start(), glider.rule, WATCH)
                .map(|classification| (classification.nature, classification.period))
        })
        .collect();

    let expected: Vec<Option<(Nature, usize)>> = gliders
        .iter()
        .map(|glider| Some((Nature::Glider, glider.period)))
        .collect();
    assert_eq!(found, expected);
}

#[test]
fn a_glider_tells_its_shift_and_layers() {
    let classification = classify(
        &THICK_SQUARE_GLIDER.start(),
        THICK_SQUARE_GLIDER.rule,
        WATCH,
    );

    assert_eq!(
        classification.map(|found| (found.shift, found.layers)),
        Some((THICK_SQUARE_GLIDER.shift, [3, 4, 5]))
    );
}

#[test]
fn a_frozen_pattern_is_classed_as_still_with_a_period_of_one() {
    let pattern = vec![vec![[0, 0, 0], [1, 0, 0]], vec![[0, 1, 0], [0, 1, 1]]];

    let classification = classify(&pattern, FROZEN, WATCH);

    assert_eq!(
        classification.map(|found| (found.nature, found.period, found.shift)),
        Some((Nature::Still, 1, [0; 3]))
    );
}

#[test]
fn a_pattern_that_dies_out_is_not_classed() {
    let lone_domino = vec![vec![[0, 0, 0], [1, 0, 0]]];

    assert!(classify(&lone_domino, DOMINO_GLIDER.rule, WATCH).is_none());
}

#[test]
fn symmetry_counts_the_rotations_that_leave_a_pattern_as_it_was() {
    let domino = vec![vec![[0, 0, 0], [1, 0, 0]]];
    let cube: Vec<Vec<Coordinates>> = vec![
        vec![[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]],
        vec![[0, 0, 1], [1, 0, 1], [0, 1, 1], [1, 1, 1]],
    ];
    let screw = vec![vec![[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]]];

    assert_eq!(
        [symmetry(&domino), symmetry(&cube), symmetry(&screw)],
        [8, 8, 2]
    );
}
