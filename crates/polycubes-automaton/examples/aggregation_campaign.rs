//! Many aggregates grown from pentacubes drawn uniformly, measured one by
//! one into a CSV file: the fractal dimension of the growth, its loops and
//! its exposed faces.
//!
//! Usage: `cargo run --release -p polycubes-automaton --example aggregation_campaign -- <aggregate count> <output.csv>`

use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

use polycubes_automaton::{aggregate, canonical_shape, growth_dimension};
use polycubes_core::Coordinates;
use polycubes_sampling::{RankedPolycubes, exposed_face_count};
use rayon::prelude::*;

const SEED_CELL_COUNT: usize = 5;
const POLYCUBES_PER_AGGREGATE: usize = 120;
const AGGREGATES_BETWEEN_PROGRESS_LINES: usize = 500;

struct Measure {
    seed_shape: String,
    seed_loop_count: usize,
    cell_count: usize,
    dimension: f64,
    loop_count: usize,
    exposed_face_count: usize,
    box_sides: [i32; 3],
}

/// Pairs of cells sharing a face, minus what a tree would have: each loop
/// hides two faces more.
fn loop_count(cells: &[Coordinates]) -> usize {
    let contact_count = (6 * cells.len() - exposed_face_count(cells)) / 2;
    contact_count + 1 - cells.len()
}

fn sorted_box_sides(cells: &[Coordinates]) -> [i32; 3] {
    let mut sides = [0, 1, 2].map(|axis| {
        let along_axis = cells.iter().map(|cell| cell[axis]);
        along_axis.clone().max().unwrap_or(0) - along_axis.min().unwrap_or(0) + 1
    });
    sides.sort_unstable();
    sides
}

/// The cells of a shape as text, one digit per coordinate: `000 100 110`.
fn shape_name(cells: &[Coordinates]) -> String {
    canonical_shape(cells)
        .iter()
        .map(|[x, y, z]| format!("{x}{y}{z}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pentacubes drawn uniformly among the fixed ones, one per aggregate.
fn draw_seeds(seed_count: usize) -> Vec<Vec<Coordinates>> {
    let mut ranked_polycubes = RankedPolycubes::new(SEED_CELL_COUNT);
    let polycube_count = ranked_polycubes.count();
    (0..seed_count)
        .map(|_| {
            ranked_polycubes
                .polycube_at(rand::random_range(0..polycube_count))
                .expect("a rank below the count names a polycube")
        })
        .collect()
}

fn grow_and_measure(seed: &[Coordinates]) -> Measure {
    let polycubes = aggregate(seed, POLYCUBES_PER_AGGREGATE, |choice_count| {
        rand::random_range(0..choice_count)
    });
    let cells = polycubes.concat();
    let distinct_cells: HashSet<&Coordinates> = cells.iter().collect();
    assert_eq!(distinct_cells.len(), cells.len(), "two polycubes overlap");
    Measure {
        seed_shape: shape_name(seed),
        seed_loop_count: loop_count(seed),
        cell_count: cells.len(),
        dimension: growth_dimension(&cells).expect("an aggregate large enough to measure"),
        loop_count: loop_count(&cells),
        exposed_face_count: exposed_face_count(&cells),
        box_sides: sorted_box_sides(&cells),
    }
}

fn mean_and_deviation(values: &[f64]) -> (f64, f64) {
    let value_count = values.len() as f64;
    let mean = values.iter().sum::<f64>() / value_count;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / value_count;
    (mean, variance.sqrt())
}

fn print_summary(measures: &[Measure]) {
    let mut dimensions: Vec<f64> = measures.iter().map(|measure| measure.dimension).collect();
    dimensions.sort_by(f64::total_cmp);
    let (mean, deviation) = mean_and_deviation(&dimensions);
    let quantile = |share: f64| dimensions[((dimensions.len() - 1) as f64 * share) as usize];
    println!("aggregates: {}", measures.len());
    println!(
        "dimension: mean {mean:.3}, deviation {deviation:.3}, min {:.3}, 5% {:.3}, median {:.3}, 95% {:.3}, max {:.3}",
        dimensions[0],
        quantile(0.05),
        quantile(0.5),
        quantile(0.95),
        dimensions[dimensions.len() - 1]
    );
    let loops: Vec<f64> = measures
        .iter()
        .map(|measure| measure.loop_count as f64)
        .collect();
    let faces: Vec<f64> = measures
        .iter()
        .map(|measure| measure.exposed_face_count as f64 / measure.cell_count as f64)
        .collect();
    let longest_sides: Vec<f64> = measures
        .iter()
        .map(|measure| f64::from(measure.box_sides[2]))
        .collect();
    println!(
        "loops: mean {:.1}, deviation {:.1}",
        mean_and_deviation(&loops).0,
        mean_and_deviation(&loops).1
    );
    println!(
        "exposed faces per cell: mean {:.3}, deviation {:.3}",
        mean_and_deviation(&faces).0,
        mean_and_deviation(&faces).1
    );
    println!(
        "longest box side: mean {:.1}, deviation {:.1}",
        mean_and_deviation(&longest_sides).0,
        mean_and_deviation(&longest_sides).1
    );

    let mut dimensions_by_seed: BTreeMap<(usize, &str), Vec<f64>> = BTreeMap::new();
    for measure in measures {
        dimensions_by_seed
            .entry((measure.seed_loop_count, &measure.seed_shape))
            .or_default()
            .push(measure.dimension);
    }
    println!("by seed shape (loops, cells): aggregates, mean dimension, deviation");
    for ((seed_loop_count, seed_shape), seed_dimensions) in &dimensions_by_seed {
        let (seed_mean, seed_deviation) = mean_and_deviation(seed_dimensions);
        println!(
            "  {seed_loop_count} [{seed_shape}]: {}, {seed_mean:.3}, {seed_deviation:.3}",
            seed_dimensions.len()
        );
    }
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let aggregate_count: usize = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .expect("first argument: aggregate count");
    let output_path = arguments.next().expect("second argument: output CSV path");

    let seeds = draw_seeds(aggregate_count);
    let done_count = AtomicUsize::new(0);
    let measures: Vec<Measure> = seeds
        .par_iter()
        .map(|seed| {
            let measure = grow_and_measure(seed);
            let done = done_count.fetch_add(1, Ordering::Relaxed) + 1;
            if done.is_multiple_of(AGGREGATES_BETWEEN_PROGRESS_LINES) {
                eprintln!("{done} / {aggregate_count}");
            }
            measure
        })
        .collect();

    let mut output =
        BufWriter::new(File::create(&output_path).expect("output file can be created"));
    writeln!(
        output,
        "aggregate,seed_shape,seed_loops,cells,dimension,loops,exposed_faces,box_side_1,box_side_2,box_side_3"
    )
    .expect("output file can be written");
    for (rank, measure) in measures.iter().enumerate() {
        writeln!(
            output,
            "{rank},{},{},{},{:.4},{},{},{},{},{}",
            measure.seed_shape,
            measure.seed_loop_count,
            measure.cell_count,
            measure.dimension,
            measure.loop_count,
            measure.exposed_face_count,
            measure.box_sides[0],
            measure.box_sides[1],
            measure.box_sides[2]
        )
        .expect("output file can be written");
    }
    print_summary(&measures);
}
