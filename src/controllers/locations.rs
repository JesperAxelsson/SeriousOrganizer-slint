use std::{cell::RefCell, rc::Rc};

use serious_organizer_lib::lens::Lens;

use crate::MenuEntry;
use crate::{
    controllers::context::{ContextActions, ContextController, Env, menu_entry},
    location_model_list::LocationModelList,
};

#[derive(Clone)]
pub struct LocationKind {
    pub model: Rc<LocationModelList>,
    lens: Rc<RefCell<Lens>>,
}

impl LocationKind {
    pub fn new(lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            model: Rc::new(LocationModelList::new(lens.clone())),
            lens,
        }
    }

    pub fn add(&self) {
        let Some(folder) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let path = folder.to_string_lossy().to_string();
        let name = folder
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        self.lens.borrow_mut().add_location(&name, &path);
        self.model.reload();
    }

    pub fn context_opened(self: &Rc<Self>, context: &ContextController, row: i32) {
        if row < 0 {
            return;
        }

        if let Some(id) = self.model.id_at(row as usize) {
            context.open(self.clone(), id);
        }
    }
}

impl ContextActions for LocationKind {
    fn menu(&self, _env: &Env, _id: i32) -> Vec<MenuEntry> {
        vec![menu_entry("remove", "Remove")]
    }

    fn run(&self, env: &Env, command: &str, id: i32) {
        if command != "remove" {
            return;
        }

        let Some(name) = self.model.name_of(id) else {
            return;
        };

        let lens = self.lens.clone();
        let model = self.model.clone();

        env.confirm.confirm(
            "Remove location",
            &format!("Remove the location '{name}'? Files on disk is not touched"),
            move || {
                lens.borrow_mut().remove_location_id(id);
                model.reload();
            },
        )
    }
}
