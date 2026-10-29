//! What happens when a notification leaves: dismissed, answered, expired as a toast, or taken back by its app.

use super::Notices;
use super::desktop;
use crate::services::notifications::server::{CLOSED_BY_APP, DISMISSED, EXPIRED};

impl Notices {
    /// It stops being a toast and stays in the centre, unless its app asked not to keep it.
    pub(super) fn end_toast(&self, notice_id: u32) {
        {
            let mut state = self.state.borrow_mut();
            state.deadlines.cancel(notice_id);
            state.toasts.remove(&notice_id);
        }
        let transient = self.row(notice_id).is_some_and(|row| row.notice().transient);
        if transient {
            self.close(notice_id, EXPIRED);
        }
        self.refresh();
    }

    pub(super) fn delete(&self, notice_id: u32) {
        self.close(notice_id, DISMISSED);
        self.refresh();
    }

    /// A stale one's app no longer answers: the click opens the app instead.
    pub(super) fn press(&self, notice_id: u32, action_key: &str) {
        let Some(row) = self.row(notice_id) else { return };
        let notice = row.notice();
        if row.is_stale() {
            desktop::open_app(&notice.desktop_entry);
            self.close(notice_id, DISMISSED);
        } else {
            self.server.invoked(notice_id, action_key);
            if !notice.resident {
                self.close(notice_id, DISMISSED);
            }
        }
        self.refresh();
    }

    /// Its app closed it: it leaves the live list and the toasts, and waits under the fold.
    pub(super) fn on_take_back(&self, notice_id: u32) {
        let Some(row) = self.row(notice_id) else { return };
        {
            let mut state = self.state.borrow_mut();
            if state.closed_by_app.contains(&notice_id) {
                return;
            }
            state.toasts.remove(&notice_id);
            state.deadlines.cancel(notice_id);
            state.closed_by_app.insert(notice_id);
        }
        row.remove_from_parent();
        row.set_stale(true);
        self.place(&row);
        self.server.closed(notice_id, CLOSED_BY_APP);
        self.refresh();
    }

    pub(super) fn clear_all(&self) {
        let ids: Vec<u32> = self.state.borrow().rows.iter().map(|row| row.id()).collect();
        for notice_id in ids {
            self.close(notice_id, DISMISSED);
        }
        self.refresh();
    }

    /// Deletes it; its app hears of it, unless it closed it itself.
    pub(super) fn close(&self, notice_id: u32, reason: u32) {
        let Some(row) = self.row(notice_id) else { return };
        let closed_by_app = {
            let mut state = self.state.borrow_mut();
            state.rows.retain(|kept| kept.id() != notice_id);
            state.toasts.remove(&notice_id);
            state.deadlines.cancel(notice_id);
            state.closed_by_app.remove(&notice_id)
        };
        row.remove_from_parent();
        if !closed_by_app {
            self.server.closed(notice_id, reason);
        }
    }
}
