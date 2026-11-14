use std::cell::RefCell;
use std::rc::Rc;

use serious_organizer_lib::lens::{Lens, SortColumn, SortOrder};
use slint::{ComponentHandle, Weak};

use crate::controllers::context::{ContextActions, ContextController};
use crate::{AppState, AppWindow};

pub trait TableKind {
    // model
    fn click(&self, row: usize, ctrl: bool, shift: bool);
    fn select_for_context(&self, row: usize);
    fn refresh(&self);
    // lens lookups
    fn id_at(&self, lens: &Lens, row: usize) -> Option<i32>;
    fn path_at(&self, lens: &Lens, row: usize) -> Option<String>;
    fn row_of(&self, lens: &Lens, id: i32) -> Option<usize>;
    fn sort(&self, lens: &mut Lens, column: SortColumn, order: SortOrder);
    // ui state
    fn current(&self, s: &AppState) -> i32;
    fn set_current(&self, s: &AppState, ix: i32);
    fn sort_state(&self, s: &AppState) -> (i32, bool);
    fn set_sort_state(&self, s: &AppState, column: i32, ascending: bool);
}

pub struct TableController<K: TableKind + 'static> {
    ui: Weak<AppWindow>,
    lens: Rc<RefCell<Lens>>,
    context: Rc<ContextController>,
    pub kind: Rc<K>,
}

impl<K: TableKind + ContextActions + 'static> TableController<K> {
    pub fn new(
        ui: &AppWindow,
        lens: Rc<RefCell<Lens>>,
        context: Rc<ContextController>,
        kind: Rc<K>,
    ) -> Self {
        TableController {
            ui: ui.as_weak(),
            lens: lens,
            context: context,
            kind: kind,
        }
    }

    pub fn clicked(&self, row: i32, ctrl: bool, shift: bool) {
        if row >= 0 {
            self.kind.click(row as usize, ctrl, shift);
        }
    }

    pub fn activated(&self, row: i32) {
        if row < 0 {
            return;
        }
        let path = self.kind.path_at(&self.lens.borrow(), row as usize);
        if let Some(path) = path {
            open_path(&path);
        }
    }

    pub fn context_opened(&self, row: i32) {
        if row < 0 {
            return;
        }
        self.kind.select_for_context(row as usize);
        if let Some(id) = self.kind.id_at(&self.lens.borrow(), row as usize) {
            self.context.open(self.kind.clone(), id);
        }
    }

    pub fn header_clicked(&self, column: i32) {
        let Some(ui) = self.ui.upgrade() else { return };
        let state = ui.global::<AppState>();

        let (cur_column, cur_ascending) = self.kind.sort_state(&state);
        let Some((sort_column, order, ascending)) = next_sort(column, cur_column, cur_ascending)
        else {
            return;
        };

        self.keep_current(&state, || {
            self.kind
                .sort(&mut self.lens.borrow_mut(), sort_column, order);
            self.kind.refresh();
        });

        self.kind.set_sort_state(&state, column, ascending);
    }

    pub fn keep_current(&self, state: &AppState, change: impl FnOnce()) {
        let cur = self.kind.current(state);
        let id = if cur >= 0 {
            self.kind.id_at(&self.lens.borrow(), cur as usize)
        } else {
            None
        };

        change();

        if let Some(id) = id {
            let ix = self
                .kind
                .row_of(&self.lens.borrow(), id)
                .map(|ix| ix as i32)
                .unwrap_or(-1);
            self.kind.set_current(state, ix);
        }
    }
}

pub fn open_path(path: &str) {
    if let Err(err) = open::that_detached(path) {
        eprintln!("Failed to open '{path}' {err}");
    }
}

fn next_sort(
    column: i32,
    cur_column: i32,
    cur_ascending: bool,
) -> Option<(SortColumn, SortOrder, bool)> {
    let sort_column = match column {
        0 => SortColumn::Name,
        1 => SortColumn::Path,
        2 => SortColumn::Size,
        _ => return None,
    };

    let ascending = if cur_column == column {
        !cur_ascending
    } else {
        true
    };

    let order = if ascending {
        SortOrder::Asc
    } else {
        SortOrder::Desc
    };

    Some((sort_column, order, ascending))
}
