use std::cell::RefCell;

use slint::{ComponentHandle, SharedString, Weak};

use crate::AppWindow;

pub struct RenameController {
    ui: Weak<AppWindow>,
    pending: RefCell<Option<Box<dyn FnOnce(String)>>>,
}

impl RenameController {
    pub fn new(ui: &AppWindow) -> Self {
        Self {
            ui: ui.as_weak(),
            pending: RefCell::new(None),
        }
    }

    // Show the dialog with the current name. `on_confirm` runs with the new name.
    pub fn ask(&self, current_name: &str, on_confirm: impl FnOnce(String) + 'static) {
        *self.pending.borrow_mut() = Some(Box::new(on_confirm));
        if let Some(ui) = self.ui.upgrade() {
            ui.invoke_open_rename_dialog(current_name.into());
        }
    }

    pub fn confirmed(&self, new_name: SharedString) {
        // take() into a local first, so the RefCell isn't borrowed while the callback
        // runs (the callback may call ask() again)
        let callback = self.pending.borrow_mut().take();
        if let Some(callback) = callback {
            callback(new_name.into());
        }
    }
}
