use std::{cell::RefCell, rc::Rc};

use crate::{
    AppState, AppWindow, MenuEntry,
    controllers::{confirm::ConfirmController, rename::RenameController},
};
use serious_organizer_lib::lens::Lens;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};

pub struct Env {
    pub lens: Rc<RefCell<Lens>>,
    pub rename: Rc<RenameController>,
    pub confirm: Rc<ConfirmController>,
    pub changed: Rc<dyn Fn()>,
}

/// What a table can do to the row that was right-clicked
pub trait ContextActions {
    fn menu(&self) -> Vec<MenuEntry>;
    fn run(&self, ctx: &Env, command: &str, id: i32);
}

pub struct ContextTarget {
    owner: Rc<dyn ContextActions>,
    id: i32,
}

pub struct ContextController {
    ui: Weak<AppWindow>,
    env: Env,
    target: RefCell<Option<ContextTarget>>,
}

impl ContextController {
    pub fn new(ui: &AppWindow, env: Env) -> Self {
        Self {
            ui: ui.as_weak(),
            env,
            target: RefCell::new(None),
        }
    }

    pub fn open(&self, owner: Rc<dyn ContextActions>, id: i32) {
        if let Some(ui) = self.ui.upgrade() {
            ui.global::<AppState>()
                .set_menu_entries(ModelRc::new(VecModel::from(owner.menu())));
        }

        *self.target.borrow_mut() = Some(ContextTarget { owner, id })
    }

    pub fn command(&self, command: SharedString) {
        let target = self.target.borrow_mut().take();
        if let Some(t) = target {
            t.owner.run(&self.env, &command, t.id);
        }
    }
}

pub fn menu_entry(command: &str, label: &str) -> MenuEntry {
    MenuEntry {
        command: command.into(),
        label: label.into(),
    }
}
