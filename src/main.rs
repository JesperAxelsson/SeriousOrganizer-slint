// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::rc::Rc;

use slint::ModelRc;

slint::include_modules!();

use serious_organizer_lib::lens::Lens;

use crate::app::App;
use crate::reload_controller::ReloadController;

mod app;
mod config;
mod controllers;
mod entry_model_list;
mod file_model_list;
mod helper;
mod label_model_list;
mod location_model_list;
mod reload_controller;
mod selection;

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");

    let db_path = config::get_dir_path();
    println!("dbpath: {}", db_path);
    let lens = Rc::new(RefCell::new(Lens::new(&db_path)));

    // Models
    let ui = AppWindow::new()?;
    let state = ui.global::<AppState>();

    let app = App::new(&ui, lens.clone());
    app.wire(&ui);

    // Setup reload controls, we don't need to keep a variable around
    ReloadController::new(ui.as_weak(), app.entry_table.kind.model.clone());
    // reload_locations(&ui, &lens.borrow());

    // Setup Entry table
    state.set_items(ModelRc::new(app.entry_table.kind.model.clone()));

    // Setup File table
    state.set_file_items(ModelRc::new(app.file_table.kind.model.clone()));

    // Setup Location Table
    state.set_locations(ModelRc::new(app.location.model.clone()));

    // Setup Label Table
    state.set_labels(ModelRc::new(app.labels.clone()));

    ui.run()?;

    Ok(())
}
