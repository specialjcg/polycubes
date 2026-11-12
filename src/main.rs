use std::time::Instant;

use anyhow::{Context, Result};
use polycubes::{count_fixed_polycubes_in_parallel, one_sided_counts_from_fixed};

const DEFAULT_MAX_CELL_COUNT: usize = 12;

fn main() -> Result<()> {
    let max_cell_count = parse_max_cell_count()?;

    let thread_count =
        std::thread::available_parallelism().context("cannot read available parallelism")?;

    let started_at = Instant::now();
    let fixed_counts = count_fixed_polycubes_in_parallel(max_cell_count, thread_count);
    let fixed_seconds = started_at.elapsed().as_secs_f64();
    let one_sided_counts = one_sided_counts_from_fixed(&fixed_counts);
    let total_seconds = started_at.elapsed().as_secs_f64();

    println!("size fixed(A001931) one-sided(A000162)");
    for (size_index, (fixed_count, one_sided_count)) in
        fixed_counts.iter().zip(&one_sided_counts).enumerate()
    {
        println!("{:>2} {fixed_count} {one_sided_count}", size_index + 1);
    }
    eprintln!(
        "fixed: {fixed_seconds:.2} s on {thread_count} threads, symmetric: {:.2} s on 1 thread",
        total_seconds - fixed_seconds
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
