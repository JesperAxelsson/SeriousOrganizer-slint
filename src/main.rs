// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use serious_organizer_lib::models::DirEntry;
use slint::{Model, ModelNotify, ModelRc, StandardListViewItem, VecModel, Weak};

slint::include_modules!();

use serious_organizer_lib::dir_search;
use serious_organizer_lib::lens::Lens;

#[cfg(debug_assertions)]
fn get_dir_path() -> String {
    ::std::env::current_exe()
        .unwrap()
        .with_file_name("test.sqlite3")
        .to_string_lossy()
        .to_string()
}

#[cfg(not(debug_assertions))]
fn get_dir_path() -> String {
    use directories::BaseDirs;
    use std::fs::{self, File};

    if let Some(base_dirs) = BaseDirs::new() {
        let dir = base_dirs.data_dir();
        let mut dir = dir.to_path_buf();
        dir.push("SeriousOrganizer");
        println!("Got base dir!");
        fs::create_dir_all(&dir).expect(&format!(
            "Failed to create data dir: {}",
            dir.to_string_lossy()
        ));

        dir.push("Database.sqlite3");

        if !dir.exists() {
            File::create(&dir).expect(&format!(
                "Failed to create db_file: {:?}",
                dir.to_string_lossy()
            ));
        }

        dir.to_string_lossy().to_string()
    } else {
        println!("No base dir! :'(");
        ::std::env::current_exe()
            .unwrap()
            .with_file_name("test.sqlite3")
            .to_string_lossy()
            .to_string()
    }
}

enum LensMessage {
    ReloadResult(Vec<(i32, DirEntry)>),
}

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");

    let db_path = get_dir_path();
    println!("dbpath: {}", db_path);
    let lens = Rc::new(RefCell::new(Lens::new(&db_path)));
    // {
    //     let mut foo = lens.lock().unwrap();
    //     foo.add_location("Banana", "/home/jesper/Videos");
    // }

    let lens_model = Rc::new(LensModelList {
        lens: lens.clone(),
        notify: ModelNotify::default(),
    });

    let ui = AppWindow::new()?;

    // ===== Setup message queue =====

    // Setup Entry table
    ui.set_items(ModelRc::new(lens_model.clone()));
    ui.on_entry_changed({
        // let ui_handle = ui.as_weak();
        let lens = lens.clone();
        move |entry_ix| {
            // let entry = lens.get_dir_entry(entry_ix);
            let file_count = lens
                .borrow()
                .get_file_count(entry_ix as usize)
                .unwrap_or_default();
            println!("Entry ix: {entry_ix} count: {file_count}");
        }
    });

    // Searching
    ui.on_search_text_changed({
        let ui_handle = ui.as_weak();
        let lens = lens.clone();
        let lens_model_c = lens_model.clone();
        move |text| {
            println!("Search text: {text}");
            let _ui = ui_handle.unwrap();
            lens.borrow_mut().update_search_text(&text);
            lens_model_c.update();
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

            let ui_handle = ui_weak.unwrap();
            update_lens(tx.clone(), ui_handle.as_weak(), paths);

            lens_model_c.update();
        }
    });

    ui.run()?;

    Ok(())
}

pub struct LensModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
}

impl Model for LensModelList {
    type Data = ModelRc<StandardListViewItem>;

    fn row_count(&self) -> usize {
        let count = self.lens.borrow().get_dir_count();

        // println!("Row count {count}");
        count
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        println!("Print row data {row}");
        if let Some(entry) = self.lens.borrow().get_dir_entry(row) {
            let items = vec![
                StandardListViewItem::from(entry.name.as_str()),
                StandardListViewItem::from(entry.path.as_str()),
                StandardListViewItem::from(format_size(entry.size).as_str()),
            ];

            let row_model = Rc::new(VecModel::from(items));

            Some(ModelRc::from(row_model))
        } else {
            None
        }
    }

    fn set_row_data(&self, _row: usize, _data: Self::Data) {
        // self.array.borrow_mut()[row] = data;
        // self.notify.row_changed(row);
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl LensModelList {
    pub fn update(&self) {
        println!("Reset the thing!");
        self.notify.reset();
    }
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

fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:>10.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:>10.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:>10.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
