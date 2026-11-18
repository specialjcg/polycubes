use polycubes_automaton::{LivingPolycube, is_solid, layers, moved_to_origin, seeded_choice};

fn living(polycubes: &[&[[i32; 3]]]) -> Vec<LivingPolycube> {
    polycubes
        .iter()
        .enumerate()
        .map(|(identity, cells)| LivingPolycube {
            identity,
            cells: cells.to_vec(),
        })
        .collect()
}

#[test]
fn moved_to_origin_puts_the_lowest_corner_at_the_origin_and_tells_where_it_stood() {
    let polycubes = living(&[&[[6, 4, 9], [5, 4, 9]], &[[5, 3, 8]]]);

    let (layout, corner) = moved_to_origin(&polycubes);

    assert_eq!(
        (layout, corner),
        (vec![vec![[0, 0, 0]], vec![[0, 1, 1], [1, 1, 1]]], [5, 3, 8])
    );
}

#[test]
fn layers_count_the_planes_crossed_along_each_axis_fewest_first() {
    let layout = vec![vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]], vec![[0, 1, 0]]];

    assert_eq!(layers(&layout), [1, 2, 3]);
}

#[test]
fn a_flat_pattern_is_not_solid() {
    let flat = vec![vec![[0, 0, 0], [1, 0, 0]], vec![[0, 1, 0]]];
    let solid = vec![vec![[0, 0, 0], [1, 0, 0]], vec![[0, 1, 1]]];

    assert_eq!((is_solid(&flat), is_solid(&solid)), (false, true));
}

#[test]
fn seeded_choices_repeat_with_the_seed_and_stay_below_the_count() {
    let draw = |seed| {
        let mut choose = seeded_choice(seed, 7);
        (0..50).map(|_| choose(5)).collect::<Vec<usize>>()
    };

    assert!(draw(3) == draw(3) && draw(3) != draw(4) && draw(3).iter().all(|&choice| choice < 5));
}
