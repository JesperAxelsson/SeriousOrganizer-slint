use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use slint::{Model, ModelNotify};

use serious_organizer_lib::lens::Lens;

use crate::TableRow;
use crate::helper::format_size;

pub struct FileModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
    entry_ix: Cell<Option<usize>>,
    /// Selected entry ids. Ids, not rows, so selection survives search/sort/reload.
    selected: RefCell<HashSet<i32>>,
    /// Entry id that shift-click ranges start from
    anchor: Cell<Option<i32>>,
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
            selected: false,
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
            entry_ix: Cell::new(None),
            selected: RefCell::new(HashSet::new()),
            anchor: Cell::new(None),
        }
    }

    pub fn update(&self) {
        // let lens = self.lens.borrow();
        // lens.sort();
        self.notify.reset();
    }

    pub fn set_entry_ix(&self, entry_ix: i32) {
        if entry_ix >= 0 {
            self.entry_ix.set(Some(entry_ix as usize));
            self.lens
                .borrow_mut()
                .set_current_entry_id(Some(entry_ix as usize));
        } else {
            self.entry_ix.set(None);
            self.lens.borrow_mut().set_current_entry_id(None);
        }

        if (entry_ix as usize) != self.entry_ix.get().unwrap_or(usize::MAX) {
            self.selected.borrow_mut().clear();
            self.anchor.replace(None);
        }

        self.notify.reset();
    }

    pub fn click(&self, row: usize, ctrl: bool, shift: bool) {
        let lens = self.lens.borrow();
        let Some(id) = lens.get_dir_entry(row).map(|e| e.id) else {
            return;
        };

        let mut selection = if ctrl | shift {
            self.selected.borrow().clone()
        } else {
            HashSet::new()
        };

        let anchor_row = self.anchor.get().and_then(|a| lens.entry_id_to_ix(a));

        match (shift, anchor_row) {
            (true, Some(anchor_row)) => {
                let (from, to) = if anchor_row <= row {
                    (anchor_row, row)
                } else {
                    (row, anchor_row)
                };

                for ix in from..=to {
                    if let Some(e) = lens.get_dir_entry(ix) {
                        selection.insert(e.id);
                    }
                }
            }
            _ => {
                if ctrl && selection.contains(&id) {
                    selection.remove(&id);
                } else {
                    selection.insert(id);
                }

                self.anchor.set(Some(id));
            }
        }

        drop(lens);
        self.replace_selection(selection);
    }

    fn replace_selection(&self, new: HashSet<i32>) {
        let old = std::mem::replace(&mut *self.selected.borrow_mut(), new);

        let lens = self.lens.borrow();
        let new = self.selected.borrow();
        for row in 0..lens.get_dir_count() {
            if let Some(entry) = lens.get_dir_entry(row) {
                if old.contains(&entry.id) != new.contains(&entry.id) {
                    self.notify.row_changed(row);
                }
            }
        }
    }
}
