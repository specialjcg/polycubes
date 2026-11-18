//! Gathers a gallery of varied patterns of the Conway-like life of whole
//! polycubes: still ones, oscillators and gliders, all of them spread over
//! at least two layers along every axis, each found once however turned.
//!
//! The shapes, rules and tiny starts are those of `glider_search`. Among the
//! patterns found, about two dozen are kept for their variety of shape,
//! nature and symmetry, then written as JSON, the most symmetric first.
//!
//! Usage: `cargo run --release -p polycubes-automaton --example pattern_gallery -- [starts per rule] [output.json]`

mod search_space;

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;

use polycubes_automaton::{
    Classification, ConwayRule, Glider, Layout, Nature, SCREW_GLIDER, SQUARE_GLIDER,
    THICK_SQUARE_GLIDER, Watch, canonical_form, classify, is_solid,
};
use polycubes_core::Coordinates;
use rayon::prelude::*;
use search_space::{tiny_start, trials};

const WATCH: Watch = Watch {
    generation_count: 60,
    population_cap: 40,
};

const GALLERY_SIZE: usize = 24;
const PER_NATURE: usize = GALLERY_SIZE / 3;
const NATURES: [Nature; 3] = [Nature::Still, Nature::Oscillator, Nature::Glider];

/// Fewer polycubes make patterns too plain to show.
const FEWEST_POLYCUBES: usize = 3;

/// The solid gliders already known, with the name of their shape: tiny
/// starts seldom turn into them.
const KNOWN_GLIDERS: [(&str, Glider); 3] = [
    ("O4", SQUARE_GLIDER),
    ("O4", THICK_SQUARE_GLIDER),
    ("screw", SCREW_GLIDER),
];

struct Found {
    shape_name: &'static str,
    rule: ConwayRule,
    classification: Classification,
}

impl Found {
    fn polycube_count(&self) -> usize {
        self.classification.layout.len()
    }

    /// Small, middling or large.
    fn size_class(&self) -> usize {
        match self.polycube_count() {
            0..=5 => 0,
            6..=10 => 1,
            _ => 2,
        }
    }

    /// Cells of the box around the pattern per living cell, in thousandths:
    /// the higher, the more scattered.
    fn spread(&self) -> usize {
        let layout = &self.classification.layout;
        let box_volume: usize = [0, 1, 2]
            .map(|axis| {
                let extent = layout.iter().flatten().map(|cell| cell[axis]).max();
                usize::try_from(extent.unwrap_or(0) + 1).unwrap_or(1)
            })
            .iter()
            .product();
        let cell_count = layout.iter().map(Vec::len).sum::<usize>().max(1);
        box_volume * 1000 / cell_count
    }
}

/// A pattern worth showing: enough polycubes, in relief.
fn is_showable(classification: &Classification) -> bool {
    classification.layout.len() >= FEWEST_POLYCUBES && is_solid(&classification.layout)
}

/// The patterns a shape turns into under a rule, each once.
fn patterns_of(
    shape_name: &'static str,
    shape: &[Coordinates],
    rule: ConwayRule,
    seeds: std::ops::Range<u64>,
) -> Vec<(Layout, Found)> {
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for seed in seeds {
        let Some(classification) = classify(&tiny_start(shape, seed), rule, WATCH) else {
            continue;
        };
        if !is_showable(&classification) {
            continue;
        }
        let canonical = canonical_form(&classification.layout);
        if seen.insert(canonical.clone()) {
            found.push((
                canonical,
                Found {
                    shape_name,
                    rule,
                    classification,
                },
            ));
        }
    }
    found
}

/// Takes `count` patterns out of `remaining`, each time the one whose
/// shape, symmetry and size were taken the least, the longest period then
/// the most symmetric first among equals.
fn varied_pick(remaining: &mut Vec<Found>, count: usize) -> Vec<Found> {
    remaining.sort_by_key(|found| {
        (
            std::cmp::Reverse(found.classification.period),
            std::cmp::Reverse(found.classification.symmetry),
            found.polycube_count(),
        )
    });
    let mut picked: Vec<Found> = Vec::new();
    while picked.len() < count && !remaining.is_empty() {
        let novelty = |found: &Found| {
            let same_shape = picked
                .iter()
                .filter(|chosen| chosen.shape_name == found.shape_name)
                .count();
            let same_symmetry = picked
                .iter()
                .filter(|chosen| chosen.classification.symmetry == found.classification.symmetry)
                .count();
            let same_size = picked
                .iter()
                .filter(|chosen| chosen.size_class() == found.size_class())
                .count();
            (same_shape, same_symmetry, same_size)
        };
        let best = (0..remaining.len())
            .min_by_key(|&rank| novelty(&remaining[rank]))
            .unwrap_or(0);
        picked.push(remaining.remove(best));
    }
    picked
}

fn nature_name(nature: Nature) -> &'static str {
    match nature {
        Nature::Still => "still",
        Nature::Oscillator => "oscillator",
        Nature::Glider => "glider",
    }
}

fn coordinates_json(cell: Coordinates) -> String {
    format!("[{},{},{}]", cell[0], cell[1], cell[2])
}

fn pattern_json(number: usize, found: &Found) -> String {
    let classification = &found.classification;
    let rule = found.rule;
    let polycubes: Vec<String> = classification
        .layout
        .iter()
        .map(|cells| {
            let cells: Vec<String> = cells.iter().map(|&cell| coordinates_json(cell)).collect();
            format!("[{}]", cells.join(","))
        })
        .collect();
    let [fewest_layers, middle_layers, most_layers] = classification.layers;
    let mut json = String::new();
    let _ = write!(
        json,
        concat!(
            "    {{\"number\": {}, \"shape\": \"{}\", \"nature\": \"{}\", \"period\": {}, ",
            "\"shift\": {}, \"layers\": [{},{},{}], \"symmetry\": {},\n",
            "     \"rule\": {{\"fewest_neighbours_to_survive\": {}, \"most_neighbours_to_survive\": {}, ",
            "\"fewest_neighbours_to_be_born\": {}, \"most_neighbours_to_be_born\": {}, ",
            "\"fewest_faces_shared_to_be_born\": {}}},\n",
            "     \"polycubes\": [{}]}}"
        ),
        number,
        found.shape_name,
        nature_name(classification.nature),
        classification.period,
        coordinates_json(classification.shift),
        fewest_layers,
        middle_layers,
        most_layers,
        classification.symmetry,
        rule.fewest_neighbours_to_survive,
        rule.most_neighbours_to_survive,
        rule.fewest_neighbours_to_be_born,
        rule.most_neighbours_to_be_born,
        rule.fewest_faces_shared_to_be_born,
        polycubes.join(",")
    );
    json
}

fn main() -> std::io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let starts_per_rule: u64 = arguments
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or(100);
    let output_path = arguments
        .next()
        .unwrap_or_else(|| "docs/vie/galerie.json".to_owned());
    let known = KNOWN_GLIDERS.iter().filter_map(|&(shape_name, glider)| {
        let classification = classify(&glider.start(), glider.rule, WATCH)?;
        Some((
            canonical_form(&classification.layout),
            Found {
                shape_name,
                rule: glider.rule,
                classification,
            },
        ))
    });
    let searched: Vec<(Layout, Found)> = trials()
        .par_iter()
        .flat_map_iter(|&(shape_name, shape, rule)| {
            patterns_of(shape_name, shape, rule, 1..starts_per_rule + 1)
        })
        .collect();
    let mut seen = HashSet::new();
    let mut by_nature: BTreeMap<Nature, Vec<Found>> = BTreeMap::new();
    for (canonical, pattern) in known.chain(searched) {
        if seen.insert((pattern.shape_name, canonical)) {
            by_nature
                .entry(pattern.classification.nature)
                .or_default()
                .push(pattern);
        }
    }
    for nature in NATURES {
        let count = by_nature.get(&nature).map_or(0, Vec::len);
        println!("{} distinct {} patterns", count, nature_name(nature));
    }
    let mut gallery: Vec<Found> = NATURES
        .iter()
        .flat_map(|nature| {
            by_nature
                .get_mut(nature)
                .map(|remaining| varied_pick(remaining, PER_NATURE))
                .unwrap_or_default()
        })
        .collect();
    // A nature short of patterns leaves its room to the others.
    let mut leftovers: Vec<Found> = by_nature.into_values().flatten().collect();
    let missing_count = GALLERY_SIZE.saturating_sub(gallery.len());
    gallery.extend(varied_pick(&mut leftovers, missing_count));
    gallery.sort_by_key(|found| {
        (
            std::cmp::Reverse(found.classification.symmetry),
            found.spread(),
        )
    });
    let entries: Vec<String> = gallery
        .iter()
        .enumerate()
        .map(|(rank, found)| pattern_json(rank + 1, found))
        .collect();
    for (rank, found) in gallery.iter().enumerate() {
        let classification = &found.classification;
        println!(
            "{:2} {:12} {:10} period {:2} symmetry {:2} layers {:?} {} polycubes",
            rank + 1,
            found.shape_name,
            nature_name(classification.nature),
            classification.period,
            classification.symmetry,
            classification.layers,
            classification.layout.len()
        );
    }
    fs::write(
        output_path,
        format!("{{\"patterns\": [\n{}\n]}}\n", entries.join(",\n")),
    )
}
