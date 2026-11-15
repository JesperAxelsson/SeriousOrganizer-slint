use std::{cell::RefCell, rc::Rc};

use serious_organizer_lib::lens::Lens;
use slint::{ComponentHandle, ModelRc, ToSharedString, VecModel, Weak};

use crate::{AppState, AppWindow, AssignRow, AssignState};

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    Unassigned,
    Partial,
    Assigned,
}

impl Mark {
    // Click cycle. Partial can only be reach by going back to the original.
    fn next(self, original: Mark) -> Mark {
        match (original, self) {
            (Mark::Partial, Mark::Partial) => Mark::Assigned,
            (Mark::Partial, Mark::Assigned) => Mark::Unassigned,
            (Mark::Partial, Mark::Unassigned) => Mark::Partial,
            (Mark::Unassigned, Mark::Unassigned) => Mark::Assigned,
            (Mark::Unassigned, _) => Mark::Unassigned,
            (Mark::Assigned, Mark::Assigned) => Mark::Unassigned,
            (Mark::Assigned, _) => Mark::Assigned,
        }
    }

    fn to_state(self) -> AssignState {
        match self {
            Mark::Unassigned => AssignState::Unassigned,
            Mark::Partial => AssignState::Partial,
            Mark::Assigned => AssignState::Assigned,
        }
    }
}

struct Row {
    id: i32,
    name: String,
    original: Mark,
    current: Mark,
}

pub struct LabelAssignController {
    ui: Weak<AppWindow>,
    lens: Rc<RefCell<Lens>>,
    /// The entries the dialog works on, a snapshot from when it was opened
    ids: RefCell<Vec<i32>>,
    rows: RefCell<Vec<Row>>,
}

impl LabelAssignController {
    pub fn new(ui: &AppWindow, lens: Rc<RefCell<Lens>>) -> Self {
        Self {
            ui: ui.as_weak(),
            lens: lens,
            ids: RefCell::new(Vec::new()),
            rows: RefCell::new(Vec::new()),
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

        let rows: Vec<Row> = {
            let lens = self.lens.borrow();
            lens.get_labels()
                .iter()
                .map(|label| {
                    let have = ids
                        .iter()
                        .filter(|e| lens.source.has_label(**e, label.id))
                        .count();
                    let mark = if have == 0 {
                        Mark::Unassigned
                    } else if have == ids.len() {
                        Mark::Assigned
                    } else {
                        Mark::Partial
                    };

                    Row {
                        id: label.id,
                        name: label.name.clone(),
                        original: mark,
                        current: mark,
                    }
                })
                .collect()
        };

        *self.ids.borrow_mut() = ids;
        *self.rows.borrow_mut() = rows;
        self.publish();

        if let Some(ui) = self.ui.upgrade() {
            ui.invoke_open_label_assign(subitle.into());
        }
    }

    /// Stage label change
    pub fn toggle(&self, label_id: i32) {
        {
            let mut rows = self.rows.borrow_mut();
            if let Some(row) = rows.iter_mut().find(|r| r.id == label_id) {
                row.current = row.current.next(row.original);
            }
        }

        self.publish();
    }

    /// Writes all staged changes in one batch. Returns true if anything changes.
    pub fn apply(&self) -> bool {
        let entries: Vec<u32> = self.ids.borrow().iter().map(|e| *e as u32).collect();

        let (add, remove): (Vec<u32>, Vec<u32>) = {
            let rows = self.rows.borrow();
            let add = rows
                .iter()
                .filter(|r| r.current == Mark::Assigned && r.original != Mark::Assigned)
                .map(|r| r.id as u32)
                .collect();
            let remove = rows
                .iter()
                .filter(|r| r.current == Mark::Unassigned && r.original != Mark::Unassigned)
                .map(|r| r.id as u32)
                .collect();

            (add, remove)
        };

        if add.is_empty() && remove.is_empty() {
            return false;
        }

        let mut lens = self.lens.borrow_mut();
        if !add.is_empty() {
            lens.add_entry_labels(entries.clone(), add);
        } else {
            lens.remove_entry_labels(entries.clone(), remove);
        }

        true
    }

    fn publish(&self) {
        let rows: Vec<AssignRow> = self
            .rows
            .borrow()
            .iter()
            .map(|r| AssignRow {
                id: r.id,
                name: r.name.to_shared_string(),
                state: r.current.to_state(),
                changed: r.current != r.original,
            })
            .collect();

        if let Some(ui) = self.ui.upgrade() {
            ui.global::<AppState>()
                .set_assign_rows(ModelRc::new(VecModel::from(rows)));
        }
    }
}
