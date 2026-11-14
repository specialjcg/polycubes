//! Desktop shell: exposes the polycube crates to the Elm frontend.
//!
//! No rule lives here: commands parse their input, call the crates and
//! serialise the result.

pub mod automaton;
pub mod closed_form;
pub mod drawing;
pub mod enumeration;
pub mod gallery;
pub mod growth;
pub mod remarkable;

use std::sync::Mutex;

use automaton::{AutomatonRun, GrowthMode, SeedKind};
use closed_form::ClosedFormComparison;
use drawing::{CellCount, DrawnPolycube, PolycubeDrawer};
use enumeration::TracedStep;
use gallery::Gallery;
use growth::GrowthComparison;
use remarkable::RemarkablePolycubes;

/// Draws a fixed polycube of `cell_count` cells uniformly at random.
#[tauri::command(async)]
fn draw_uniform_polycube(
    cell_count: usize,
    drawer: tauri::State<'_, Mutex<PolycubeDrawer>>,
) -> Result<DrawnPolycube, String> {
    let cell_count = CellCount::try_from(cell_count).map_err(|error| error.to_string())?;
    let mut drawer = drawer
        .lock()
        .map_err(|_| "a previous draw failed, restart the application".to_owned())?;
    drawer
        .draw(cell_count, |polycube_count| {
            rand::random_range(0..polycube_count)
        })
        .map_err(|error| error.to_string())
}

/// Builds a polycube by random neighbour growth and draws another one
/// uniformly, both of `cell_count` cells.
#[tauri::command(async)]
fn compare_growths(
    cell_count: usize,
    drawer: tauri::State<'_, Mutex<PolycubeDrawer>>,
) -> Result<GrowthComparison, String> {
    let cell_count = CellCount::try_from(cell_count).map_err(|error| error.to_string())?;
    let mut drawer = drawer
        .lock()
        .map_err(|_| "a previous draw failed, restart the application".to_owned())?;
    growth::compare_growths(
        &mut drawer,
        cell_count,
        |polycube_count| rand::random_range(0..polycube_count),
        |candidate_count| rand::random_range(0..candidate_count),
    )
    .map_err(|error| error.to_string())
}

/// Traces the enumeration of the fixed polycubes of at most `cell_count`
/// cells, node by node.
#[tauri::command(async)]
fn trace_enumeration(cell_count: usize) -> Result<Vec<TracedStep>, String> {
    enumeration::trace_enumeration(cell_count).map_err(|error| error.to_string())
}

/// Picks a polycube of `cell_count` cells at random in the enumeration tree
/// and counts its extensions by the closed formula and by visiting them.
#[tauri::command(async)]
fn compare_closed_form(cell_count: usize) -> Result<ClosedFormComparison, String> {
    closed_form::compare_closed_form(cell_count, |candidate_count| {
        rand::random_range(0..candidate_count)
    })
    .map_err(|error| error.to_string())
}

/// Samples uniform draws and neighbour growths of `cell_count` cells and
/// sorts them by compactness.
#[tauri::command(async)]
fn build_gallery(
    cell_count: usize,
    drawer: tauri::State<'_, Mutex<PolycubeDrawer>>,
) -> Result<Gallery, String> {
    let mut drawer = drawer
        .lock()
        .map_err(|_| "a previous draw failed, restart the application".to_owned())?;
    gallery::build_gallery(
        &mut drawer,
        cell_count,
        |polycube_count| rand::random_range(0..polycube_count),
        |candidate_count| rand::random_range(0..candidate_count),
    )
    .map_err(|error| error.to_string())
}

/// Finds the exceptional polycubes of `cell_count` cells.
#[tauri::command(async)]
fn find_remarkable(cell_count: usize) -> Result<RemarkablePolycubes, String> {
    remarkable::find_remarkable(cell_count).map_err(|error| error.to_string())
}

/// Picks a seed of `cell_count` cells and makes it grow.
#[tauri::command(async)]
fn run_automaton(
    cell_count: usize,
    seed_kind: SeedKind,
    mode: GrowthMode,
    drawer: tauri::State<'_, Mutex<PolycubeDrawer>>,
) -> Result<AutomatonRun, String> {
    automaton::check_seed_cell_count(cell_count).map_err(|error| error.to_string())?;
    let seed = match seed_kind {
        SeedKind::Uniform => {
            let cell_count = CellCount::try_from(cell_count).map_err(|error| error.to_string())?;
            let mut drawer = drawer
                .lock()
                .map_err(|_| "a previous draw failed, restart the application".to_owned())?;
            drawer
                .draw(cell_count, |polycube_count| {
                    rand::random_range(0..polycube_count)
                })
                .map_err(|error| error.to_string())?
                .cells
        }
        SeedKind::Growth => {
            polycubes_sampling::grow_by_random_neighbour(cell_count, |candidate_count| {
                rand::random_range(0..candidate_count)
            })
        }
        SeedKind::MostLooped => polycubes_sampling::most_looped_polycubes(cell_count)
            .shapes
            .into_iter()
            .next()
            .map(|shape| shape.cells)
            .ok_or_else(|| "no polycube of that size".to_owned())?,
    };
    Ok(match mode {
        GrowthMode::Replicator => automaton::replicate(&seed),
        GrowthMode::EdgeReplicator => automaton::replicate_over_edges(&seed),
        GrowthMode::Colony => automaton::grow(
            &seed,
            || rand::random_bool(automaton::COLONY_DIVISION_CHANCE),
            |free_neighbour_count| rand::random_range(0..free_neighbour_count),
        ),
        GrowthMode::Mutation => automaton::bud_mutants(
            &seed,
            || rand::random_bool(automaton::BUDDING_CHANCE),
            |choice_count| rand::random_range(0..choice_count),
        ),
        GrowthMode::Budding => automaton::bud(
            &seed,
            || rand::random_bool(automaton::BUDDING_CHANCE),
            |placement_count| rand::random_range(0..placement_count),
        ),
    })
}

/// Runs the desktop application until its window is closed.
///
/// # Panics
///
/// Panics if the window system cannot be initialised.
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(PolycubeDrawer::default()))
        .invoke_handler(tauri::generate_handler![
            draw_uniform_polycube,
            compare_growths,
            trace_enumeration,
            compare_closed_form,
            build_gallery,
            find_remarkable,
            run_automaton
        ])
        .run(tauri::generate_context!())
        .expect("the desktop application failed to start");
}
