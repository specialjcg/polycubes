//! One long game of the life of polycubes, with mutation: which shapes
//! live, period after period, into a CSV file.
//!
//! Each line gives, for one period and one shape, the share of the living
//! polycubes that had this shape, averaged over the generations of the
//! period. The rule and the world are those of the Automaton page.
//!
//! Usage: `cargo run --release -p polycubes-automaton --example shape_shares -- <generation count> <output.csv>`

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};

use polycubes_automaton::{PolycubeLifeRule, PolycubeWorld, canonical_shape, live_as_polycubes};
use polycubes_core::Coordinates;
use polycubes_sampling::RankedPolycubes;

const SEED_CELL_COUNT: usize = 5;
const GENERATIONS_PER_PERIOD: usize = 25;
const BUDDING_CHANCE: f64 = 0.5;

/// Must match `LIFE_RULE` of the application.
const RULE: PolycubeLifeRule = PolycubeLifeRule {
    most_neighbours_to_survive: 2,
    fewest_neighbours_to_be_born: 1,
    most_neighbours_to_be_born: 2,
    mutation_one_in: 5,
};

/// The smallest world of the application: a cube of 11 cells a side.
const WORLD_REACH: i32 = 5;

/// The cells of a shape as text, one digit per coordinate: `000 100 110`.
fn shape_name(shape: &[Coordinates]) -> String {
    shape
        .iter()
        .map(|[x, y, z]| format!("{x}{y}{z}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Sides of the smallest box holding the shape, the shortest first.
fn box_sides(shape: &[Coordinates]) -> [i32; 3] {
    let mut sides =
        [0, 1, 2].map(|axis| shape.iter().map(|cell| cell[axis]).max().unwrap_or(0) + 1);
    sides.sort_unstable();
    sides
}

/// Pairs of cells sharing a face, minus what a tree would have.
fn loop_count(shape: &[Coordinates]) -> usize {
    let contact_count = shape
        .iter()
        .enumerate()
        .flat_map(|(rank, cell)| {
            shape[rank + 1..].iter().map(move |other| {
                (0..3)
                    .map(|axis| (cell[axis] - other[axis]).abs())
                    .sum::<i32>()
            })
        })
        .filter(|&distance| distance == 1)
        .count();
    contact_count + 1 - shape.len()
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let generation_count: usize = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .expect("first argument: generation count");
    let output_path = arguments.next().expect("second argument: output CSV path");

    let mut ranked_polycubes = RankedPolycubes::new(SEED_CELL_COUNT);
    let seed = ranked_polycubes
        .polycube_at(rand::random_range(0..ranked_polycubes.count()))
        .expect("a rank below the count names a polycube");
    let world = PolycubeWorld {
        generation_count,
        reach: WORLD_REACH,
    };
    let generations = live_as_polycubes(
        &seed,
        RULE,
        world,
        || rand::random_bool(BUDDING_CHANCE),
        |choice_count| rand::random_range(0..choice_count),
    );

    let mut output =
        BufWriter::new(File::create(&output_path).expect("output file can be created"));
    writeln!(
        output,
        "period_start,shape,box_side_1,box_side_2,box_side_3,loops,share,mean_polycubes"
    )
    .expect("output file can be written");
    for (period, living_by_generation) in generations.chunks(GENERATIONS_PER_PERIOD).enumerate() {
        let mut count_by_shape: BTreeMap<Vec<Coordinates>, usize> = BTreeMap::new();
        let mut polycube_count = 0;
        for polycube in living_by_generation.iter().flatten() {
            *count_by_shape
                .entry(canonical_shape(&polycube.cells))
                .or_insert(0) += 1;
            polycube_count += 1;
        }
        for (shape, count) in &count_by_shape {
            let [shortest, middle, longest] = box_sides(shape);
            writeln!(
                output,
                "{},{},{shortest},{middle},{longest},{},{:.5},{:.1}",
                period * GENERATIONS_PER_PERIOD,
                shape_name(shape),
                loop_count(shape),
                *count as f64 / polycube_count as f64,
                polycube_count as f64 / living_by_generation.len() as f64
            )
            .expect("output file can be written");
        }
    }
    println!(
        "seed [{}], {} generations, {} polycubes alive at the end",
        shape_name(&canonical_shape(&seed)),
        generation_count,
        generations.last().map_or(0, Vec::len)
    );
}
