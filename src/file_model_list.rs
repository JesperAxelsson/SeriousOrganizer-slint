use std::cell::RefCell;
use std::rc::Rc;

use slint::{Model, ModelNotify};

use serious_organizer_lib::lens::Lens;

use crate::TableRow;
use crate::helper::format_size;
use crate::selection::{RowSource, Selection};

struct FileRows<'a>(&'a Lens);

impl RowSource for FileRows<'_> {
    fn count(&self) -> usize {
        self.0.get_file_count().unwrap_or(0)
    }
    fn id_at(&self, row: usize) -> Option<i32> {
        self.0.get_file_entry(row).map(|e| e.id)
    }
    fn row_of(&self, id: i32) -> Option<usize> {
        self.0.file_id_to_ix(id)
    }
}

pub struct FileModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
    selection: Selection,
}

impl Model for FileModelList {
    type Data = TableRow;

    fn row_count(&self) -> usize {
        self.lens.borrow().get_file_count().unwrap_or(0)
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let lens = self.lens.borrow();
        let file = lens.get_file_entry(row)?;

        Some(TableRow {
            id: file.id,
            name: file.name.as_str().into(),
            path: file.path.as_str().into(),
            size_str: format_size(file.size),
            selected: self.selection.contains(file.id),
        })
    }

    fn set_row_data(&self, _row: usize, _data: Self::Data) {}

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl FileModelList {
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
                .retain(|id| lens.get_file_entry_by_id(id).is_some());
        }
        self.notify.reset();
    }

    pub fn set_entry_ix(&self, entry_ix: i32) {
        let new_ix = if entry_ix >= 0 {
            Some(entry_ix as usize)
        } else {
            None
        };

        let old_ix = self
            .lens
            .borrow()
            .get_current_entry_id()
            .and_then(|id| self.lens.borrow().file_id_to_ix(id));

        if new_ix != old_ix {
            self.selection.reset();
            self.lens.borrow_mut().set_current_entry_id(new_ix);
            self.notify.reset();
        }
    }

    pub fn click(&self, row: usize, ctrl: bool, shift: bool) {
        let changed = {
            let lens = self.lens.borrow();
            self.selection.click(&FileRows(&lens), row, ctrl, shift)
        };

        changed.into_iter().for_each(|r| self.notify.row_changed(r));
    }

    pub fn select_for_context(&self, row: usize) {
        let changed = {
            let lens = self.lens.borrow();
            self.selection.select_for_context(&FileRows(&lens), row)
        };

        changed.into_iter().for_each(|r| self.notify.row_changed(r));
    }
}
