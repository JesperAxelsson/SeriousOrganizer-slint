use std::cell::{Cell, RefCell};
use std::collections::HashSet;

/// What Selection needs to know about the rows a table currently shows
pub trait RowSource {
    fn count(&self) -> usize;
    fn id_at(&self, row: usize) -> Option<i32>;
    fn row_of(&self, id: i32) -> Option<usize>;
}

#[derive(Default)]
pub struct Selection {
    selected: RefCell<HashSet<i32>>,
    anchor: Cell<Option<i32>>,
}

impl Selection {
    pub fn new() -> Self {
        Selection {
            selected: RefCell::new(HashSet::new()),
            anchor: Cell::new(None),
        }
    }

    pub fn contains(&self, id: i32) -> bool {
        self.selected.borrow().contains(&id)
    }

    pub fn ids(&self) -> Vec<i32> {
        self.selected.borrow().iter().copied().collect()
    }

    /// Forrget everything without differing (the caller resets the model anyway)
    pub fn reset(&self) {
        self.selected.borrow_mut().clear();
        self.anchor.set(None);
    }

    /// Drop ids that no longer exists
    pub fn retain(&self, keep: impl Fn(i32) -> bool) {
        self.selected.borrow_mut().retain(|id| keep(*id));
    }

    pub fn click(&self, src: &dyn RowSource, row: usize, ctrl: bool, shift: bool) -> Vec<usize> {
        let Some(id) = src.id_at(row) else {
            return vec![];
        };

        let mut selection = if ctrl | shift {
            self.selected.borrow().clone()
        } else {
            HashSet::new()
        };

        let anchor_row = self.anchor.get().and_then(|a| src.row_of(a));

        match (shift, anchor_row) {
            (true, Some(anchor_row)) => {
                let (from, to) = (anchor_row.min(row), anchor_row.max(row));
                selection.extend((from..=to).filter_map(|ix| src.id_at(ix)));
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

        self.replace(src, selection)
    }

    pub fn select_for_context(&self, src: &dyn RowSource, row: usize) -> Vec<usize> {
        let Some(id) = src.id_at(row) else {
            return vec![];
        };

        if self.contains(id) {
            return vec![];
        }

        self.anchor.set(Some(id));
        self.replace(src, HashSet::from([id]))
    }

    fn replace(&self, src: &dyn RowSource, new: HashSet<i32>) -> Vec<usize> {
        let old = std::mem::replace(&mut *self.selected.borrow_mut(), new);
        let new = self.selected.borrow();

        (0..src.count())
            .filter(|&row| {
                src.id_at(row)
                    .is_some_and(|id| old.contains(&id) != new.contains(&id))
            })
            .collect()
    }
}
