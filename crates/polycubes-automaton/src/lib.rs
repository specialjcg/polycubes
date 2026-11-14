//! Cellular automata on the cubic lattice, seeded with polycubes.

mod aggregation;
mod budding;
mod census;
mod colony;
mod evolution;
mod polycube_life;
mod rule;

pub use aggregation::aggregate;
pub use budding::{Lineage, canonical_shape, grow_by_budding, grow_by_mutating_buds};
pub use census::{Census, census};
pub use colony::grow_colony;
pub use evolution::{Evolution, Fate, Limits, evolve, next_generation};
pub use polycube_life::{LivingPolycube, PolycubeLifeRule, PolycubeWorld, live_as_polycubes};
pub use rule::{Neighbourhood, Rule, RuleError};
