//! What happens when an app sends a notification: it gets an id (or takes over an earlier one), becomes
//! a row and a toast, and the oldest ones beyond the limit go.

use std::time::Instant;

use super::row::{Callbacks, NoticeRow};
use super::{MAX_KEPT, Notices};
use crate::services::notifications::notice::Notice;
use crate::services::notifications::server::DISMISSED;
use crate::services::notifications::sound;

impl Notices {
    /// Runs inside the D-Bus reply: nothing slow in here.
    pub(super) fn on_notify(&self, mut notice: Notice) -> u32 {
        notice.id = self.replaced_id(&notice).unwrap_or_else(|| self.new_id());
        let notice_id = notice.id;
        match self.row(notice_id) {
            None => self.add_row(notice.clone()),
            Some(row) => {
                self.state.borrow_mut().deadlines.cancel(notice_id);
                row.show_notice(notice.clone());
                self.bring_back(&row);
            }
        }
        self.show_as_toast_unless_muted(&notice);
        self.keep_at_most(MAX_KEPT);
        self.refresh();
        notice_id
    }

    pub(super) fn new_row(&self, notice: Notice) -> std::rc::Rc<NoticeRow> {
        let on_close = self.me.clone();
        let on_press = self.me.clone();
        NoticeRow::new(
            notice,
            Callbacks {
                on_close: Box::new(move |notice_id| {
                    if let Some(notices) = on_close.upgrade() {
                        notices.delete(notice_id);
                    }
                }),
                on_press: Box::new(move |notice_id, action_key| {
                    if let Some(notices) = on_press.upgrade() {
                        notices.press(notice_id, action_key);
                    }
                }),
            },
        )
    }

    fn add_row(&self, notice: Notice) {
        let row = self.new_row(notice);
        self.state.borrow_mut().rows.push(row.clone());
        self.place(&row);
    }

    fn show_as_toast_unless_muted(&self, notice: &Notice) {
        let muted = self.state.borrow().dnd && !notice.is_critical();
        if muted {
            self.state.borrow_mut().toasts.remove(&notice.id);
            return;
        }
        {
            let mut state = self.state.borrow_mut();
            state.toasts.insert(notice.id);
            state.deadlines.start(notice, Instant::now());
        }
        self.watch_deadlines();
        sound::chime(notice);
    }

    /// An app sent again a notification it had taken back: it is live again.
    fn bring_back(&self, row: &std::rc::Rc<NoticeRow>) {
        row.set_stale(false);
        let was_taken_back = self.state.borrow_mut().closed_by_app.remove(&row.id());
        if was_taken_back {
            row.remove_from_parent();
            self.place(row);
        }
    }

    /// The oldest of those their apps took back go first, then the oldest of all.
    fn keep_at_most(&self, count: usize) {
        loop {
            let oldest = {
                let state = self.state.borrow();
                if state.rows.len() <= count {
                    return;
                }
                let taken_back = state.rows.iter().find(|row| state.closed_by_app.contains(&row.id()));
                taken_back.or(state.rows.first()).map(|row| row.id())
            };
            let Some(oldest) = oldest else { return };
            self.close(oldest, DISMISSED);
        }
    }

    fn new_id(&self) -> u32 {
        let mut state = self.state.borrow_mut();
        state.last_id += 1;
        state.last_id
    }

    fn replaced_id(&self, notice: &Notice) -> Option<u32> {
        if self.row(notice.replaces_id).is_some() {
            return Some(notice.replaces_id);
        }
        if notice.synchronous.is_empty() {
            return None;
        }
        let state = self.state.borrow();
        let same_hint = state.rows.iter().find(|row| row.notice().synchronous == notice.synchronous);
        same_hint.map(|row| row.id())
    }
}
