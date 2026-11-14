//! Cellular automata on the cubic lattice, seeded with polycubes.

mod census;
mod evolution;
mod rule;

pub use census::{Census, census};
pub use evolution::{Evolution, Fate, Limits, evolve, next_generation};
pub use rule::{Neighbourhood, Rule, RuleError};
