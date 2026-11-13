//! Drawing polycubes: exact uniform sampling and biased growth processes.

mod growth;
mod ranked;

pub use growth::{bounding_box, grow_by_random_neighbour};
pub use ranked::RankedPolycubes;
