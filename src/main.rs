use std::time::Instant;

use anyhow::{Context, Result};
use polycubes::count_fixed_polycubes_in_parallel;

const DEFAULT_MAX_CELL_COUNT: usize = 12;

fn main() -> Result<()> {
    let max_cell_count = parse_max_cell_count()?;

    let thread_count =
        std::thread::available_parallelism().context("cannot read available parallelism")?;

    let started_at = Instant::now();
    let counts = count_fixed_polycubes_in_parallel(max_cell_count, thread_count);
    let elapsed_seconds = started_at.elapsed().as_secs_f64();

    for (size_index, count) in counts.iter().enumerate() {
        println!("{:>2} {count}", size_index + 1);
    }
    let enumerated_total: u64 = counts.iter().sum();
    eprintln!(
        "{enumerated_total} polycubes in {elapsed_seconds:.2} s on {thread_count} threads ({:.1} M/s)",
        enumerated_total as f64 / elapsed_seconds / 1e6
    );
    Ok(())
}

fn parse_max_cell_count() -> Result<usize> {
    match std::env::args().nth(1) {
        Some(argument) => argument
            .parse()
            .with_context(|| format!("invalid maximum cell count: {argument}")),
        None => Ok(DEFAULT_MAX_CELL_COUNT),
    }
}
