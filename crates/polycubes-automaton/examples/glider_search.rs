//! Searches for gliders that are truly three-dimensional in the Conway-like
//! life of whole polycubes: patterns that come back as they were, moved,
//! and whose cells spread over at least two layers along every axis.
//!
//! Tiny random starts are tried for many shapes and rules; each glider found
//! is printed once per shape, rule, period and shift, with its cells.
//!
//! Usage: `cargo run --release -p polycubes-automaton --example glider_search -- <starts per rule> [first soup seed]`

mod search_space;

use std::collections::HashMap;

use polycubes_automaton::{ConwayRule, Torus, is_solid, layers, moved_to_origin, play_conway};
use polycubes_core::Coordinates;
use rayon::prelude::*;
use search_space::{tiny_start, trials};

const GENERATION_COUNT: usize = 60;
const POPULATION_CAP: usize = 60;

/// Wide enough that nothing comes back round within the game.
const TORUS: Torus = Torus { side: 1000 };
const MIDDLE: i32 = 500;

struct Glider {
    shape_name: &'static str,
    rule: ConwayRule,
    period: usize,
    shift: Coordinates,
    layout: Vec<Vec<Coordinates>>,
}

/// A tiny start in the middle of the torus.
fn centred_tiny_start(shape: &[Coordinates], seed: u64) -> Vec<Vec<Coordinates>> {
    tiny_start(shape, seed)
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
    let trials = trials();
    let gliders: Vec<Glider> = trials
        .par_iter()
        .flat_map_iter(|&(shape_name, shape, rule)| {
            let mut found: HashMap<(usize, Coordinates), Glider> = HashMap::new();
            for seed in first_seed..first_seed + starts_per_rule {
                if let Some((period, shift, layout)) =
                    glider_from(&centred_tiny_start(shape, seed), rule)
                {
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
