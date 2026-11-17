//! Sweeps Conway-like rules for whole polycubes over random soups and tells
//! how each game ends, into a CSV file: one line per rule and soup.
//!
//! A game ends when the living come back as they were before: in place
//! (`still` for a period of one, `oscillates` otherwise) or moved
//! (`travels`, a glider or a group of them). It may also die out, exceed
//! the population cap (`explodes`) or still run after the last generation.
//!
//! Usage: `cargo run --release -p polycubes-automaton --example conway_sweep -- <output.csv>`

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

use polycubes_automaton::{ConwayRule, LivingPolycube, Soup, Torus, play_conway, soup};
use polycubes_core::Coordinates;
use rayon::prelude::*;

const GENERATION_COUNT: usize = 300;
const POPULATION_CAP: usize = 1500;
const SOUPS_PER_RULE: u64 = 20;

/// Wide enough that nothing born of a soup comes back round within the game,
/// as in an endless world.
const TORUS: Torus = Torus { side: 1000 };

const SOUP: Soup = Soup {
    side: 8,
    attempt_count: 40,
};

const SHAPES: [(&str, &[Coordinates]); 3] = [
    ("domino", &[[0, 0, 0], [1, 0, 0]]),
    ("I3", &[[0, 0, 0], [1, 0, 0], [2, 0, 0]]),
    ("L3", &[[0, 0, 0], [1, 0, 0], [1, 1, 0]]),
];

const SURVIVALS: [(usize, usize); 4] = [(1, 2), (1, 3), (2, 3), (2, 4)];
const BIRTHS: [(usize, usize); 4] = [(1, 1), (2, 2), (3, 3), (2, 3)];
const FACES_SHARED: [usize; 4] = [2, 3, 4, 5];

/// The living with their lowest corner at the origin, sorted, and where that
/// corner stood.
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

/// How a game ended: its name, its period, how far it moved per period and
/// its largest population.
struct Ending {
    name: &'static str,
    period: usize,
    shift: Coordinates,
    peak: usize,
}

fn ending_of(generations: &[Vec<LivingPolycube>]) -> Ending {
    let peak = generations.iter().map(Vec::len).max().unwrap_or(0);
    let ending = |name, period, shift| Ending {
        name,
        period,
        shift,
        peak,
    };
    let mut seen: HashMap<Vec<Vec<Coordinates>>, (usize, Coordinates)> = HashMap::new();
    for (generation, living) in generations.iter().enumerate() {
        if living.is_empty() {
            return ending("dies out", 0, [0; 3]);
        }
        if living.len() > POPULATION_CAP {
            return ending("explodes", 0, [0; 3]);
        }
        let (layout, corner) = moved_to_origin(living);
        if let Some(&(earlier, earlier_corner)) = seen.get(&layout) {
            let period = generation - earlier;
            let shift = [0, 1, 2].map(|axis| corner[axis] - earlier_corner[axis]);
            let name = match (period, shift == [0; 3]) {
                (_, false) => "travels",
                (1, true) => "still",
                (_, true) => "oscillates",
            };
            return ending(name, period, shift);
        }
        seen.insert(layout, (generation, corner));
    }
    ending("runs", 0, [0; 3])
}

/// Choices that look random but are the same at every run of a soup.
fn seeded_choice(soup_seed: u64) -> impl FnMut(usize) -> usize {
    let mut state = soup_seed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    move |choice_count| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).expect("31 bits fit") % choice_count
    }
}

/// The soup moved to the middle of the torus.
fn centred_soup(shape: &[Coordinates], soup_seed: u64) -> Vec<Vec<Coordinates>> {
    let middle = TORUS.side / 2;
    soup(shape, SOUP, seeded_choice(soup_seed))
        .into_iter()
        .map(|cells| {
            cells
                .into_iter()
                .map(|cell| cell.map(|coordinate| coordinate + middle))
                .collect()
        })
        .collect()
}

/// Plays up to the cap, a step at a time, so that an exploding game stops
/// early.
fn played(start: &[Vec<Coordinates>], rule: ConwayRule) -> Vec<Vec<LivingPolycube>> {
    let mut generations = play_conway(start, rule, TORUS, 0);
    for _ in 0..GENERATION_COUNT {
        let Some(living) = generations.last() else {
            break;
        };
        if living.is_empty() || living.len() > POPULATION_CAP {
            break;
        }
        let cells: Vec<Vec<Coordinates>> = living
            .iter()
            .map(|polycube| polycube.cells.clone())
            .collect();
        let next = play_conway(&cells, rule, TORUS, 1)
            .pop()
            .unwrap_or_default();
        generations.push(next);
    }
    generations
}

fn main() -> std::io::Result<()> {
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "conway_sweep.csv".to_owned());
    let mut games = Vec::new();
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
                    for soup_seed in 1..=SOUPS_PER_RULE {
                        games.push((shape_name, shape, rule, soup_seed));
                    }
                }
            }
        }
    }
    let lines: Vec<String> = games
        .par_iter()
        .map(|&(shape_name, shape, rule, soup_seed)| {
            let ending = ending_of(&played(&centred_soup(shape, soup_seed), rule));
            let [dx, dy, dz] = ending.shift;
            format!(
                "{shape_name},{}-{},{}-{},{},{soup_seed},{},{},{dx} {dy} {dz},{}",
                rule.fewest_neighbours_to_survive,
                rule.most_neighbours_to_survive,
                rule.fewest_neighbours_to_be_born,
                rule.most_neighbours_to_be_born,
                rule.fewest_faces_shared_to_be_born,
                ending.name,
                ending.period,
                ending.peak
            )
        })
        .collect();
    let mut output = BufWriter::new(File::create(output_path)?);
    writeln!(
        output,
        "shape,survival,birth,fewest_faces_shared,soup,ending,period,shift,peak"
    )?;
    for line in lines {
        writeln!(output, "{line}")?;
    }
    output.flush()
}
