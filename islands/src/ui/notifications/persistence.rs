//! The centre across restarts: read when the island starts, written soon after each change.

use gtk4::glib;

use super::Notices;
use crate::services::notifications::store::{self, Saved};

impl Notices {
    /// The centre as it was. Their apps no longer know these notifications, so they are stale.
    pub(super) fn restore(&self) {
        let saved = store::load(&store::default_folder());
        for notice in saved.notices {
            let row = self.new_row(notice);
            row.set_stale(true);
            {
                let mut state = self.state.borrow_mut();
                if saved.closed_by_app.contains(&row.id()) {
                    state.closed_by_app.insert(row.id());
                }
                state.rows.push(row.clone());
                state.last_id = state.last_id.max(row.id());
            }
            self.place(&row);
        }
        if saved.dnd {
            self.toggle_dnd();
        }
    }

    pub(super) fn save_soon(&self) {
        {
            let mut state = self.state.borrow_mut();
            if state.save_pending {
                return;
            }
            state.save_pending = true;
        }
        let me = self.me.clone();
        glib::idle_add_local_once(move || {
            if let Some(notices) = me.upgrade() {
                notices.save();
            }
        });
    }

    fn save(&self) {
        let mut state = self.state.borrow_mut();
        state.save_pending = false;
        let mut closed_by_app: Vec<u32> = state.closed_by_app.iter().copied().collect();
        closed_by_app.sort_unstable();
        let saved = Saved { notices: state.rows.iter().map(|row| row.notice()).collect(), closed_by_app, dnd: state.dnd };
        drop(state);
        store::save_soon(saved);
    }
}
