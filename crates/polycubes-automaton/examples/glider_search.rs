//! Searches for gliders that are truly three-dimensional in the Conway-like
//! life of whole polycubes: patterns that come back as they were, moved,
//! and whose cells spread over at least two layers along every axis.
//!
//! Tiny random starts are tried for many shapes and rules; each glider found
//! is printed once per shape, rule, period and shift, with its cells.
//!
//! Usage: `cargo run --release -p polycubes-automaton --example glider_search -- <starts per rule> [first soup seed]`

use std::collections::{BTreeSet, HashMap};

use polycubes_automaton::{ConwayRule, LivingPolycube, Soup, Torus, play_conway, soup};
use polycubes_core::Coordinates;
use rayon::prelude::*;

const GENERATION_COUNT: usize = 60;
const POPULATION_CAP: usize = 60;

/// Wide enough that nothing comes back round within the game.
const TORUS: Torus = Torus { side: 1000 };
const MIDDLE: i32 = 500;

const SHAPES: [(&str, &[Coordinates]); 10] = [
    ("domino", &[[0, 0, 0], [1, 0, 0]]),
    ("I3", &[[0, 0, 0], [1, 0, 0], [2, 0, 0]]),
    ("L3", &[[0, 0, 0], [1, 0, 0], [1, 1, 0]]),
    ("O4", &[[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]]),
    ("T4", &[[0, 0, 0], [1, 0, 0], [2, 0, 0], [1, 1, 0]]),
    ("S4", &[[0, 0, 0], [1, 0, 0], [1, 1, 0], [2, 1, 0]]),
    ("L4", &[[0, 0, 0], [1, 0, 0], [2, 0, 0], [2, 1, 0]]),
    ("screw", &[[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]]),
    (
        "mirror screw",
        &[[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, -1]],
    ),
    ("tripod", &[[0, 0, 0], [1, 0, 0], [0, 1, 0], [0, 0, 1]]),
];

const SURVIVALS: [(usize, usize); 5] = [(1, 2), (1, 3), (2, 3), (2, 4), (3, 4)];
const BIRTHS: [(usize, usize); 4] = [(1, 1), (2, 2), (3, 3), (2, 3)];
const FACES_SHARED: [usize; 5] = [2, 3, 4, 5, 6];

struct Glider {
    shape_name: &'static str,
    rule: ConwayRule,
    period: usize,
    shift: Coordinates,
    layout: Vec<Vec<Coordinates>>,
}

fn moved_to_origin(living: &[LivingPolycube]) -> (Vec<Vec<Coordinates>>, Coordinates) {
    let lowest_corner = [0, 1, 2].map(|axis| {
        living
            .iter()
            .flat_map(|polycube| &polycube.cells)
            .map(|cell| cell[axis])
            .min()
            .unwrap_or(0)
    });
    let mut layout: Vec<Vec<Coordinates>> = living
        .iter()
        .map(|polycube| {
            let mut cells: Vec<Coordinates> = polycube
                .cells
                .iter()
                .map(|cell| [0, 1, 2].map(|axis| cell[axis] - lowest_corner[axis]))
                .collect();
            cells.sort_unstable();
            cells
        })
        .collect();
    layout.sort_unstable();
    (layout, lowest_corner)
}

/// Number of layers the pattern spreads over along each axis, fewest first.
fn layers(layout: &[Vec<Coordinates>]) -> [usize; 3] {
    let mut counts = [0, 1, 2].map(|axis| {
        layout
            .iter()
            .flatten()
            .map(|cell| cell[axis])
            .collect::<BTreeSet<i32>>()
            .len()
    });
    counts.sort_unstable();
    counts
}

/// At least two layers along every axis: not a flat pattern.
fn is_solid(layout: &[Vec<Coordinates>]) -> bool {
    (0..3).all(|axis| {
        layout
            .iter()
            .flatten()
            .map(|cell| cell[axis])
            .collect::<BTreeSet<i32>>()
            .len()
            >= 2
    })
}

fn seeded_choice(seed: u64) -> impl FnMut(usize) -> usize {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x2545_f491_4f6c_dd1d;
    move |choice_count| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).expect("31 bits fit") % choice_count
    }
}

/// A tiny start: a few polycubes poured in a box of 4 to 6 cells.
fn tiny_start(shape: &[Coordinates], seed: u64) -> Vec<Vec<Coordinates>> {
    let mut choose = seeded_choice(seed);
    let side = 4 + i32::try_from(choose(3)).unwrap_or(0);
    let attempt_count = 3 + choose(8);
    soup(
        shape,
        Soup {
            side,
            attempt_count,
        },
        choose,
    )
    .into_iter()
    .map(|cells| {
        cells
            .into_iter()
            .map(|cell| cell.map(|coordinate| coordinate + MIDDLE))
            .collect()
    })
    .collect()
}

/// The solid glider the start turns into, if any.
fn glider_from(
    start: &[Vec<Coordinates>],
    rule: ConwayRule,
) -> Option<(usize, Coordinates, Vec<Vec<Coordinates>>)> {
    let mut seen: HashMap<Vec<Vec<Coordinates>>, (usize, Coordinates)> = HashMap::new();
    let mut cells: Vec<Vec<Coordinates>> = start.to_vec();
    for generation in 0..=GENERATION_COUNT {
        let living = play_conway(&cells, rule, TORUS, 0)
            .pop()
            .unwrap_or_default();
        if living.is_empty() || living.len() > POPULATION_CAP {
            return None;
        }
        let (layout, corner) = moved_to_origin(&living);
        if let Some(&(earlier, earlier_corner)) = seen.get(&layout) {
            let shift = [0, 1, 2].map(|axis| corner[axis] - earlier_corner[axis]);
            return (shift != [0; 3] && is_solid(&layout)).then_some((
                generation - earlier,
                shift,
                layout,
            ));
        }
        seen.insert(layout, (generation, corner));
        cells = play_conway(&cells, rule, TORUS, 1)
            .pop()
            .unwrap_or_default()
            .into_iter()
            .map(|polycube| polycube.cells)
            .collect();
    }
    None
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let starts_per_rule: u64 = arguments
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or(200);
    let first_seed: u64 = arguments
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or(1);
    let mut trials = Vec::new();
    for (shape_name, shape) in SHAPES {
        for (fewest_to_survive, most_to_survive) in SURVIVALS {
            for (fewest_to_be_born, most_to_be_born) in BIRTHS {
                for fewest_faces in FACES_SHARED {
                    let rule = ConwayRule {
                        fewest_neighbours_to_survive: fewest_to_survive,
                        most_neighbours_to_survive: most_to_survive,
                        fewest_neighbours_to_be_born: fewest_to_be_born,
                        most_neighbours_to_be_born: most_to_be_born,
                        fewest_faces_shared_to_be_born: fewest_faces,
                    };
                    trials.push((shape_name, shape, rule));
                }
            }
        }
    }
    let gliders: Vec<Glider> = trials
        .par_iter()
        .flat_map_iter(|&(shape_name, shape, rule)| {
            let mut found: HashMap<(usize, Coordinates), Glider> = HashMap::new();
            for seed in first_seed..first_seed + starts_per_rule {
                if let Some((period, shift, layout)) = glider_from(&tiny_start(shape, seed), rule) {
                    let mut sorted_shift = shift.map(i32::abs);
                    sorted_shift.sort_unstable();
                    found.entry((period, sorted_shift)).or_insert(Glider {
                        shape_name,
                        rule,
                        period,
                        shift,
                        layout,
                    });
                }
            }
            found.into_values()
        })
        .collect();
    let mut gliders = gliders;
    gliders.sort_by_key(|glider| std::cmp::Reverse(layers(&glider.layout)));
    for glider in &gliders {
        let rule = glider.rule;
        println!(
            "layers {:?} {} S{}-{} B{}-{} F{}: period {}, shift {:?}, {} polycubes, cells {:?}",
            layers(&glider.layout),
            glider.shape_name,
            rule.fewest_neighbours_to_survive,
            rule.most_neighbours_to_survive,
            rule.fewest_neighbours_to_be_born,
            rule.most_neighbours_to_be_born,
            rule.fewest_faces_shared_to_be_born,
            glider.period,
            glider.shift,
            glider.layout.len(),
            glider.layout
        );
    }
    println!("{} solid gliders", gliders.len());
}
