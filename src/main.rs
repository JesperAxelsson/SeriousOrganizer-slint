// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use serious_organizer_lib::models::{DirEntry, File};
use slint::{ModelRc, StandardListViewItem, VecModel, Weak};

slint::include_modules!();

use serious_organizer_lib::dir_search;
use serious_organizer_lib::lens::Lens;

use crate::entry_model_list::LensModelList;
use crate::file_model_list::FileModelList;
use crate::helper::format_size;

mod config;
mod entry_model_list;
mod file_model_list;
mod helper;

enum LensMessage {
    ReloadResult(Vec<(i32, DirEntry)>),
}

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");

    let db_path = config::get_dir_path();
    println!("dbpath: {}", db_path);
    let lens = Rc::new(RefCell::new(Lens::new(&db_path)));
    // {
    //     let mut foo = lens.lock().unwrap();
    //     foo.add_location("Banana", "/home/jesper/Videos");
    // }

    let lens_model = Rc::new(LensModelList::new(lens.clone()));
    let _file_model = Rc::new(FileModelList::new(lens.clone()));

    let ui = AppWindow::new()?;

    // ===== Setup message queue =====

    // Setup Entry table
    ui.set_items(ModelRc::new(lens_model.clone()));
    ui.on_entry_changed({
        let ui_handle = ui.as_weak();
        let lens = lens.clone();
        move |entry_ix| {
            let file_count = lens
                .borrow()
                .get_file_count(entry_ix as usize)
                .unwrap_or_default();

            let ui_handle = ui_handle.unwrap();

            if let Some(files) = lens.borrow().get_dir_files(entry_ix as usize) {
                println!("Files: {:?}", files.len());
            }

            // let cur_ix = ui_handle.get_current_entry();
            // let entry_id = if cur_ix >= 0 {
            //     lens.borrow().get_dir_entry(cur_ix as usize).map(|e| e.id)
            // } else {
            //     None
            // };
            // println!(
            //     "Entry ix: {entry_ix} count: {file_count} cur: {:?}",
            //     entry_id // ui_handle.upgrade().unwrap().get_current_entry()
            // );
        }
    });

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

    // Reloading
    let (tx, rx): (Sender<LensMessage>, Receiver<LensMessage>) = channel();
    {
        let lens_model = lens_model.clone();

        ui.on_reload_done(move || {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    LensMessage::ReloadResult(mut result) => {
                        lens_model.lens.borrow_mut().update_data(&mut result);
                        lens_model.update();
                    }
                }
            }
        });
    }

    ui.on_reload({
        let lens_model_c = lens_model.clone();
        let lens_c = lens.clone();
        let ui_weak = ui.as_weak();

        move || {
            let paths = lens_c
                .borrow()
                .get_locations()
                .iter()
                .map(|e| (e.id, e.path.clone()))
                .collect();

            update_lens(tx.clone(), ui_weak.clone(), paths);
            lens_model_c.update();
        }
    });

    ui.run()?;

    Ok(())
}

fn update_lens(tx: Sender<LensMessage>, ui: Weak<AppWindow>, paths: Vec<(i32, String)>) {
    thread::spawn(move || {
        println!("Locations: {:?}", paths);

        let dir_s = dir_search::get_all_data(&paths);

        tx.send(LensMessage::ReloadResult(dir_s))
            .expect("Failed to send Lens Reload");

        slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade() {
                ui.invoke_reload_done();
            }
        })
        .unwrap();
    });
}

pub fn convert_file_to_model(file: File) -> ModelRc<StandardListViewItem> {
    let items = vec![
        StandardListViewItem::from(file.name.as_str()),
        StandardListViewItem::from(file.path.as_str()),
        StandardListViewItem::from(format_size(file.size).as_str()),
    ];

    let row_model = Rc::new(VecModel::from(items));

    ModelRc::from(row_model)
}
