use std::{cell::RefCell, rc::Rc};

use crate::{TableRow, helper::format_size};
use serious_organizer_lib::{lens::Lens, models::Location};
use slint::{Model, ModelNotify, ToSharedString};

// #[derive(Clone)]
pub struct LocationModelList {
    pub lens: Rc<RefCell<Lens>>,
    locations: RefCell<Vec<Location>>,
    notify: ModelNotify,
}

// pub fn reload_locations(ui: &AppWindow, lens: &Lens) {
//     let locations = lens.get_locations();

//     let items: Vec<_> = locations
//         .into_iter()
//         .map(|loc| LocationData {
//             id: loc.id,
//             name: SharedString::from(loc.name),
//             path: SharedString::from(loc.path),
//             size_str: SharedString::from(helper::format_size(loc.size)),
//         })
//         .collect();

//     ui.set_locations(ModelRc::new(VecModel::from(items)));
// }

impl LocationModelList {
    pub fn new(lens: Rc<RefCell<Lens>>) -> Self {
        let locations = lens.borrow().get_locations();
        Self {
            lens,
            locations: RefCell::new(locations),
            notify: ModelNotify::default(),
        }
    }

    pub fn reload(&self) {
        *self.locations.borrow_mut() = self.lens.borrow().get_locations();
        self.notify.reset();
    }

    pub fn id_at(&self, row: usize) -> Option<i32> {
        self.locations.borrow().get(row).map(|l| l.id)
    }

    pub fn name_of(&self, id: i32) -> Option<String> {
        self.locations
            .borrow()
            .iter()
            .find(|l| l.id == id)
            .map(|l| l.name.clone())
    }
}

impl Model for LocationModelList {
    type Data = TableRow;

    fn row_count(&self) -> usize {
        self.locations.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let locations = self.locations.borrow();
        let loc = locations.get(row)?;
        Some(TableRow {
            id: loc.id,
            name: loc.name.to_shared_string(),
            path: loc.path.to_shared_string(),
            size_str: format_size(loc.size),
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
