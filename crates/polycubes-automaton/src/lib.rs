//! Cellular automata on the cubic lattice, seeded with polycubes.

mod budding;
mod census;
mod colony;
mod evolution;
mod rule;

pub use budding::grow_by_budding;
pub use census::{Census, census};
pub use colony::grow_colony;
pub use evolution::{Evolution, Fate, Limits, evolve, next_generation};
pub use rule::{Neighbourhood, Rule, RuleError};
