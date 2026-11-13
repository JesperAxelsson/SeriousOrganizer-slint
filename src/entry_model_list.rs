use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use slint::{Model, ModelNotify};

use serious_organizer_lib::lens::Lens;

use crate::TableRow;
use crate::helper::format_size;

pub struct LensModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
    /// Selected entry ids. Ids, not rows, so selection survives search/sort/reload.
    selected: RefCell<HashSet<i32>>,
    /// Entry id that shift-click ranges start from
    anchor: Cell<Option<i32>>,
}

impl Model for LensModelList {
    type Data = TableRow;

    fn row_count(&self) -> usize {
        let count = self.lens.borrow().get_dir_count();

        // println!("Row count {count}");
        count
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        // println!("Print row data {row}");
        if let Some(entry) = self.lens.borrow().get_dir_entry(row) {
            return Some(TableRow {
                id: entry.id,
                name: entry.name.as_str().into(),
                path: entry.path.as_str().into(),
                size_str: format_size(entry.size).into(),
                selected: self.selected.borrow().contains(&entry.id),
            });
        } else {
            None
        }
    }

    fn set_row_data(&self, _row: usize, _data: Self::Data) {
        // self.array.borrow_mut()[row] = data;
        // self.notify.row_changed(row);
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl LensModelList {
    pub fn new(lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            lens,
            notify: ModelNotify::default(),
            selected: RefCell::new(HashSet::new()),
            anchor: Cell::new(None),
        }
    }

    pub fn update(&self) {
        // Drop ids of entries that no longer exist (e.g. after a reload).
        // Entries that are only filtered out by a search stay selected.
        {
            let lens = self.lens.borrow();
            self.selected
                .borrow_mut()
                .retain(|id| lens.get_dir_entry_by_id(*id).is_some());
        }

        // println!("Reset the thing!");
        self.notify.reset();
    }

    pub fn selected_ids(&self) -> Vec<i32> {
        self.selected.borrow().iter().copied().collect()
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

    pub fn select_for_context(&self, row: usize) {
        let id = match self.lens.borrow().get_dir_entry(row) {
            Some(e) => e.id,
            None => return,
        };

        if !self.selected.borrow().contains(&id) {
            self.anchor.set(Some(id));
            self.replace_selection(HashSet::from([id]));
        }
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
