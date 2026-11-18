//! Tools to tell patterns of the Conway-like life apart: where they stand,
//! how thick they are, and seeded choices to pour the starts they come from.

use std::collections::BTreeSet;

use polycubes_core::Coordinates;

use crate::polycube_life::LivingPolycube;

/// The polycubes of a pattern, each one's cells sorted, sorted themselves.
pub type Layout = Vec<Vec<Coordinates>>;

/// The living moved so that their lowest corner is the origin, as a layout,
/// and where that corner stood: two generations alike but for a move have
/// the same layout.
#[must_use]
pub fn moved_to_origin(living: &[LivingPolycube]) -> (Layout, Coordinates) {
    layout_of(living.iter().map(|polycube| polycube.cells.as_slice()))
}

pub(crate) fn layout_of<'cells>(
    polycubes: impl Iterator<Item = &'cells [Coordinates]> + Clone,
) -> (Layout, Coordinates) {
    let lowest_corner = [0, 1, 2].map(|axis| {
        polycubes
            .clone()
            .flatten()
            .map(|cell| cell[axis])
            .min()
            .unwrap_or(0)
    });
    let mut layout: Layout = polycubes
        .map(|cells| {
            let mut moved: Vec<Coordinates> = cells
                .iter()
                .map(|cell| [0, 1, 2].map(|axis| cell[axis] - lowest_corner[axis]))
                .collect();
            moved.sort_unstable();
            moved
        })
        .collect();
    layout.sort_unstable();
    (layout, lowest_corner)
}

/// Number of layers the pattern spreads over along each axis, fewest first.
#[must_use]
pub fn layers(layout: &[Vec<Coordinates>]) -> [usize; 3] {
    let mut counts = [0, 1, 2].map(|axis| {
        layout
            .iter()
            .flatten()
            .map(|cell| cell[axis])
            .collect::<BTreeSet<i32>>()
            .len()
    });
    counts.sort_unstable();
    counts
}

/// At least two layers along every axis: not a flat pattern.
#[must_use]
pub fn is_solid(layout: &[Vec<Coordinates>]) -> bool {
    layers(layout)[0] >= 2
}

/// Choices that look random but are the same at every run with the same
/// `seed` and `salt`: each call answers an index below the number of
/// choices it is given.
pub fn seeded_choice(seed: u64, salt: u64) -> impl FnMut(usize) -> usize {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ salt;
    move |choice_count| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).unwrap_or(usize::MAX) % choice_count
    }
}
