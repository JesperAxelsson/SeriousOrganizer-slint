use std::cell::{Cell, RefCell};
use std::rc::Rc;

use slint::{ComponentHandle, SharedString, Weak};

use serious_organizer_lib::lens::Lens;

use crate::controllers::context::ContextTarget;
use crate::controllers::entry_kind::EntryKind;
use crate::controllers::file_kind::FileKind;
use crate::controllers::table::{TableController, TableKind};
use crate::entry_model_list::EntryModelList;
use crate::file_model_list::FileModelList;
use crate::{AppState, AppWindow};

macro_rules! handler {
    ($app:ident . $field:ident . $method:ident ( $($arg:ident),* )) => {{
        let app = $app.clone();
        move |$($arg),*| app.$field.$method($($arg),*)
    }};
    ($app:ident . $method:ident ( $($arg:ident),* )) => {{
        let app = $app.clone();
        move |$($arg),*| app.$method($($arg),*)
    }}
}

pub struct App {
    ui: Weak<AppWindow>,
    lens: Rc<RefCell<Lens>>,

    pub entry_table: TableController<EntryKind>,
    pub file_table: TableController<FileKind>,

    context_target: Rc<Cell<Option<ContextTarget>>>,
}

impl App {
    pub fn new(ui: &AppWindow, lens: Rc<RefCell<Lens>>) -> Rc<Self> {
        let entry_model = Rc::new(EntryModelList::new(lens.clone()));
        let file_model = Rc::new(FileModelList::new(lens.clone()));
        let target = Rc::new(Cell::new(None::<ContextTarget>));

        Rc::new(App {
            ui: ui.as_weak(),
            lens: lens.clone(),
            entry_table: TableController::new(
                ui,
                lens.clone(),
                target.clone(),
                EntryKind {
                    model: entry_model.clone(),
                },
            ),
            file_table: TableController::new(
                ui,
                lens.clone(),
                target.clone(),
                FileKind {
                    model: file_model.clone(),
                },
            ),
            context_target: target,
        })
    }

    pub fn wire(self: &Rc<Self>, ui: &AppWindow) {
        let state = ui.global::<AppState>();

        // Search
        state.on_search_text_changed(handler!(self.on_search_text_changed(text)));

        // Entry
        let file_model = self.file_table.kind.model.clone();
        state.on_entry_changed(handler!(file_model.set_entry_ix(row)));
        state.on_entry_clicked(handler!(self.entry_table.clicked(row, ctrl, shift)));
        state.on_entry_activated(handler!(self.entry_table.activated(row)));
        state.on_entry_header_clicked(handler!(self.entry_table.header_clicked(column)));

        ui.on_entry_context_opened(handler!(self.entry_table.context_opened(row)));

        // File
        // state.on_file_changed(handler!(self.file_changed(row)));
        state.on_file_clicked(handler!(self.file_table.clicked(row, ctrl, shift)));
        state.on_file_activated(handler!(self.file_table.activated(row)));
        state.on_file_header_clicked(handler!(self.file_table.header_clicked(column)));

        ui.on_file_context_opened(handler!(self.file_table.context_opened(row)));

        // Shared?
        ui.on_context_command(handler!(self.on_context_command(command)));
        ui.on_rename_confirmed(handler!(self.on_rename_confirmed(new_name)));
    }

    /// Handles both file and entry context menu
    fn on_context_command(&self, command: SharedString) {
        println!("Got command: '{command}'");
        if command != "Rename" {
            return;
        }

        let Some(context_target) = self.context_target.get() else {
            return;
        };

        let name = {
            let lens = self.lens.borrow();
            match context_target {
                ContextTarget::Entry(id) => lens.get_dir_entry_by_id(id).map(|e| e.name.clone()),
                ContextTarget::File(id) => lens
                    .file_id_to_ix(id)
                    .and_then(|ix| lens.get_file_entry(ix))
                    .map(|e| e.name.clone()),
            }
        };

        if let (Some(name), Some(ui)) = (name, self.ui.upgrade()) {
            ui.invoke_open_rename_dialog(name.as_str().into());
        }
    }

    fn on_rename_confirmed(&self, new_name: SharedString) {
        let Some(target) = self.context_target.get() else {
            return;
        };

        println!("Rename confirmed {:?}", target);

        match target {
            ContextTarget::Entry(id) => {
                {
                    let mut lens = self.lens.borrow_mut();
                    let Some(entry) = lens.get_dir_entry_by_id(id).cloned() else {
                        return;
                    };

                    println!("Rename from {} to {}", entry.name, &new_name);
                    if let Err(err) = lens.rename_entry(entry, &new_name) {
                        eprintln!("Error while renaming file: {:?}", err);
                        println!("Show error dialog...");
                    }
                }

                self.entry_table.kind.refresh();
                self.file_table.kind.refresh();
            }
            ContextTarget::File(id) => {
                println!("Rename file is not implemented yet. id: {}", id);
            }
        }
    }

    // ** Search handler ***
    fn on_search_text_changed(&self, text: SharedString) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = ui.global::<AppState>();

        self.entry_table.keep_current(&state, || {
            self.lens.borrow_mut().update_search_text(&text);
            self.entry_table.kind.refresh();
            self.file_table.kind.refresh();
        });
    }
}
