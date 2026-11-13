// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::rc::Rc;

use slint::ModelRc;

slint::include_modules!();

use serious_organizer_lib::lens::Lens;

use crate::entry_model_list::LensModelList;
use crate::file_model_list::FileModelList;
use crate::location_model_list::reload_locations;
use crate::reload_controller::ReloadController;

mod config;
mod entry_model_list;
mod file_model_list;
mod helper;
mod location_model_list;
mod reload_controller;

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");

    let db_path = config::get_dir_path();
    println!("dbpath: {}", db_path);
    let lens = Rc::new(RefCell::new(Lens::new(&db_path)));
    // {
    //     let mut foo = lens.lock().unwrap();
    //     foo.add_location("Banana", "/home/jesper/Videos");
    // }

    // Models
    let lens_model = Rc::new(LensModelList::new(lens.clone()));
    let file_model = Rc::new(FileModelList::new(lens.clone()));

    let ui = AppWindow::new()?;

    // Setup reload controls, we don't need to keep a variable around
    ReloadController::new(ui.as_weak(), lens_model.clone());
    reload_locations(&ui, &lens.borrow());

    // ===== Setup message queue =====

    // Setup Entry table
    ui.set_items(ModelRc::new(lens_model.clone()));
    ui.on_entry_changed({
        let file_model = file_model.clone();
        move |entry_ix| {
            file_model.set_entry_ix(entry_ix);
        }
    });
    ui.on_entry_clicked({
        let lens_model = lens_model.clone();
        move |row, ctrl, shift| {
            if row >= 0 {
                lens_model.click(row as usize, ctrl, shift);
            }
        }
    });
    ui.on_entry_context_opened({
        let lens_model = lens_model.clone();
        move |row| {
            if row >= 0 {
                lens_model.select_for_context(row as usize);
            }
        }
    });
    ui.on_entry_activated({
        let lens = lens.clone();
        move |row| {
            if row < 0 {
                return;
            }

            if let Some(entry) = lens.borrow().get_dir_entry(row as usize) {
                if let Err(err) = open::that_detached(&entry.path) {
                    eprintln!("Failed to open '{}': {err}", entry.path);
                }
            }
        }
    });

    ui.on_context_command(|command| {
        println!("Got command: '{command}'");
    });

    // Setup File table
    ui.set_file_items(ModelRc::new(file_model));

    // Searching
    ui.on_search_text_changed({
        let ui_handle = ui.as_weak();
        let lens = lens.clone();
        let lens_model_c = lens_model.clone();

        move |text| {
            let ui_handle = ui_handle.unwrap();

            // Get current entry id
            let cur_ix = ui_handle.get_current_entry();
            let entry_id = if cur_ix >= 0 {
                lens.borrow().get_dir_entry(cur_ix as usize).map(|e| e.id)
            } else {
                None
            };

            // Search
            println!("Search text: {text} {:?}", entry_id);
            lens.borrow_mut().update_search_text(&text);
            lens_model_c.update();

            // Check if current entry is still present and select it
            let mut new_entry_id = None;
            if let Some(entry_id) = entry_id {
                // Is the old entry filtered out?
                if let Some(new_ix) = lens.borrow().entry_id_to_ix(entry_id) {
                    ui_handle.set_current_entry(new_ix as i32);
                    new_entry_id = Some(new_ix as i32);
                } else {
                    // Reset selection
                    ui_handle.set_current_entry(-1);
                    new_entry_id = Some(-1);
                }
            }

            println!("Search done, current entry ix: {:?}", new_entry_id);
        }
    });

    // Locations
    ui.on_add_location({
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
