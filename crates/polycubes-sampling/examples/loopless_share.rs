//! How often a polycube has no loop, for uniform draws and for polycubes
//! grown by random neighbour. A polycube without loop shows `4n + 2` faces,
//! the most its size allows.
//!
//! The uniform side is exact: every fixed polycube is visited. The growth
//! side is sampled and comes with a 95 % Wilson interval.
//!
//! Usage: `cargo run --release -p polycubes-sampling --example loopless_share -- <largest cell count> <growth sample count>`

use polycubes_sampling::{exposed_face_count, exposed_face_distribution, grow_by_random_neighbour};

/// Normal quantile of a two-sided 95 % interval.
const CONFIDENCE_QUANTILE: f64 = 1.959_964;

struct Share {
    estimate: f64,
    lowest: f64,
    highest: f64,
}

/// Wilson score interval of a proportion.
fn wilson_share(success_count: u64, sample_count: u64) -> Share {
    let sample_size = sample_count as f64;
    let estimate = success_count as f64 / sample_size;
    let squared_quantile = CONFIDENCE_QUANTILE * CONFIDENCE_QUANTILE;
    let denominator = 1.0 + squared_quantile / sample_size;
    let centre = (estimate + squared_quantile / (2.0 * sample_size)) / denominator;
    let half_width = CONFIDENCE_QUANTILE
        * (estimate * (1.0 - estimate) / sample_size
            + squared_quantile / (4.0 * sample_size * sample_size))
            .sqrt()
        / denominator;
    Share {
        estimate,
        lowest: centre - half_width,
        highest: centre + half_width,
    }
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let largest_cell_count: usize = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .expect("first argument: largest cell count");
    let growth_sample_count: u64 = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .expect("second argument: growth sample count");

    println!(
        "cells | polycubes | loopless, uniform (exact) | mean faces, uniform | loopless, growth [95 %] | mean faces, growth"
    );
    for cell_count in 1..=largest_cell_count {
        let loopless_face_count = 4 * cell_count + 2;

        let distribution = exposed_face_distribution(cell_count);
        let polycube_count: u64 = distribution.values().sum();
        let loopless_count = distribution.get(&loopless_face_count).copied().unwrap_or(0);
        let uniform_face_total: u64 = distribution
            .iter()
            .map(|(&face_count, &count)| face_count as u64 * count)
            .sum();

        let mut grown_loopless_count = 0;
        let mut grown_face_total = 0;
        for _ in 0..growth_sample_count {
            let grown = grow_by_random_neighbour(cell_count, |candidate_count| {
                rand::random_range(0..candidate_count)
            });
            let face_count = exposed_face_count(&grown);
            grown_face_total += face_count as u64;
            grown_loopless_count += u64::from(face_count == loopless_face_count);
        }
        let grown_share = wilson_share(grown_loopless_count, growth_sample_count);

        println!(
            "{cell_count} | {polycube_count} | {:.4} | {:.3} | {:.4} [{:.4}, {:.4}] | {:.3}",
            loopless_count as f64 / polycube_count as f64,
            uniform_face_total as f64 / polycube_count as f64,
            grown_share.estimate,
            grown_share.lowest,
            grown_share.highest,
            grown_face_total as f64 / growth_sample_count as f64,
        );
    }
}
