//! Cellular automata on the cubic lattice, seeded with polycubes.

mod aggregation;
mod budding;
mod census;
mod colony;
mod conway_life;
mod evolution;
mod fractal;
mod pattern;
mod polycube_life;
mod rule;

pub use aggregation::aggregate;
pub use budding::{Lineage, canonical_shape, grow_by_budding, grow_by_mutating_buds};
pub use census::{Census, census};
pub use colony::grow_colony;
pub use conway_life::{
    ConwayRule, DOMINO, DOMINO_GLIDER, Glider, SCREW_GLIDER, SQUARE_GLIDER, Soup,
    THICK_SQUARE_GLIDER, Torus, play_conway, soup,
};
pub use evolution::{Evolution, Fate, Limits, evolve, next_generation};
pub use fractal::growth_dimension;
pub use pattern::{
    Classification, Layout, Nature, Watch, canonical_form, classify, is_solid, layers,
    moved_to_origin, seeded_choice, symmetry,
};
pub use polycube_life::{LivingPolycube, PolycubeLifeRule, PolycubeWorld, live_as_polycubes};
pub use rule::{Neighbourhood, Rule, RuleError};
