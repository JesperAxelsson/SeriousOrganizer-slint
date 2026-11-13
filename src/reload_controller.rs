use std::{
    rc::Rc,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
};

use serious_organizer_lib::{dir_search, models::DirEntry};
use slint::{ComponentHandle, Weak};

use crate::{AppState, AppWindow, entry_model_list::EntryModelList};

enum LensMessage {
    ReloadResult(Vec<(i32, DirEntry)>),
}

pub struct ReloadController {
    ui: Weak<AppWindow>,
    lens_model: Rc<EntryModelList>,
    tx: Sender<LensMessage>,
}

impl ReloadController {
    pub fn new(ui: Weak<AppWindow>, lens_model: Rc<EntryModelList>) -> Self {
        let (tx, rx): (Sender<LensMessage>, Receiver<LensMessage>) = channel();

        let controller = Self { ui, lens_model, tx };

        controller.setup_reciever(rx);
        controller.setup_trigger();

        controller
    }

    fn setup_reciever(&self, rx: Receiver<LensMessage>) {
        let lens_model = self.lens_model.clone();

        if let Some(ui) = self.ui.upgrade() {
            ui.global::<AppState>().on_reload_done(move || {
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
    }

    fn setup_trigger(&self) {
        let ui_weak = self.ui.clone();
        let lens_model = self.lens_model.clone();
        let tx = self.tx.clone();

        if let Some(ui) = self.ui.upgrade() {
            ui.global::<AppState>().on_reload(move || {
                let paths = lens_model
                    .lens
                    .borrow()
                    .get_locations()
                    .iter()
                    .map(|e| (e.id, e.path.clone()))
                    .collect();

                // update_lens(tx.clone(), ui_weak.clone(), paths);
                Self::spawn_worker(tx.clone(), ui_weak.clone(), paths);

                lens_model.update();
            });
        }
    }

    fn spawn_worker(tx: Sender<LensMessage>, ui_weak: Weak<AppWindow>, paths: Vec<(i32, String)>) {
        thread::spawn(move || {
            println!("Locations: {:?}", paths);

            let dir_s = dir_search::get_all_data(&paths);

            if tx.send(LensMessage::ReloadResult(dir_s)).is_ok() {
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.global::<AppState>().invoke_reload_done();
                    }
                })
                .unwrap();
            }
        });
    }
}
