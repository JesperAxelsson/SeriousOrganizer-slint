use std::cell::{Cell, RefCell};
use std::rc::Rc;

use slint::{Model, ModelNotify, ModelRc, StandardListViewItem, VecModel};

use serious_organizer_lib::lens::Lens;

use crate::helper::format_size;

pub struct FileModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
    entry_ix: Cell<Option<usize>>,
}

impl Model for FileModelList {
    type Data = ModelRc<StandardListViewItem>;

    fn row_count(&self) -> usize {
        if let Some(entry_ix) = self.entry_ix.get() {
            self.lens.borrow().get_file_count(entry_ix).unwrap_or(0)
        } else {
            0
        }
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        // println!("Print row data {row}");
        if let Some(entry_ix) = self.entry_ix.get() {
            if let Some(file) = self.lens.borrow().get_file_entry(entry_ix, row) {
                let items = vec![
                    StandardListViewItem::from(file.name.as_str()),
                    StandardListViewItem::from(file.path.as_str()),
                    StandardListViewItem::from(format_size(file.size).as_str()),
                ];

                let row_model = Rc::new(VecModel::from(items));

                return Some(ModelRc::from(row_model));
            }
        }

        None
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

impl FileModelList {
    pub fn new(lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            lens,
            notify: ModelNotify::default(),
            entry_ix: Cell::new(None),
        }
    }

    pub fn set_entry_ix(&self, entry_ix: i32) {
        println!("Reset the thing!");

        if entry_ix >= 0 {
            self.entry_ix.set(Some(entry_ix as usize));
        } else {
            self.entry_ix.set(None);
        }

        self.notify.reset();
    }
}
