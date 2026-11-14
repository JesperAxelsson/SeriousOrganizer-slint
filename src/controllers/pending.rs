use std::cell::{Cell, RefCell};

pub struct Pending<T> {
    next_id: Cell<i32>,
    slot: RefCell<Option<(i32, Box<dyn FnOnce(T)>)>>,
}

impl<T> Pending<T> {
    pub fn new() -> Self {
        Self {
            next_id: Cell::new(0),
            slot: RefCell::new(None),
        }
    }

    pub fn register(&self, callback: impl FnOnce(T) + 'static) -> i32 {
        let id = self.next_id.get() + 1;
        self.next_id.set(id);
        *self.slot.borrow_mut() = Some((id, Box::new(callback)));
        id
    }

    pub fn resolve(&self, id: i32, value: T) {
        // Take the closure out before calling it. It may ask a new question.
        let callback = {
            let mut slot = self.slot.borrow_mut();
            match slot.take() {
                Some((pending_id, cb)) if pending_id == id => Some(cb),
                other => {
                    *slot = other;
                    None
                }
            }
        };

        if let Some(callback) = callback {
            callback(value);
        }
    }
}
