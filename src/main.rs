// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread;

use slint::{Model, ModelNotify, ModelRc};

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

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");

    let db_path = get_dir_path();
    println!("dbpath: {}", db_path);
    let lens = Arc::new(Mutex::new(Lens::new(&db_path)));
    // {
    //     let mut foo = lens.lock().unwrap();
    //     foo.add_location("Banana", "/home/jesper/Videos");
    // }
    // println!("Hello: {:?}", foo);

    // update_lens(lens);

    let lens_model = LensModelList {
        lens: lens.clone(),
        notify: ModelNotify::default(),
    };

    // let model: VecModel<Rect> = VecModel::new();
    // model.push(Rect {
    //     text: "Blue".into(),
    //     color: Color::from_rgb_u8(0, 0, 0xff),
    //     bg: Color::from_rgb_u8(0, 0xff, 0),
    // });
    // model.push(Rect {
    //     text: "Red".into(),
    //     color: Color::from_rgb_u8(0, 0, 0xff),
    //     bg: Color::from_rgb_u8(0xff, 0, 0),
    // });

    let ui = AppWindow::new()?;

    // ui.set_items(ModelRc::new(model));
    ui.set_items(ModelRc::new(lens_model));

    ui.on_search_text_changed({
        let ui_handle = ui.as_weak();
        let lens_c = lens.clone();
        move |text| {
            println!("Search text: {text}");
            let _ui = ui_handle.unwrap();
            let mut lens = lens_c.lock().unwrap();
            lens.update_search_text(&text);
            // ui.set_search_text(ui.get_search_text() );
        }
    });

    ui.on_reload({
        let lens_c = lens.clone();
        move || {
            update_lens(lens_c.clone());
        }
    });

    ui.run()?;

    Ok(())
}

pub struct LensModelList {
    lens: Arc<Mutex<Lens>>,
    notify: ModelNotify,
}

impl Model for LensModelList {
    type Data = EntryModel;

    fn row_count(&self) -> usize {
        let count = self.lens.lock().unwrap().get_dir_count();

        // println!("Row count {count}");
        count
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        println!("Row data {row}");
        if let Some(entry) = self.lens.lock().unwrap().get_dir_entry(row) {
            Some(EntryModel {
                name: entry.name.clone().into(),
                path: entry.path.clone().into(),
                size: entry.size.to_string().into(),
            })
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

pub struct VecModel<T> {
    array: std::cell::RefCell<Vec<T>>,
    notify: ModelNotify,
}

impl<T: Clone + 'static> Model for VecModel<T> {
    type Data = T;

    fn row_count(&self) -> usize {
        self.array.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.array.borrow().get(row).cloned()
    }

    fn set_row_data(&self, row: usize, data: Self::Data) {
        self.array.borrow_mut()[row] = data;
        self.notify.row_changed(row);
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl<T> VecModel<T> {
    pub fn new() -> Self {
        VecModel {
            array: RefCell::new(Vec::new()),
            notify: ModelNotify::default(),
        }
    }
    pub fn push(&self, value: T) {
        self.array.borrow_mut().push(value);
        self.notify.row_added(self.array.borrow().len() - 1, 1);
    }

    pub fn remove(&self, index: usize) {
        self.array.borrow_mut().remove(index);
        self.notify.row_removed(index, 1);
    }
}

fn update_lens(lens: Arc<Mutex<Lens>>) {
    thread::spawn(move || {
        println!("Start update data");
        let mut lens = lens.lock().unwrap();

        let paths = lens
            .get_locations()
            .iter()
            .map(|e| (e.id, e.path.clone()))
            .collect();

        println!("Locations: {:?}", paths);

        let mut dir_s = dir_search::get_all_data(&paths);
        lens.update_data(&mut dir_s);

        // sender_c.send(Message::HideLoading);
        println!("Done update data {:?}", lens.ix_list.len());
    });
}
