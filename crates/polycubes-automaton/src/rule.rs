//! Which cells live on and which are born, from their living neighbours.

/// The cells that count as neighbours of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Neighbourhood {
    /// The 6 cells sharing a face: the contacts of a polycube.
    Faces,
    /// The 12 cells sharing an edge only.
    Edges,
    /// The 26 cells sharing a face, an edge or a corner.
    FacesEdgesCorners,
}

impl Neighbourhood {
    #[must_use]
    pub fn neighbour_count(self) -> usize {
        match self {
            Self::Faces => 6,
            Self::Edges => 12,
            Self::FacesEdgesCorners => 26,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RuleError {
    #[error("a cell has at most {neighbour_count} neighbours, got {count}")]
    CountOutOfRange {
        count: usize,
        neighbour_count: usize,
    },
    #[error("birth without any living neighbour would fill the whole lattice")]
    BirthWithoutNeighbour,
}

/// A living cell stays alive when its number of living neighbours is a
/// survival count; a dead cell comes alive when it is a birth count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub(crate) neighbourhood: Neighbourhood,
    /// Bit `count` is set when `count` living neighbours keep a cell alive.
    pub(crate) survival_counts: u32,
    pub(crate) birth_counts: u32,
}

impl Rule {
    /// # Errors
    ///
    /// Returns [`RuleError::CountOutOfRange`] if a count exceeds the number
    /// of neighbours, and [`RuleError::BirthWithoutNeighbour`] if zero is a
    /// birth count.
    pub fn new(
        neighbourhood: Neighbourhood,
        survival_counts: &[usize],
        birth_counts: &[usize],
    ) -> Result<Self, RuleError> {
        if birth_counts.contains(&0) {
            return Err(RuleError::BirthWithoutNeighbour);
        }
        Ok(Self {
            neighbourhood,
            survival_counts: count_set(neighbourhood, survival_counts)?,
            birth_counts: count_set(neighbourhood, birth_counts)?,
        })
    }

    #[must_use]
    pub fn neighbourhood(&self) -> Neighbourhood {
        self.neighbourhood
    }

    pub(crate) fn keeps_alive(&self, living_neighbour_count: usize) -> bool {
        self.survival_counts >> living_neighbour_count & 1 == 1
    }

    pub(crate) fn gives_birth(&self, living_neighbour_count: usize) -> bool {
        self.birth_counts >> living_neighbour_count & 1 == 1
    }
}

fn count_set(neighbourhood: Neighbourhood, counts: &[usize]) -> Result<u32, RuleError> {
    let neighbour_count = neighbourhood.neighbour_count();
    let mut count_set = 0;
    for &count in counts {
        if count > neighbour_count {
            return Err(RuleError::CountOutOfRange {
                count,
                neighbour_count,
            });
        }
        count_set |= 1 << count;
    }
    Ok(count_set)
}

impl Neighbourhood {
    /// Steps from a cell to each of its neighbours.
    pub(crate) fn steps(self) -> Vec<[i32; 3]> {
        let mut steps = Vec::with_capacity(self.neighbour_count());
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let changed_axis_count = [x, y, z].iter().filter(|&&step| step != 0).count();
                    let is_neighbour = match self {
                        Self::Faces => changed_axis_count == 1,
                        Self::Edges => changed_axis_count == 2,
                        Self::FacesEdgesCorners => changed_axis_count >= 1,
                    };
                    if is_neighbour {
                        steps.push([x, y, z]);
                    }
                }
            }
        }
        steps
    }
}
