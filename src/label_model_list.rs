use std::{cell::RefCell, rc::Rc};

use serious_organizer_lib::lens::{LabelState, Lens};
use slint::{Model, ModelNotify, ModelTracker, ToSharedString};

use crate::{FilterState, LabelData};

pub struct LabelModelList {
    pub lens: Rc<RefCell<Lens>>,
    notify: ModelNotify,
}

impl LabelModelList {
    pub fn new(lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            lens,
            notify: ModelNotify::default(),
        }
    }

    pub fn step(&self, row: usize, toward_include: bool) -> bool {
        let (id, state) = {
            let lens = self.lens.borrow();
            let Some(label) = lens.get_labels().get(row) else {
                return false;
            };
            (label.id as u32, label.state)
        };

        {
            let mut lens = self.lens.borrow_mut();
            match (state, toward_include) {
                (LabelState::Exclude, true) | (LabelState::Include, false) => {
                    lens.remove_label_filter(id);
                }
                (LabelState::Unset, true) => lens.add_inlude_label(id),
                (LabelState::Unset, false) => lens.add_exclude_label(id),
                _ => return false,
            }
        }

        self.notify.row_changed(row);
        true
    }

    /// Returns false if the name was empty or already exists
    pub fn add(&self, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() {
            return false;
        }
        let added = self.lens.borrow_mut().add_label(name);
        if added {
            self.notify.reset();
        }
        added
    }

    pub fn clear_filters(&self) {
        self.lens.borrow_mut().clear_label_filters();
        self.notify.reset();
    }

    pub fn exclude_all(&self) {
        self.lens.borrow_mut().exclude_all_labels();
        self.notify.reset();
    }
}

impl Model for LabelModelList {
    type Data = LabelData;

    fn row_count(&self) -> usize {
        self.lens.borrow().get_labels().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let lens = self.lens.borrow();
        let label = lens.get_labels().get(row)?;

        Some(LabelData {
            id: label.id,
            name: label.name.to_shared_string(),
            state: match label.state {
                LabelState::Unset => FilterState::Unset,
                LabelState::Exclude => FilterState::Exclude,
                LabelState::Include => FilterState::Include,
            },
        })
    }

    fn set_row_data(&self, _row: usize, _data: Self::Data) {}

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}
