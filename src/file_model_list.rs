use std::cell::RefCell;
use std::rc::Rc;

use serious_organizer_lib::models::File;
use slint::{Model, ModelNotify, ModelRc, StandardListViewItem, VecModel};

use serious_organizer_lib::lens::Lens;

use crate::helper::format_size;

pub struct FileModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
    entry_id: Option<usize>,
}

impl Model for FileModelList {
    type Data = ModelRc<StandardListViewItem>;

    fn row_count(&self) -> usize {
        let count = self.lens.borrow().get_dir_count();

        // println!("Row count {count}");
        count
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        // println!("Print row data {row}");
        if let Some(entry) = self.lens.borrow().get_dir_entry(row) {
            let items = vec![
                StandardListViewItem::from(entry.name.as_str()),
                StandardListViewItem::from(entry.path.as_str()),
                StandardListViewItem::from(format_size(entry.size).as_str()),
            ];

            let row_model = Rc::new(VecModel::from(items));

            Some(ModelRc::from(row_model))
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

impl FileModelList {
    pub fn new(lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            lens,
            notify: ModelNotify::default(),
            entry_id: None,
        }
    }

    pub fn set_entry_id(&mut self, entry_id: Option<usize>) {
        println!("Reset the thing!");
        self.entry_id = entry_id;
        self.notify.reset();
    }
}
