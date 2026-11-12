use std::num::NonZeroUsize;

use polycubes::count_one_sided_polycubes;

/// OEIS A000162, sizes 1 to 10.
const KNOWN_ONE_SIDED_COUNTS: [u64; 10] = [1, 1, 2, 8, 29, 166, 1_023, 6_922, 48_311, 346_543];

#[test]
fn matches_oeis_a000162_for_every_max_size() {
    let thread_count = NonZeroUsize::new(4).expect("four is not zero");

    for max_cell_count in 0..=KNOWN_ONE_SIDED_COUNTS.len() {
        let counts = count_one_sided_polycubes(max_cell_count, thread_count);

        assert_eq!(counts, KNOWN_ONE_SIDED_COUNTS[..max_cell_count]);
    }
}
