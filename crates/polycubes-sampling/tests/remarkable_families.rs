use std::collections::HashSet;

use polycubes_core::Coordinates;
use polycubes_sampling::{
    exposed_face_count, most_looped_square_free_polycubes, tightest_tree_polycubes,
};

fn holds_a_square(cells: &[Coordinates]) -> bool {
    let occupied: HashSet<Coordinates> = cells.iter().copied().collect();
    let planes = [(0, 1), (0, 2), (1, 2)];
    cells.iter().any(|&cell| {
        planes.iter().any(|&(first_axis, second_axis)| {
            let mut beside = cell;
            beside[first_axis] += 1;
            let mut above = cell;
            above[second_axis] += 1;
            let mut across = beside;
            across[second_axis] += 1;
            [beside, above, across]
                .iter()
                .all(|corner| occupied.contains(corner))
        })
    })
}

#[test]
fn no_loop_avoids_the_square_below_six_cells() {
    for cell_count in 1..=5 {
        assert_eq!(most_looped_square_free_polycubes(cell_count), None);
    }
}

#[test]
fn the_smallest_loop_without_square_is_the_cube_less_two_opposite_corners() {
    let most_looped = most_looped_square_free_polycubes(6).expect("six cells go around a corner");

    assert_eq!(most_looped.loop_count, 1);
    assert_eq!(most_looped.shapes.len(), 1);
    // One per diagonal of the cube.
    assert_eq!(most_looped.shapes[0].orientation_count, 4);
    let cells: HashSet<Coordinates> = most_looped.shapes[0].cells.iter().copied().collect();
    let missing_corners: Vec<Coordinates> = (0..8)
        .map(|corner| [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1])
        .filter(|corner| !cells.contains(corner))
        .collect();
    assert_eq!(missing_corners.len(), 2);
    let [first, second] = [missing_corners[0], missing_corners[1]];
    assert_eq!([0, 1, 2].map(|axis| first[axis] + second[axis]), [1, 1, 1]);
}

#[test]
fn loops_without_square_hold_no_square() {
    for cell_count in 6..=8 {
        let most_looped =
            most_looped_square_free_polycubes(cell_count).expect("a loop without square exists");

        assert!(most_looped.loop_count >= 1);
        let expected_face_count = 4 * cell_count + 2 - 2 * most_looped.loop_count;
        for shape in &most_looped.shapes {
            assert!(!holds_a_square(&shape.cells), "{:?}", shape.cells);
            assert_eq!(exposed_face_count(&shape.cells), expected_face_count);
        }
    }
}

#[test]
fn two_cubes_make_a_single_tree() {
    let tightest = tightest_tree_polycubes(2);

    assert_eq!(tightest.bounding_box, [1, 1, 2]);
    assert_eq!(tightest.shapes.len(), 1);
    assert_eq!(tightest.shapes[0].orientation_count, 3);
}

#[test]
fn the_tightest_trees_of_four_cells_are_the_three_that_leave_the_plane() {
    let tightest = tightest_tree_polycubes(4);

    assert_eq!(tightest.bounding_box, [2, 2, 2]);
    let mut orientation_counts: Vec<u64> = tightest
        .shapes
        .iter()
        .map(|shape| shape.orientation_count)
        .collect();
    orientation_counts.sort_unstable();
    // The corner and the two mirrored screws.
    assert_eq!(orientation_counts, vec![8, 12, 12]);
}

#[test]
fn tightest_trees_have_no_loop() {
    for cell_count in 1..=8 {
        let tightest = tightest_tree_polycubes(cell_count);

        assert!(!tightest.shapes.is_empty());
        for shape in &tightest.shapes {
            assert_eq!(exposed_face_count(&shape.cells), 4 * cell_count + 2);
        }
    }
}
