use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, SharedString, Weak};

use serious_organizer_lib::lens::Lens;

use crate::controllers::confirm::ConfirmController;
use crate::controllers::context::{ContextController, Env};
use crate::controllers::entry_kind::EntryKind;
use crate::controllers::file_kind::FileKind;
use crate::controllers::locations::LocationKind;
use crate::controllers::rename::RenameController;
use crate::controllers::table::{TableController, TableKind};
use crate::entry_model_list::EntryModelList;
use crate::file_model_list::FileModelList;
use crate::label_model_list::LabelModelList;
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
    pub location: Rc<LocationKind>,
    pub labels: Rc<LabelModelList>,

    rename: Rc<RenameController>,
    context: Rc<ContextController>,
    confirm: Rc<ConfirmController>,
}

impl App {
    pub fn new(ui: &AppWindow, lens: Rc<RefCell<Lens>>) -> Rc<Self> {
        let entry_model = Rc::new(EntryModelList::new(lens.clone()));
        let file_model = Rc::new(FileModelList::new(lens.clone()));

        let rename = Rc::new(RenameController::new(ui));
        let confirm = Rc::new(ConfirmController::new(ui));

        let env = Env {
            lens: lens.clone(),
            rename: rename.clone(),
            changed: Rc::new({
                let (e, f) = (entry_model.clone(), file_model.clone());
                move || {
                    e.update();
                    f.update();
                }
            }),
            confirm: confirm.clone(),
        };

        let context = Rc::new(ContextController::new(ui, env));

        Rc::new(App {
            ui: ui.as_weak(),
            lens: lens.clone(),
            entry_table: TableController::new(
                ui,
                lens.clone(),
                context.clone(),
                Rc::new(EntryKind {
                    model: entry_model.clone(),
                }),
            ),
            file_table: TableController::new(
                ui,
                lens.clone(),
                context.clone(),
                Rc::new(FileKind {
                    model: file_model.clone(),
                }),
            ),
            location: Rc::new(LocationKind::new(lens.clone())),
            labels: Rc::new(LabelModelList::new(lens.clone())),
            context: context,
            rename: rename,
            confirm: confirm,
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

        // Locations
        ui.on_add_location(handler!(self.location.add()));
        ui.on_location_context_opened({
            let app = self.clone();
            move |row| app.location.context_opened(&app.context, row)
        });

        // Context Menu
        ui.on_context_command(handler!(self.context.command(command)));
        ui.on_rename_confirmed(handler!(self.rename.confirmed(new_name)));

        // Confirm Dialog
        ui.on_confirm_answered(handler!(self.confirm.answered(yes, request)));

        // Label
        state.on_label_stepped(handler!(self.on_label_stepped(row, toward_include)));
        state.on_label_add_requested(handler!(self.on_label_add(name)));
        state.on_label_filters_reset(handler!(self.labels.clear_filters()));
        state.on_label_filters_exclude_all(handler!(self.labels.exclude_all()));
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

    fn on_label_stepped(&self, row: i32, toward_include: bool) {
        if row < 0 {
            return;
        }

        let Some(ui) = self.ui.upgrade() else {
            return;
        };

        let state = ui.global::<AppState>();

        self.entry_table.keep_current(&state, || {
            if self.labels.step(row as usize, toward_include) {
                self.entry_table.kind.refresh();
                self.file_table.kind.refresh();
            }
        });
    }

    fn on_label_add(&self, name: SharedString) {
        if !self.labels.add(&name) {
            self.confirm.notify(
                "Label not added",
                &format!("A label named '{name}' already exists."),
            );
        }
    }
}
