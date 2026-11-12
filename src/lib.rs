//! Enumeration of polycubes (face-connected sets of unit cubes).

mod redelmeier;

pub use redelmeier::{count_fixed_polycubes, count_fixed_polycubes_in_parallel};
