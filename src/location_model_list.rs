use crate::LocationData;
use serious_organizer_lib::lens::Lens;
use slint::{ModelRc, SharedString, VecModel};

use crate::{AppWindow, helper};

// pub struct LocationModelList {
//     pub lens: Rc<RefCell<Lens>>,
//     // locations:
// }

pub fn reload_locations(ui: &AppWindow, lens: &Lens) {
    let locations = lens.get_locations();

    let items: Vec<_> = locations
        .into_iter()
        .map(|loc| LocationData {
            id: loc.id,
            name: SharedString::from(loc.name),
            path: SharedString::from(loc.path),
            size_str: SharedString::from(helper::format_size(loc.size)),
        })
        .collect();

    ui.set_locations(ModelRc::new(VecModel::from(items)));
}
