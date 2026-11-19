//! The shapes, rules and tiny starts that the searches for patterns try.

use polycubes_automaton::{ConwayRule, Soup, seeded_choice, soup};
use polycubes_core::Coordinates;

pub const SHAPES: [(&str, &[Coordinates]); 10] = [
    ("domino", &[[0, 0, 0], [1, 0, 0]]),
    ("I3", &[[0, 0, 0], [1, 0, 0], [2, 0, 0]]),
    ("L3", &[[0, 0, 0], [1, 0, 0], [1, 1, 0]]),
    ("O4", &[[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]]),
    ("T4", &[[0, 0, 0], [1, 0, 0], [2, 0, 0], [1, 1, 0]]),
    ("S4", &[[0, 0, 0], [1, 0, 0], [1, 1, 0], [2, 1, 0]]),
    ("L4", &[[0, 0, 0], [1, 0, 0], [2, 0, 0], [2, 1, 0]]),
    ("screw", &[[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, 1]]),
    (
        "mirror screw",
        &[[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 1, -1]],
    ),
    ("tripod", &[[0, 0, 0], [1, 0, 0], [0, 1, 0], [0, 0, 1]]),
];

const SURVIVALS: [(usize, usize); 5] = [(1, 2), (1, 3), (2, 3), (2, 4), (3, 4)];
const BIRTHS: [(usize, usize); 4] = [(1, 1), (2, 2), (3, 3), (2, 3)];
const FACES_SHARED: [usize; 5] = [2, 3, 4, 5, 6];

/// Mixed into the seeds so that tiny starts differ from the soups of
/// `conway_sweep`.
const SEED_SALT: u64 = 0x2545_f491_4f6c_dd1d;

/// Every shape under every rule, each with its name.
pub fn trials() -> Vec<(&'static str, &'static [Coordinates], ConwayRule)> {
    let mut trials = Vec::new();
    for (shape_name, shape) in SHAPES {
        for (fewest_to_survive, most_to_survive) in SURVIVALS {
            for (fewest_to_be_born, most_to_be_born) in BIRTHS {
                for fewest_faces in FACES_SHARED {
                    let rule = ConwayRule {
                        fewest_neighbours_to_survive: fewest_to_survive,
                        most_neighbours_to_survive: most_to_survive,
                        fewest_neighbours_to_be_born: fewest_to_be_born,
                        most_neighbours_to_be_born: most_to_be_born,
                        fewest_faces_shared_to_be_born: fewest_faces,
                    };
                    trials.push((shape_name, shape, rule));
                }
            }
        }
    }
    trials
}

/// A tiny start: a few polycubes poured in a box of 4 to 6 cells from the
/// origin.
pub fn tiny_start(shape: &[Coordinates], seed: u64) -> Vec<Vec<Coordinates>> {
    let mut choose = seeded_choice(seed, SEED_SALT);
    let side = 4 + i32::try_from(choose(3)).unwrap_or(0);
    let attempt_count = 3 + choose(8);
    soup(
        shape,
        Soup {
            side,
            attempt_count,
        },
        choose,
    )
}
