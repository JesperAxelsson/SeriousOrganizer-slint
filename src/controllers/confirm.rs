use slint::{ComponentHandle, Weak};

use crate::{AppWindow, controllers::pending::Pending};

pub struct ConfirmController {
    ui: Weak<AppWindow>,
    pending: Pending<bool>,
}

impl ConfirmController {
    pub fn new(ui: &AppWindow) -> Self {
        Self {
            ui: ui.as_weak(),
            pending: Pending::new(),
        }
    }

    pub fn confirm(&self, title: &str, message: &str, on_yes: impl FnOnce() + 'static) {
        self.show(title, message, "Yes", true, move |yes| {
            if yes {
                on_yes();
            }
        });
    }

    pub fn notify(&self, title: &str, message: &str) {
        self.show(title, message, "OK", false, |_| {});
    }

    pub fn answered(&self, yes: bool, request: i32) {
        self.pending.resolve(request, yes);
    }

    fn show(
        &self,
        title: &str,
        message: &str,
        confirm_text: &str,
        show_cancel: bool,
        callback: impl FnOnce(bool) + 'static,
    ) {
        let request = self.pending.register(callback);
        if let Some(ui) = self.ui.upgrade() {
            ui.invoke_open_confirm(
                title.into(),
                message.into(),
                confirm_text.into(),
                show_cancel,
                request,
            )
        }
    }
}
