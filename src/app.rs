use std::cell::{Cell, RefCell};
use std::rc::Rc;

use slint::{ComponentHandle, SharedString, Weak};

use serious_organizer_lib::lens::{Lens, SortColumn, SortOrder};

use crate::entry_model_list::EntryModelList;
use crate::file_model_list::FileModelList;
use crate::{AppState, AppWindow};

macro_rules! handler {
    ($app:ident . $method:ident ( $($arg:ident),* )) => {{
        let app = $app.clone();
        move |$($arg),*| app.$method($($arg),*)
    }};
}

#[derive(Clone, Copy, Debug)]
pub enum ContextTarget {
    Entry(i32),
    File(i32),
}

pub struct App {
    ui: Weak<AppWindow>,
    lens: Rc<RefCell<Lens>>,

    pub entry_model: Rc<EntryModelList>,
    pub file_model: Rc<FileModelList>,

    context_target: Cell<Option<ContextTarget>>,
}

impl App {
    pub fn new(ui: &AppWindow, lens: Rc<RefCell<Lens>>) -> Rc<Self> {
        Rc::new(App {
            ui: ui.as_weak(),
            lens: lens.clone(),
            entry_model: Rc::new(EntryModelList::new(lens.clone())),
            file_model: Rc::new(FileModelList::new(lens.clone())),
            context_target: Cell::new(None::<ContextTarget>),
        })
    }

    pub fn wire(self: &Rc<Self>, ui: &AppWindow) {
        let state = ui.global::<AppState>();

        // Search
        state.on_search_text_changed(handler!(self.on_search_text_changed(text)));

        // Entry
        state.on_entry_changed(handler!(self.entry_changed(row)));
        state.on_entry_clicked(handler!(self.entry_clicked(row, ctrl, shift)));
        state.on_entry_activated(handler!(self.entry_activated(row)));
        state.on_entry_header_clicked(handler!(self.on_entry_header_clicked(column)));

        ui.on_entry_context_opened(handler!(self.on_entry_context_opened(row)));

        // File
        // state.on_file_changed(handler!(self.file_changed(row)));
        state.on_file_clicked(handler!(self.file_clicked(row, ctrl, shift)));
        state.on_file_activated(handler!(self.file_activated(row)));
        state.on_file_header_clicked(handler!(self.on_file_header_clicked(column)));

        ui.on_file_context_opened(handler!(self.on_file_context_opened(row)));

        // Shared?
        ui.on_context_command(handler!(self.on_context_command(command)));
        ui.on_rename_confirmed(handler!(self.on_rename_confirmed(new_name)));
    }

    fn entry_changed(&self, row: i32) {
        self.file_model.set_entry_ix(row);
    }

    fn entry_clicked(&self, row: i32, ctrl: bool, shift: bool) {
        if row >= 0 {
            self.entry_model.click(row as usize, ctrl, shift);
        }
    }

    fn entry_activated(&self, row: i32) {
        if row < 0 {
            return;
        }

        if let Some(entry) = self.lens.borrow().get_dir_entry(row as usize) {
            if let Err(err) = open::that_detached(&entry.path) {
                eprintln!("Failed to open '{}': {err}", entry.path);
            }
        }
    }

    fn on_entry_context_opened(&self, row: i32) {
        if row >= 0 {
            self.entry_model.select_for_context(row as usize);
            let id = self.lens.borrow().get_dir_entry(row as usize).map(|e| e.id);
            self.context_target.set(id.map(ContextTarget::Entry));
        }
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

                self.entry_model.update();
                self.file_model.update();
            }
            ContextTarget::File(id) => {
                println!("Rename file is not implemented yet. id: {}", id);
            }
        }
    }

    fn on_entry_header_clicked(&self, column: i32) {
        // let state = state_handle.unwrap();
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = ui.global::<AppState>();

        if let Some((sort_column, order, ascending)) = next_sort(
            column,
            state.get_entry_sort_column(),
            state.get_entry_sort_ascending(),
        ) {
            keep_current_entry(&state, &self.lens, || {
                self.lens.borrow_mut().order_by(sort_column, order);
                self.entry_model.update();
            });

            state.set_entry_sort_column(column);
            state.set_entry_sort_ascending(ascending);
        }
    }

    // ** File handler ***

    fn on_file_header_clicked(&self, column: i32) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = ui.global::<AppState>();

        if let Some((sort_column, order, ascending)) = next_sort(
            column,
            state.get_file_sort_column(),
            state.get_file_sort_ascending(),
        ) {
            keep_current_file(&state, &self.lens, || {
                self.lens.borrow_mut().order_files_by(sort_column, order);
                self.file_model.update();
            });

            state.set_file_sort_column(column);
            state.set_file_sort_ascending(ascending);
        }
    }

    // fn entry_changed(&self, row: i32) {
    //     self.file_model.set_entry_ix(row);
    // }

    fn file_clicked(&self, row: i32, ctrl: bool, shift: bool) {
        if row >= 0 {
            self.file_model.click(row as usize, ctrl, shift);
        }
    }

    fn file_activated(&self, row: i32) {
        if row < 0 {
            return;
        }

        if let Some(file) = self.lens.borrow().get_file_entry(row as usize) {
            if let Err(err) = open::that_detached(&file.path) {
                eprintln!("Failed to open '{}': {err}", file.path);
            }
        }
    }

    fn on_file_context_opened(&self, row: i32) {
        if row >= 0 {
            self.file_model.select_for_context(row as usize);
            let id = self
                .lens
                .borrow()
                .get_file_entry(row as usize)
                .map(|e| e.id);
            self.context_target.set(id.map(ContextTarget::File));
        }
    }

    fn on_search_text_changed(&self, text: SharedString) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = ui.global::<AppState>();

        keep_current_entry(&state, &self.lens, || {
            self.lens.borrow_mut().update_search_text(&text);
            self.entry_model.update();
            self.file_model.update();
        });
    }
}

fn keep_current_entry(state: &AppState, lens: &RefCell<Lens>, change: impl FnOnce()) {
    let cur_ix = state.get_current_entry();
    let entry_id = if cur_ix >= 0 {
        lens.borrow().get_dir_entry(cur_ix as usize).map(|e| e.id)
    } else {
        None
    };

    change();

    if let Some(entry_id) = entry_id {
        let new_ix = lens
            .borrow()
            .entry_id_to_ix(entry_id)
            .map(|ix| ix as i32)
            .unwrap_or(-1);
        state.set_current_entry(new_ix);
    }
}

fn keep_current_file(state: &AppState, lens: &RefCell<Lens>, change: impl FnOnce()) {
    let cur_ix = state.get_current_file();
    let file_id = if cur_ix >= 0 {
        lens.borrow().get_file_entry(cur_ix as usize).map(|e| e.id)
    } else {
        None
    };

    change();

    if let Some(file_id) = file_id {
        let new_ix = lens
            .borrow()
            .file_id_to_ix(file_id)
            .map(|ix| ix as i32)
            .unwrap_or(-1);
        state.set_current_file(new_ix);
    }
}

fn next_sort(
    column: i32,
    cur_column: i32,
    cur_ascending: bool,
) -> Option<(SortColumn, SortOrder, bool)> {
    let sort_column = match column {
        0 => SortColumn::Name,
        1 => SortColumn::Path,
        2 => SortColumn::Size,
        _ => return None,
    };

    let ascending = if cur_column == column {
        !cur_ascending
    } else {
        true
    };

    let order = if ascending {
        SortOrder::Asc
    } else {
        SortOrder::Desc
    };

    Some((sort_column, order, ascending))
}
