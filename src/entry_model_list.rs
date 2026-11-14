use std::cell::RefCell;
use std::rc::Rc;

use slint::{Model, ModelNotify};

use serious_organizer_lib::lens::Lens;

use crate::TableRow;
use crate::helper::format_size;
use crate::selection::{RowSource, Selection};

struct EntryRows<'a>(&'a Lens);

impl RowSource for EntryRows<'_> {
    fn count(&self) -> usize {
        self.0.get_dir_count()
    }
    fn id_at(&self, row: usize) -> Option<i32> {
        self.0.get_dir_entry(row).map(|e| e.id)
    }
    fn row_of(&self, id: i32) -> Option<usize> {
        self.0.entry_id_to_ix(id)
    }
}

pub struct EntryModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
    selection: Selection,
}

impl Model for EntryModelList {
    type Data = TableRow;

    fn row_count(&self) -> usize {
        self.lens.borrow().get_dir_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        if let Some(entry) = self.lens.borrow().get_dir_entry(row) {
            return Some(TableRow {
                id: entry.id,
                name: entry.name.as_str().into(),
                path: entry.path.as_str().into(),
                size_str: format_size(entry.size).into(),
                selected: self.selection.contains(entry.id),
            });
        } else {
            None
        }
    }

    fn set_row_data(&self, _row: usize, _data: Self::Data) {}

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl EntryModelList {
    pub fn new(lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            lens,
            notify: ModelNotify::default(),
            selection: Selection::new(),
        }
    }

    pub fn update(&self) {
        {
            let lens = self.lens.borrow();
            self.selection
                .retain(|id| lens.get_dir_entry_by_id(id).is_some());
        }

        self.notify.reset();
    }

    pub fn selected_ids(&self) -> Vec<i32> {
        self.selection.ids()
    }

    pub fn click(&self, row: usize, ctrl: bool, shift: bool) {
        let changed = {
            let lens = self.lens.borrow();
            self.selection.click(&EntryRows(&lens), row, ctrl, shift)
        };

        changed.into_iter().for_each(|r| self.notify.row_changed(r));
    }

    pub fn select_for_context(&self, row: usize) {
        let changed = {
            let lens = self.lens.borrow();
            self.selection.select_for_context(&EntryRows(&lens), row)
        };

        changed.into_iter().for_each(|r| self.notify.row_changed(r));
    }
}
