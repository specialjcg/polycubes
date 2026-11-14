//! Generations of an automaton and what becomes of a seed.

use std::collections::{HashMap, HashSet};

use polycubes_core::Coordinates;

use crate::rule::Rule;

/// When to give up on a seed that neither dies nor repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub generation_count: usize,
    pub population: usize,
}

/// What becomes of a seed. `generation` is the first one of the final
/// behaviour: the seed itself is generation zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fate {
    /// No cell is left at `generation`.
    Dies { generation: usize },
    /// The cells no longer change from `generation` on.
    Still { generation: usize },
    /// The same cells come back every `period` generations.
    Cycles { generation: usize, period: usize },
    /// The same shape comes back every `period` generations, moved by
    /// `displacement`.
    Travels {
        generation: usize,
        period: usize,
        displacement: Coordinates,
    },
    /// A limit was reached first.
    Unsettled,
}

/// The generations of a seed, each sorted, up to the first repeat or limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evolution {
    pub generations: Vec<Vec<Coordinates>>,
    pub fate: Fate,
}

/// The living cells one generation later.
#[must_use]
pub fn next_generation(living: &HashSet<Coordinates>, rule: &Rule) -> HashSet<Coordinates> {
    let steps = rule.neighbourhood.steps();
    let mut living_neighbour_counts: HashMap<Coordinates, usize> = HashMap::new();
    for cell in living {
        for step in &steps {
            let neighbour = [0, 1, 2].map(|axis| cell[axis] + step[axis]);
            *living_neighbour_counts.entry(neighbour).or_insert(0) += 1;
        }
    }
    let mut next: HashSet<Coordinates> = living_neighbour_counts
        .iter()
        .filter(|&(cell, &count)| {
            if living.contains(cell) {
                rule.keeps_alive(count)
            } else {
                rule.gives_birth(count)
            }
        })
        .map(|(&cell, _)| cell)
        .collect();
    // A living cell without living neighbour appears in no count.
    if rule.keeps_alive(0) {
        next.extend(
            living
                .iter()
                .filter(|cell| !living_neighbour_counts.contains_key(*cell)),
        );
    }
    next
}

/// Runs the automaton from `seed` until it dies, repeats or reaches a limit.
#[must_use]
pub fn evolve(seed: &[Coordinates], rule: &Rule, limits: Limits) -> Evolution {
    let mut living: HashSet<Coordinates> = seed.iter().copied().collect();
    let mut generations = vec![sorted(&living)];
    // Each shape seen, moved against the origin, with when and where.
    let mut seen_shapes: HashMap<Vec<Coordinates>, (usize, Coordinates)> = HashMap::new();
    let fate = loop {
        let generation = generations.len() - 1;
        let Some((shape, position)) = shape_and_position(&generations[generation]) else {
            break Fate::Dies { generation };
        };
        if let Some(&(first_generation, first_position)) = seen_shapes.get(&shape) {
            let displacement = [0, 1, 2].map(|axis| position[axis] - first_position[axis]);
            break repeat_fate(
                first_generation,
                generation - first_generation,
                displacement,
            );
        }
        seen_shapes.insert(shape, (generation, position));
        if generation == limits.generation_count {
            break Fate::Unsettled;
        }
        let next = next_generation(&living, rule);
        if next.len() > limits.population {
            break Fate::Unsettled;
        }
        living = next;
        generations.push(sorted(&living));
    };
    Evolution { generations, fate }
}

fn repeat_fate(generation: usize, period: usize, displacement: Coordinates) -> Fate {
    if displacement != [0, 0, 0] {
        Fate::Travels {
            generation,
            period,
            displacement,
        }
    } else if period == 1 {
        Fate::Still { generation }
    } else {
        Fate::Cycles { generation, period }
    }
}

fn sorted(living: &HashSet<Coordinates>) -> Vec<Coordinates> {
    let mut cells: Vec<Coordinates> = living.iter().copied().collect();
    cells.sort_unstable();
    cells
}

/// The cells moved against the origin and the corner they were moved from;
/// `None` when no cell is left.
fn shape_and_position(cells: &[Coordinates]) -> Option<(Vec<Coordinates>, Coordinates)> {
    if cells.is_empty() {
        return None;
    }
    let position = [0, 1, 2].map(|axis| cells.iter().map(|cell| cell[axis]).min().unwrap_or(0));
    let shape = cells
        .iter()
        .map(|cell| [0, 1, 2].map(|axis| cell[axis] - position[axis]))
        .collect();
    Some((shape, position))
}
