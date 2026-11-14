use std::collections::HashSet;

use polycubes_core::Coordinates;
use polycubes_sampling::{exposed_face_count, most_looped_polycubes};

/// Fixed polycubes with the most loops, sizes 1 to 8: last non-zero entries
/// of the triangle by number of loops.
const KNOWN_FIXED_COUNTS: [u64; 8] = [1, 3, 15, 3, 48, 18, 8, 1];
const KNOWN_LOOP_COUNTS: [usize; 8] = [0, 0, 0, 1, 1, 2, 3, 5];

#[test]
fn finds_as_many_fixed_polycubes_as_the_triangle_says() {
    for (size_index, &known_fixed_count) in KNOWN_FIXED_COUNTS.iter().enumerate() {
        let most_looped = most_looped_polycubes(size_index + 1);

        let fixed_count: u64 = most_looped
            .shapes
            .iter()
            .map(|shape| shape.orientation_count)
            .sum();
        assert_eq!(fixed_count, known_fixed_count, "{} cells", size_index + 1);
        assert_eq!(most_looped.loop_count, KNOWN_LOOP_COUNTS[size_index]);
    }
}

#[test]
fn the_square_is_the_only_tetracube_with_a_loop() {
    let most_looped = most_looped_polycubes(4);

    assert_eq!(most_looped.shapes.len(), 1);
    assert_eq!(
        most_looped.shapes[0].cells,
        vec![[0, 0, 0], [0, 0, 1], [0, 1, 0], [0, 1, 1]]
    );
    assert_eq!(most_looped.shapes[0].orientation_count, 3);
}

#[test]
fn the_cube_is_the_only_shape_of_eight_cells_with_five_loops() {
    let most_looped = most_looped_polycubes(8);

    assert_eq!(most_looped.shapes.len(), 1);
    assert_eq!(most_looped.shapes[0].orientation_count, 1);
    let cells: HashSet<Coordinates> = most_looped.shapes[0].cells.iter().copied().collect();
    for corner in 0..8 {
        assert!(cells.contains(&[corner & 1, (corner >> 1) & 1, (corner >> 2) & 1]));
    }
}

#[test]
fn every_shape_shows_the_fewest_faces_of_its_size() {
    for cell_count in 1..=7 {
        let most_looped = most_looped_polycubes(cell_count);

        let expected_face_count = 4 * cell_count + 2 - 2 * most_looped.loop_count;
        for shape in &most_looped.shapes {
            assert_eq!(shape.cells.len(), cell_count);
            assert_eq!(exposed_face_count(&shape.cells), expected_face_count);
        }
    }
}

#[test]
fn a_shape_turned_around_is_listed_once() {
    // The straight and the bent tricube, whatever their direction.
    let most_looped = most_looped_polycubes(3);

    assert_eq!(most_looped.shapes.len(), 2);
    let orientation_counts: Vec<u64> = most_looped
        .shapes
        .iter()
        .map(|shape| shape.orientation_count)
        .collect();
    assert_eq!(orientation_counts.iter().sum::<u64>(), 15);
    assert!(orientation_counts.contains(&3) && orientation_counts.contains(&12));
}
