use polycubes_sampling::exposed_face_count;

#[test]
fn a_single_cube_shows_its_six_faces() {
    assert_eq!(exposed_face_count(&[[0, 0, 0]]), 6);
}

#[test]
fn each_contact_hides_two_faces() {
    let stick = [[0, 0, 0], [1, 0, 0], [2, 0, 0]];

    assert_eq!(exposed_face_count(&stick), 14);
}

#[test]
fn a_full_two_by_two_by_two_block_shows_twenty_four_faces() {
    let block: Vec<[i32; 3]> = (0..8)
        .map(|index| [index & 1, (index >> 1) & 1, (index >> 2) & 1])
        .collect();

    assert_eq!(exposed_face_count(&block), 24);
}

#[test]
fn does_not_depend_on_the_order_of_the_cells() {
    let bent = [[0, 0, 0], [0, 0, 1], [0, 1, 1], [1, 1, 1]];
    let mut reversed = bent;
    reversed.reverse();

    assert_eq!(exposed_face_count(&bent), 18);
    assert_eq!(exposed_face_count(&reversed), 18);
}

mod exact_distribution {
    use std::collections::BTreeMap;

    use polycubes_sampling::exposed_face_distribution;

    #[test]
    fn tetracubes_are_loopless_except_the_three_squares() {
        let distribution = exposed_face_distribution(4);

        assert_eq!(distribution, BTreeMap::from([(16, 3), (18, 83)]));
    }

    #[test]
    fn counts_every_fixed_polycube() {
        // OEIS A001931, sizes 1 to 6.
        let known_fixed_counts = [1, 3, 15, 86, 534, 3_481];

        for (size_index, known_count) in known_fixed_counts.into_iter().enumerate() {
            let distribution = exposed_face_distribution(size_index + 1);

            assert_eq!(distribution.values().sum::<u64>(), known_count);
        }
    }

    #[test]
    fn no_polycube_shows_more_faces_than_a_loopless_one() {
        let cell_count = 6;

        let distribution = exposed_face_distribution(cell_count);

        assert_eq!(distribution.keys().max(), Some(&(4 * cell_count + 2)));
    }
}
