//! Desktop shell: exposes the polycube crates to the Elm frontend.
//!
//! No rule lives here: commands parse their input, call the crates and
//! serialise the result.

pub mod closed_form;
pub mod drawing;
pub mod enumeration;
pub mod gallery;
pub mod growth;
pub mod remarkable;

use std::sync::Mutex;

use closed_form::ClosedFormComparison;
use drawing::{CellCount, DrawnPolycube, PolycubeDrawer};
use enumeration::TracedStep;
use gallery::Gallery;
use growth::GrowthComparison;
use remarkable::MostLoopedPolycubes;

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

/// Finds the polycubes of `cell_count` cells with the most loops.
#[tauri::command(async)]
fn find_most_looped(cell_count: usize) -> Result<MostLoopedPolycubes, String> {
    remarkable::find_most_looped(cell_count).map_err(|error| error.to_string())
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
            find_most_looped
        ])
        .run(tauri::generate_context!())
        .expect("the desktop application failed to start");
}
