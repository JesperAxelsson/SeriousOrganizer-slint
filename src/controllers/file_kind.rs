use std::rc::Rc;

use crate::{
    controllers::{context::ContextTarget, table::TableKind},
    file_model_list::FileModelList,
};

pub struct FileKind {
    pub model: Rc<FileModelList>,
}

impl TableKind for FileKind {
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
        lens.get_file_entry(row).map(|e| e.id)
    }

    fn path_at(&self, lens: &serious_organizer_lib::lens::Lens, row: usize) -> Option<String> {
        lens.get_file_entry(row).map(|e| e.path.clone())
    }

    fn row_of(&self, lens: &serious_organizer_lib::lens::Lens, id: i32) -> Option<usize> {
        lens.file_id_to_ix(id)
    }

    fn sort(
        &self,
        lens: &mut serious_organizer_lib::lens::Lens,
        column: serious_organizer_lib::lens::SortColumn,
        order: serious_organizer_lib::lens::SortOrder,
    ) {
        lens.order_files_by(column, order);
    }

    fn context_target(&self, id: i32) -> ContextTarget {
        ContextTarget::File(id)
    }

    fn current(&self, s: &crate::AppState) -> i32 {
        s.get_current_file()
    }

    fn set_current(&self, s: &crate::AppState, ix: i32) {
        s.set_current_file(ix);
    }

    fn sort_state(&self, s: &crate::AppState) -> (i32, bool) {
        (s.get_file_sort_column(), s.get_file_sort_ascending())
    }

    fn set_sort_state(&self, s: &crate::AppState, column: i32, ascending: bool) {
        s.set_file_sort_column(column);
        s.set_file_sort_ascending(ascending);
    }
}
