use std::{cell::RefCell, rc::Rc};

use crate::{AppState, AppWindow, MenuEntry};
use serious_organizer_lib::lens::Lens;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};

// #[derive(Clone, Copy, Debug)]
// pub enum ContextTarget {
//     Entry(i32),
//     File(i32),
// }

/// What a table can do to the row that was right-clicked
pub trait ContextActions {
    fn menu(&self) -> Vec<MenuEntry>;
    fn run(&self, ctx: &ContextController, command: &str, id: i32);
    fn name(&self, lens: &Lens, id: i32) -> Option<String>;
    fn rename(&self, lens: &mut Lens, id: i32, new_name: &str) -> Result<(), String>;
}

pub struct ContextTarget {
    owner: Rc<dyn ContextActions>,
    id: i32,
}

pub struct ContextController {
    ui: Weak<AppWindow>,
    lens: Rc<RefCell<Lens>>,
    target: RefCell<Option<ContextTarget>>,
    on_changed: Box<dyn Fn()>,
}

impl ContextController {
    pub fn new(ui: &AppWindow, lens: Rc<RefCell<Lens>>, on_changed: impl Fn() + 'static) -> Self {
        Self {
            ui: ui.as_weak(),
            lens,
            target: RefCell::new(None),
            on_changed: Box::new(on_changed),
        }
    }

    pub fn lens(&self) -> &Rc<RefCell<Lens>> {
        &self.lens
    }

    pub fn changed(&self) {
        (self.on_changed)();
    }

    pub fn open(&self, owner: Rc<dyn ContextActions>, id: i32) {
        if let Some(ui) = self.ui.upgrade() {
            ui.global::<AppState>()
                .set_menu_entries(ModelRc::new(VecModel::from(owner.menu())));
        }

        *self.target.borrow_mut() = Some(ContextTarget { owner, id })
    }

    pub fn command(&self, command: SharedString) {
        let target = self
            .target
            .borrow()
            .as_ref()
            .map(|t| (t.owner.clone(), t.id));
        if let Some((owner, id)) = target {
            owner.run(self, &command, id);
        }
    }

    // Shared service: ask for a new name, rename_confirmed finishes the job
    pub fn start_rename(&self) {
        let name = {
            let target = self.target.borrow();
            let Some(t) = target.as_ref() else { return };
            t.owner.name(&self.lens.borrow(), t.id)
        };
        if let (Some(name), Some(ui)) = (name, self.ui.upgrade()) {
            ui.invoke_open_rename_dialog(name.as_str().into());
        }
    }

    pub fn rename_confirmed(&self, new_name: SharedString) {
        let Some(t) = self.target.borrow_mut().take() else {
            return;
        };

        let result = t.owner.rename(&mut self.lens.borrow_mut(), t.id, &new_name);
        match result {
            Ok(()) => (self.on_changed)(),
            Err(err) => eprintln!("Rename failed: {err}"), // error dialog goes here later
        }
    }
}

pub fn menu_entry(command: &str, label: &str) -> MenuEntry {
    MenuEntry {
        command: command.into(),
        label: label.into(),
    }
}
