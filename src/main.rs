// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::rc::Rc;

use slint::ModelRc;

slint::include_modules!();

use serious_organizer_lib::lens::Lens;

use crate::app::App;
use crate::location_model_list::reload_locations;
use crate::reload_controller::ReloadController;

mod app;
mod config;
mod entry_model_list;
mod file_model_list;
mod helper;
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
    ReloadController::new(ui.as_weak(), app.entry_model.clone());
    reload_locations(&ui, &lens.borrow());

    // Setup Entry table
    state.set_items(ModelRc::new(app.entry_model.clone()));

    // Setup File table
    state.set_file_items(ModelRc::new(app.file_model.clone()));

    // *** Locations ***
    ui.on_add_location({
        // let state_handle = state.as_weak();
        let ui_handle = ui.as_weak();
        let lens = lens.clone();

        move || {
            if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
                let path_str = folder_path.to_string_lossy().to_string();
                let name = folder_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                lens.borrow_mut().add_location(&name, &path_str);

                if let Some(ui) = ui_handle.upgrade() {
                    reload_locations(&ui, &lens.borrow());
                }
            }
        }
    });

    ui.on_remove_location({
        let ui_handle = ui.as_weak();
        let lens = lens.clone();
        move |id| {
            lens.borrow_mut().remove_location_id(id);

            if let Some(ui) = ui_handle.upgrade() {
                reload_locations(&ui, &lens.borrow());
            }
        }
    });

    ui.run()?;

    Ok(())
}
