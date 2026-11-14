//! Fixed polycubes counted by cell count and by number of independent loops.
//!
//! A polycube of `n` cells with `k` pairs of touching cells shows `6n - 2k`
//! faces and has `k - (n - 1)` independent loops, so a polycube without loop
//! shows `4n + 2` faces. Each printed row lists the counts for 0, 1, 2, …
//! loops; its first entry is OEIS A118356 and its sum is OEIS A001931.
//!
//! Usage: `cargo run --release -p polycubes-sampling --example cycle_triangle -- <largest cell count>`

use std::collections::BTreeMap;

use polycubes_sampling::exposed_face_distribution;

/// Counts by number of loops, from none to the most the size allows.
fn counts_by_loop_count(cell_count: usize, counts_by_faces: &BTreeMap<usize, u64>) -> Vec<u64> {
    let loopless_face_count = 4 * cell_count + 2;
    let smallest_face_count = counts_by_faces
        .keys()
        .next()
        .copied()
        .unwrap_or(loopless_face_count);
    // One more contact hides two faces, so face counts go down by twos.
    (smallest_face_count..=loopless_face_count)
        .rev()
        .step_by(2)
        .map(|face_count| counts_by_faces.get(&face_count).copied().unwrap_or(0))
        .collect()
}

fn main() {
    let largest_cell_count: usize = std::env::args()
        .nth(1)
        .and_then(|argument| argument.parse().ok())
        .expect("first argument: largest cell count");

    for cell_count in 1..=largest_cell_count {
        let row = counts_by_loop_count(cell_count, &exposed_face_distribution(cell_count));
        let entries: Vec<String> = row.iter().map(u64::to_string).collect();
        println!("{cell_count}: {}", entries.join(", "));
    }
}
