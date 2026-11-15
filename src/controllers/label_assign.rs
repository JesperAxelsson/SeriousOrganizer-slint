use std::{cell::RefCell, rc::Rc};

use serious_organizer_lib::lens::Lens;
use slint::{ComponentHandle, ModelRc, ToSharedString, VecModel, Weak};

use crate::{AppState, AppWindow, AssignRow, AssignState};

pub struct LabelAssignController {
    ui: Weak<AppWindow>,
    lens: Rc<RefCell<Lens>>,
    ids: RefCell<Vec<i32>>,
}

impl LabelAssignController {
    pub fn new(ui: &AppWindow, lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            ui: ui.as_weak(),
            lens: lens,
            ids: RefCell::new(Vec::new()),
        }
    }

    pub fn open(&self, ids: Vec<i32>) {
        if ids.is_empty() {
            return;
        }

        let subitle = if ids.len() == 1 {
            "1 entry selected".to_string()
        } else {
            format!("{} entries selected", ids.len())
        };

        *self.ids.borrow_mut() = ids;
        self.refresh_rows();

        if let Some(ui) = self.ui.upgrade() {
            ui.invoke_open_label_assign(subitle.into());
        }
    }

    pub fn toggle(&self, label_id: i32) {
        let ids = self.ids.borrow().clone();
        {
            let mut lens = self.lens.borrow_mut();
            let all = ids.iter().all(|e| lens.source.has_label(*e, label_id));
            let entries: Vec<u32> = ids.iter().map(|e| *e as u32).collect();

            if all {
                lens.remove_entry_labels(entries, vec![label_id as u32]);
            } else {
                lens.add_entry_labels(entries, vec![label_id as u32]);
            }
        }

        self.refresh_rows();
    }

    fn refresh_rows(&self) {
        let rows: Vec<AssignRow> = {
            let lens = self.lens.borrow();
            let ids = self.ids.borrow();

            lens.get_labels()
                .iter()
                .map(|label| {
                    let have = ids
                        .iter()
                        .filter(|e| lens.source.has_label(**e, label.id))
                        .count();
                    let state = if have == 0 {
                        AssignState::Unassigned
                    } else if have == ids.len() {
                        AssignState::Assigned
                    } else {
                        AssignState::Partial
                    };
                    AssignRow {
                        id: label.id,
                        name: label.name.to_shared_string(),
                        state,
                    }
                })
                .collect()
        };

        if let Some(ui) = self.ui.upgrade() {
            ui.global::<AppState>()
                .set_assign_rows(ModelRc::new(VecModel::from(rows)));
        }
    }
}
