use std::collections::{BTreeSet, HashSet};

use polycubes_automaton::{
    ConwayRule, DOMINO, DOMINO_GLIDER, LivingPolycube, SCREW_GLIDER, SQUARE_GLIDER, Soup,
    THICK_SQUARE_GLIDER, Torus, play_conway, soup,
};
use polycubes_core::Coordinates;

const RULE: ConwayRule = DOMINO_GLIDER.rule;

const TORUS: Torus = Torus { side: 31 };

fn wrapped(cell: Coordinates) -> Coordinates {
    cell.map(|coordinate| coordinate.rem_euclid(TORUS.side))
}

fn moved(polycubes: &[Vec<Coordinates>], shift: Coordinates) -> Vec<Vec<Coordinates>> {
    polycubes
        .iter()
        .map(|cells| {
            cells
                .iter()
                .map(|cell| wrapped([0, 1, 2].map(|axis| cell[axis] + shift[axis])))
                .collect()
        })
        .collect()
}

/// The polycubes of a generation, whatever order and identities they have.
fn layout(
    polycubes: impl IntoIterator<Item = Vec<Coordinates>>,
) -> BTreeSet<BTreeSet<Coordinates>> {
    polycubes
        .into_iter()
        .map(|cells| cells.into_iter().collect())
        .collect()
}

fn layout_of(generation: &[LivingPolycube]) -> BTreeSet<BTreeSet<Coordinates>> {
    layout(generation.iter().map(|polycube| polycube.cells.clone()))
}

fn glider_at(shift: Coordinates) -> Vec<Vec<Coordinates>> {
    moved(&DOMINO_GLIDER.start(), shift)
}

fn touch(cell: Coordinates, other: Coordinates) -> bool {
    let gaps = [0, 1, 2].map(|axis| (cell[axis] - other[axis]).rem_euclid(TORUS.side));
    let steps: i32 = gaps.iter().map(|&gap| gap.min(TORUS.side - gap)).sum();
    steps == 1
}

fn faces_shared(cells: &[Coordinates], others: &[Coordinates]) -> usize {
    cells
        .iter()
        .map(|&cell| others.iter().filter(|&&other| touch(cell, other)).count())
        .sum()
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

fn soup_start() -> Vec<Vec<Coordinates>> {
    soup(
        &DOMINO,
        Soup {
            side: 8,
            attempt_count: 40,
        },
        varied_choice(),
    )
}

/// A soup, which mostly settles, and a glider, which never stops giving birth.
fn played_starts() -> Vec<Vec<Vec<LivingPolycube>>> {
    [soup_start(), glider_at([15, 15, 15])]
        .iter()
        .map(|start| play_conway(start, RULE, TORUS, 30))
        .collect()
}

#[test]
fn every_glider_comes_back_moved_after_its_period() {
    for glider in [
        DOMINO_GLIDER,
        SQUARE_GLIDER,
        THICK_SQUARE_GLIDER,
        SCREW_GLIDER,
    ] {
        let start = moved(&glider.start(), [15, 15, 15]);

        let generations = play_conway(&start, glider.rule, TORUS, glider.period);

        assert_eq!(
            layout_of(&generations[glider.period]),
            layout(moved(&start, glider.shift)),
            "{glider:?}"
        );
    }
}

/// Number of layers the cells spread over along each axis.
fn layer_counts(polycubes: &[Vec<Coordinates>]) -> [usize; 3] {
    [0, 1, 2].map(|axis| {
        polycubes
            .iter()
            .flatten()
            .map(|cell| cell[axis])
            .collect::<BTreeSet<i32>>()
            .len()
    })
}

#[test]
fn the_square_glider_is_a_solid_flying_along_a_diagonal_of_the_cube() {
    assert!(
        layer_counts(&SQUARE_GLIDER.start())
            .iter()
            .all(|&count| count >= 3)
    );
    assert!(SQUARE_GLIDER.shift.iter().all(|&step| step != 0));
}

#[test]
fn the_thick_square_glider_shares_the_rule_and_speed_of_the_square_glider_in_fewer_generations() {
    let speed = |glider: polycubes_automaton::Glider| {
        f64::from(glider.shift[0].abs()) / glider.period as f64
    };

    assert_eq!(THICK_SQUARE_GLIDER.rule, SQUARE_GLIDER.rule);
    const { assert!(THICK_SQUARE_GLIDER.period < SQUARE_GLIDER.period) };
    assert!(THICK_SQUARE_GLIDER.shift.iter().all(|&step| step != 0));
    assert!((speed(THICK_SQUARE_GLIDER) - speed(SQUARE_GLIDER)).abs() < f64::EPSILON);
    assert!(
        layer_counts(&THICK_SQUARE_GLIDER.start())
            .iter()
            .all(|&count| count >= 3)
    );
}

#[test]
fn the_screw_glider_moves_one_cell_a_generation_along_an_axis() {
    let moved_axes = SCREW_GLIDER.shift.iter().filter(|&&step| step != 0).count();
    let distance: i32 = SCREW_GLIDER.shift.iter().map(|step| step.abs()).sum();

    assert_eq!(moved_axes, 1);
    assert_eq!(usize::try_from(distance), Ok(SCREW_GLIDER.period));
}

#[test]
fn a_glider_leaving_the_torus_comes_back_on_the_other_side() {
    let start = glider_at([1, TORUS.side - 5, 4]);

    let generations = play_conway(&start, RULE, TORUS, 2 * DOMINO_GLIDER.period);

    let twice_shifted = DOMINO_GLIDER.shift.map(|step| 2 * step);
    assert_eq!(
        layout_of(&generations[2 * DOMINO_GLIDER.period]),
        layout(moved(&start, twice_shifted))
    );
}

#[test]
fn a_lone_polycube_dies_of_isolation() {
    let generations = play_conway(&[DOMINO.to_vec()], RULE, TORUS, 1);

    assert!(generations[1].is_empty());
}

#[test]
fn a_turned_start_plays_the_same_game_turned() {
    let quarter_turn = |[x, y, z]: Coordinates| wrapped([x, -z, y]);
    let start = soup_start();
    let turned_start: Vec<Vec<Coordinates>> = start
        .iter()
        .map(|cells| cells.iter().copied().map(quarter_turn).collect())
        .collect();

    let played = play_conway(&start, RULE, TORUS, 12);
    let turned_played = play_conway(&turned_start, RULE, TORUS, 12);

    for (generation, turned_generation) in played.iter().zip(&turned_played) {
        let turned_layout = layout(
            generation
                .iter()
                .map(|polycube| polycube.cells.iter().copied().map(quarter_turn).collect()),
        );
        assert_eq!(layout_of(turned_generation), turned_layout);
    }
}

#[test]
fn a_polycube_lives_on_while_two_or_three_lean_on_it() {
    for pair in played_starts()
        .iter()
        .flat_map(|generations| generations.windows(2))
    {
        let (before, after) = (&pair[0], &pair[1]);
        for polycube in before {
            let neighbour_count = before
                .iter()
                .filter(|other| {
                    other.identity != polycube.identity
                        && faces_shared(&polycube.cells, &other.cells) > 0
                })
                .count();
            assert_eq!(
                after.contains(polycube),
                (2..=3).contains(&neighbour_count),
                "polycube {}",
                polycube.identity
            );
        }
    }
}

#[test]
fn a_polycube_is_born_snug_against_exactly_two_and_apart_from_other_newborns() {
    let mut birth_count = 0;
    for pair in played_starts()
        .iter()
        .flat_map(|generations| generations.windows(2))
    {
        let (before, after) = (&pair[0], &pair[1]);
        let known: HashSet<usize> = before.iter().map(|polycube| polycube.identity).collect();
        let newborns: Vec<&LivingPolycube> = after
            .iter()
            .filter(|polycube| !known.contains(&polycube.identity))
            .collect();
        for newborn in &newborns {
            let leant_on: Vec<&LivingPolycube> = before
                .iter()
                .filter(|elder| faces_shared(&newborn.cells, &elder.cells) > 0)
                .collect();
            let shared: usize = leant_on
                .iter()
                .map(|elder| faces_shared(&newborn.cells, &elder.cells))
                .sum();
            assert_eq!(leant_on.len(), 2, "polycube {}", newborn.identity);
            assert!(shared >= 2, "polycube {}", newborn.identity);
            for other in &newborns {
                if other.identity != newborn.identity {
                    assert_eq!(faces_shared(&newborn.cells, &other.cells), 0);
                }
            }
        }
        birth_count += newborns.len();
    }
    assert!(birth_count > 0, "nothing was ever born");
}

#[test]
fn no_cell_is_taken_twice_and_all_stay_on_the_torus() {
    for generation in played_starts().into_iter().flatten() {
        let cells: Vec<Coordinates> = generation
            .iter()
            .flat_map(|polycube| polycube.cells.iter().copied())
            .collect();
        let distinct: HashSet<&Coordinates> = cells.iter().collect();
        assert_eq!(distinct.len(), cells.len());
        assert!(
            cells
                .iter()
                .flatten()
                .all(|coordinate| (0..TORUS.side).contains(coordinate))
        );
    }
}

#[test]
fn a_soup_is_whole_dominoes_apart_inside_its_box() {
    let start = soup_start();

    assert!(start.len() > 10, "only {} dominoes", start.len());
    let mut taken = HashSet::new();
    for cells in &start {
        assert_eq!(cells.len(), DOMINO.len());
        assert_eq!(faces_shared(&cells[..1], &cells[1..]), 1);
        for cell in cells {
            assert!(cell.iter().all(|coordinate| (0..8).contains(coordinate)));
            assert!(taken.insert(*cell), "two dominoes overlap");
        }
    }
}
