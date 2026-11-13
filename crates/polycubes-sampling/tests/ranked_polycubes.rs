use std::collections::BTreeSet;

use polycubes_core::Coordinates;
use polycubes_sampling::RankedPolycubes;

/// OEIS A001931, sizes 1 to 7.
const KNOWN_FIXED_COUNTS: [u64; 7] = [1, 3, 15, 86, 534, 3_481, 23_502];
/// Largest size whose polycubes are all drawn, to keep the tests fast.
const LARGEST_EXHAUSTIVE_SIZE: usize = 6;

fn all_ranked_polycubes(cell_count: usize) -> Vec<Vec<Coordinates>> {
    let mut ranked_polycubes = RankedPolycubes::new(cell_count);
    (0..ranked_polycubes.count())
        .map(|rank| {
            ranked_polycubes
                .polycube_at(rank)
                .expect("rank is below the count")
        })
        .collect()
}

fn is_face_connected(cells: &[Coordinates]) -> bool {
    let cell_set: BTreeSet<Coordinates> = cells.iter().copied().collect();
    let mut reached_cells = BTreeSet::from([cells[0]]);
    let mut frontier_cells = vec![cells[0]];
    while let Some([x, y, z]) = frontier_cells.pop() {
        let neighbours = [
            [x + 1, y, z],
            [x - 1, y, z],
            [x, y + 1, z],
            [x, y - 1, z],
            [x, y, z + 1],
            [x, y, z - 1],
        ];
        for neighbour in neighbours {
            if cell_set.contains(&neighbour) && reached_cells.insert(neighbour) {
                frontier_cells.push(neighbour);
            }
        }
    }
    reached_cells.len() == cell_set.len()
}

#[test]
fn counts_match_oeis_a001931() {
    for (size_index, &known_count) in KNOWN_FIXED_COUNTS.iter().enumerate() {
        let ranked_polycubes = RankedPolycubes::new(size_index + 1);

        assert_eq!(ranked_polycubes.count(), known_count);
    }
}

#[test]
fn every_rank_gives_a_different_polycube() {
    for cell_count in 1..=LARGEST_EXHAUSTIVE_SIZE {
        let polycubes = all_ranked_polycubes(cell_count);

        let distinct_polycubes: BTreeSet<BTreeSet<Coordinates>> = polycubes
            .iter()
            .map(|cells| cells.iter().copied().collect())
            .collect();
        assert_eq!(
            distinct_polycubes.len() as u64,
            KNOWN_FIXED_COUNTS[cell_count - 1]
        );
    }
}

#[test]
fn every_ranked_polycube_has_the_requested_size() {
    for cell_count in 1..=LARGEST_EXHAUSTIVE_SIZE {
        let polycubes = all_ranked_polycubes(cell_count);

        assert!(
            polycubes.iter().all(|cells| {
                cells.iter().copied().collect::<BTreeSet<_>>().len() == cell_count
            })
        );
    }
}

#[test]
fn every_ranked_polycube_is_face_connected() {
    for cell_count in 1..=LARGEST_EXHAUSTIVE_SIZE {
        let polycubes = all_ranked_polycubes(cell_count);

        assert!(polycubes.iter().all(|cells| is_face_connected(cells)));
    }
}

#[test]
fn every_ranked_polycube_has_its_smallest_cell_at_the_origin() {
    for cell_count in 1..=LARGEST_EXHAUSTIVE_SIZE {
        let polycubes = all_ranked_polycubes(cell_count);

        assert!(polycubes.iter().all(|cells| {
            let smallest_cell = cells.iter().map(|&[x, y, z]| [z, y, x]).min();
            smallest_cell == Some([0, 0, 0])
        }));
    }
}

#[test]
fn returns_nothing_beyond_the_last_rank() {
    let mut ranked_polycubes = RankedPolycubes::new(4);

    let polycube = ranked_polycubes.polycube_at(KNOWN_FIXED_COUNTS[3]);

    assert_eq!(polycube, None);
}
