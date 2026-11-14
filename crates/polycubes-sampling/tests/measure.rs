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
