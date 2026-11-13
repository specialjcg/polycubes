//! Desktop shell: exposes the polycube crates to the Elm frontend.
//!
//! No rule lives here: commands parse their input, call the crates and
//! serialise the result.

pub mod drawing;

use std::sync::Mutex;

use drawing::{CellCount, DrawnPolycube, PolycubeDrawer};

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

/// Runs the desktop application until its window is closed.
///
/// # Panics
///
/// Panics if the window system cannot be initialised.
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(PolycubeDrawer::default()))
        .invoke_handler(tauri::generate_handler![draw_uniform_polycube])
        .run(tauri::generate_context!())
        .expect("the desktop application failed to start");
}
