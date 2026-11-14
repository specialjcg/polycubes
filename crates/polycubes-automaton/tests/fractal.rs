use polycubes_automaton::growth_dimension;
use polycubes_core::Coordinates;

/// Cells of a box around the origin, the nearest to it first: a growth
/// that fills space evenly in as many directions as the box is wide.
fn grown_outwards(reach: [i32; 3]) -> Vec<Coordinates> {
    let mut cells = Vec::new();
    for x in -reach[0]..=reach[0] {
        for y in -reach[1]..=reach[1] {
            for z in -reach[2]..=reach[2] {
                cells.push([x, y, z]);
            }
        }
    }
    cells.sort_by_key(|[x, y, z]| x * x + y * y + z * z);
    cells
}

fn assert_dimension_near(cells: &[Coordinates], expected: f64) {
    let dimension = growth_dimension(cells).expect("enough cells to measure");
    assert!(
        (dimension - expected).abs() < 0.15,
        "dimension {dimension}, expected about {expected}"
    );
}

#[test]
fn a_growth_that_fills_space_has_dimension_three() {
    assert_dimension_near(&grown_outwards([7, 7, 7]), 3.0);
}

#[test]
fn a_growth_that_fills_a_plane_has_dimension_two() {
    assert_dimension_near(&grown_outwards([20, 20, 0]), 2.0);
}

#[test]
fn a_growth_along_a_line_has_dimension_one() {
    assert_dimension_near(&grown_outwards([400, 0, 0]), 1.0);
}

#[test]
fn a_growth_too_small_to_measure_has_no_dimension() {
    assert_eq!(growth_dimension(&grown_outwards([1, 1, 1])), None);
}
