//! Draws the polycube of a given rank and reports how long it took.
//!
//! Usage: `cargo run --release -p polycubes-sampling --example draw -- <cell count> <rank>`

use std::time::Instant;

use polycubes_sampling::RankedPolycubes;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let cell_count: usize = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .expect("first argument: cell count");
    let rank: u64 = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .expect("second argument: rank");

    let started_at = Instant::now();
    let mut ranked_polycubes = RankedPolycubes::new(cell_count);
    let counting_seconds = started_at.elapsed().as_secs_f64();
    let polycube = ranked_polycubes.polycube_at(rank);
    let drawing_seconds = started_at.elapsed().as_secs_f64() - counting_seconds;

    println!(
        "{} polycubes of {cell_count} cells",
        ranked_polycubes.count()
    );
    println!("rank {rank}: {polycube:?}");
    eprintln!("counting: {counting_seconds:.3} s, drawing: {drawing_seconds:.3} s");
}
