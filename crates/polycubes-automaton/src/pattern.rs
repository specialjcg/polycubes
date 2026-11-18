//! Tools to tell patterns of the Conway-like life apart: where they stand,
//! how thick they are, how they behave, which way they are turned, and
//! seeded choices to pour the starts they come from.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use polycubes_core::Coordinates;

use crate::conway_life::{ConwayRule, Torus, play_conway};
use crate::polycube_life::LivingPolycube;

/// Wide enough that nothing watched comes back round.
const WATCH_TORUS: Torus = Torus { side: 1000 };
const WATCH_MIDDLE: i32 = 500;

/// A rotation of the cube as the rows of its matrix.
type Rotation = [Coordinates; 3];

const IDENTITY: Rotation = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];

/// Quarter turns about two axes: together they reach all 24 rotations of
/// the cube, and no mirror image.
const QUARTER_TURNS: [Rotation; 2] = [
    [[1, 0, 0], [0, 0, -1], [0, 1, 0]],
    [[0, 0, 1], [0, 1, 0], [-1, 0, 0]],
];

static ROTATIONS: LazyLock<Vec<Rotation>> = LazyLock::new(|| {
    let mut rotations = vec![IDENTITY];
    let mut composed_count = 0;
    while composed_count < rotations.len() {
        for quarter_turn in QUARTER_TURNS {
            let rotation = composed(quarter_turn, rotations[composed_count]);
            if !rotations.contains(&rotation) {
                rotations.push(rotation);
            }
        }
        composed_count += 1;
    }
    rotations
});

fn composed(outer: Rotation, inner: Rotation) -> Rotation {
    outer
        .map(|row| [0, 1, 2].map(|column| (0..3).map(|step| row[step] * inner[step][column]).sum()))
}

fn turned(rotation: &Rotation, cell: Coordinates) -> Coordinates {
    rotation.map(|row| row[0] * cell[0] + row[1] * cell[1] + row[2] * cell[2])
}

fn turned_layout(rotation: &Rotation, polycubes: &[Vec<Coordinates>]) -> Layout {
    let turned_polycubes: Vec<Vec<Coordinates>> = polycubes
        .iter()
        .map(|cells| cells.iter().map(|&cell| turned(rotation, cell)).collect())
        .collect();
    layout_of(turned_polycubes.iter().map(Vec::as_slice)).0
}

/// The same layout for a pattern however it is moved or turned, never
/// mirrored: the least of the layouts of its 24 rotations.
#[must_use]
pub fn canonical_form(polycubes: &[Vec<Coordinates>]) -> Layout {
    ROTATIONS
        .iter()
        .map(|rotation| turned_layout(rotation, polycubes))
        .min()
        .unwrap_or_default()
}

/// Number of the 24 rotations of the cube that leave the pattern as it
/// was, once moved back: 1 for a pattern with no symmetry, 24 for a cube.
#[must_use]
pub fn symmetry(polycubes: &[Vec<Coordinates>]) -> usize {
    let layout = turned_layout(&IDENTITY, polycubes);
    ROTATIONS
        .iter()
        .filter(|rotation| turned_layout(rotation, polycubes) == layout)
        .count()
}

/// How a pattern behaves once it comes back as it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nature {
    /// Back as it was at the next generation, in place.
    Still,
    /// Back in place after a few generations.
    Oscillator,
    /// Back as it was, moved.
    Glider,
}

/// How long a pattern is watched, and the population past which it is
/// given up on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Watch {
    pub generation_count: usize,
    pub population_cap: usize,
}

/// What a pattern turned into once it came back as it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    pub nature: Nature,
    /// Generations between two returns.
    pub period: usize,
    /// How far the pattern moved per period.
    pub shift: Coordinates,
    /// Layers it spreads over along each axis, fewest first.
    pub layers: [usize; 3],
    /// Rotations that leave it as it was, from 1 to 24.
    pub symmetry: usize,
    /// The pattern as it came back, its lowest corner at the origin.
    pub layout: Layout,
}

impl Classification {
    fn of(layout: Layout, period: usize, shift: Coordinates) -> Self {
        let nature = match (shift == [0; 3], period) {
            (false, _) => Nature::Glider,
            (true, 1) => Nature::Still,
            (true, _) => Nature::Oscillator,
        };
        Self {
            nature,
            period,
            shift,
            layers: layers(&layout),
            symmetry: symmetry(&layout),
            layout,
        }
    }
}

/// Plays `start` under `rule`, in an endless world, until it comes back as
/// it was before, moved or not, and tells what it became; nothing if it
/// dies out, outgrows the cap or never comes back within the watch.
#[must_use]
pub fn classify(
    start: &[Vec<Coordinates>],
    rule: ConwayRule,
    watch: Watch,
) -> Option<Classification> {
    let mut polycubes: Vec<Vec<Coordinates>> = start
        .iter()
        .map(|cells| {
            cells
                .iter()
                .map(|cell| cell.map(|coordinate| coordinate + WATCH_MIDDLE))
                .collect()
        })
        .collect();
    let mut seen: HashMap<Layout, (usize, Coordinates)> = HashMap::new();
    for generation in 0..=watch.generation_count {
        if polycubes.is_empty() || polycubes.len() > watch.population_cap {
            return None;
        }
        let (layout, corner) = layout_of(polycubes.iter().map(Vec::as_slice));
        if let Some(&(earlier, earlier_corner)) = seen.get(&layout) {
            let shift = [0, 1, 2].map(|axis| corner[axis] - earlier_corner[axis]);
            return Some(Classification::of(layout, generation - earlier, shift));
        }
        seen.insert(layout, (generation, corner));
        polycubes = play_conway(&polycubes, rule, WATCH_TORUS, 1)
            .pop()
            .unwrap_or_default()
            .into_iter()
            .map(|polycube| polycube.cells)
            .collect();
    }
    None
}

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
