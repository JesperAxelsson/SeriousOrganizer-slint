use std::rc::Rc;

use serious_organizer_lib::lens::Lens;

use crate::MenuEntry;
use crate::controllers::context::{ContextController, menu_entry};
use crate::controllers::table::open_path;
use crate::{
    controllers::{context::ContextActions, table::TableKind},
    entry_model_list::EntryModelList,
};

pub struct EntryKind {
    pub model: Rc<EntryModelList>,
}

impl TableKind for EntryKind {
    fn click(&self, row: usize, ctrl: bool, shift: bool) {
        self.model.click(row, ctrl, shift)
    }

    fn select_for_context(&self, row: usize) {
        self.model.select_for_context(row);
    }

    fn refresh(&self) {
        self.model.update();
    }

    fn id_at(&self, lens: &serious_organizer_lib::lens::Lens, row: usize) -> Option<i32> {
        lens.get_dir_entry(row).map(|e| e.id)
    }

    fn path_at(&self, lens: &serious_organizer_lib::lens::Lens, row: usize) -> Option<String> {
        lens.get_dir_entry(row).map(|e| e.path.clone())
    }

    fn row_of(&self, lens: &serious_organizer_lib::lens::Lens, id: i32) -> Option<usize> {
        lens.entry_id_to_ix(id)
    }

    fn sort(
        &self,
        lens: &mut serious_organizer_lib::lens::Lens,
        column: serious_organizer_lib::lens::SortColumn,
        order: serious_organizer_lib::lens::SortOrder,
    ) {
        lens.order_by(column, order);
    }

    fn current(&self, s: &crate::AppState) -> i32 {
        s.get_current_entry()
    }

    fn set_current(&self, s: &crate::AppState, ix: i32) {
        s.set_current_entry(ix);
    }

    fn sort_state(&self, s: &crate::AppState) -> (i32, bool) {
        (s.get_entry_sort_column(), s.get_entry_sort_ascending())
    }

    fn set_sort_state(&self, s: &crate::AppState, column: i32, ascending: bool) {
        s.set_entry_sort_column(column);
        s.set_entry_sort_ascending(ascending);
    }
}

impl ContextActions for EntryKind {
    fn menu(&self) -> Vec<MenuEntry> {
        vec![
            menu_entry("open", "Open"),
            menu_entry("rename", "Rename"),
            menu_entry("move-to-dir", "Move to dir"),
        ]
    }

    fn run(&self, ctx: &ContextController, command: &str, id: i32) {
        match command {
            "open" => {
                let path = ctx
                    .lens()
                    .borrow()
                    .get_dir_entry_by_id(id)
                    .map(|e| e.path.clone());
                if let Some(path) = path {
                    open_path(&path);
                }
            }
            "rename" => ctx.start_rename(),
            _ => eprintln!("Unknown command: {command}"),
        }
    }

    fn name(&self, lens: &serious_organizer_lib::lens::Lens, id: i32) -> Option<String> {
        lens.get_dir_entry_by_id(id).map(|e| e.name.clone())
    }

    fn rename(&self, lens: &mut Lens, id: i32, new_name: &str) -> Result<(), String> {
        let entry = lens
            .get_dir_entry_by_id(id)
            .cloned()
            .ok_or("entry not found")?;
        lens.rename_entry(entry, new_name)
            .map_err(|e| format!("{e:?}"))
    }
}
