use polycubes::count_fixed_polycubes;

/// OEIS A001931, sizes 1 to 10.
const KNOWN_FIXED_COUNTS: [u64; 10] = [
    1, 3, 15, 86, 534, 3_481, 23_502, 162_913, 1_152_870, 8_294_738,
];

#[test]
fn matches_oeis_a001931_up_to_size_10() {
    let counts = count_fixed_polycubes(KNOWN_FIXED_COUNTS.len());

    assert_eq!(counts, KNOWN_FIXED_COUNTS);
}

#[test]
fn returns_no_count_when_max_size_is_zero() {
    let counts = count_fixed_polycubes(0);

    assert!(counts.is_empty());
}

#[test]
fn counts_the_single_cube() {
    let counts = count_fixed_polycubes(1);

    assert_eq!(counts, [1]);
}
