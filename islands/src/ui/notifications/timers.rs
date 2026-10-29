//! The clock of the toasts: every quarter second it ends the ones whose time is up, unless the pointer is on them.

use std::time::{Duration, Instant};

use gtk4::glib;

use super::Notices;

const CHECK_EVERY: Duration = Duration::from_millis(250);

impl Notices {
    pub(super) fn watch_deadlines(&self) {
        {
            let mut state = self.state.borrow_mut();
            if state.checking_deadlines {
                return;
            }
            state.checking_deadlines = true;
        }
        let me = self.me.clone();
        glib::timeout_add_local(CHECK_EVERY, move || match me.upgrade() {
            Some(notices) => notices.end_expired_toasts(),
            None => glib::ControlFlow::Break,
        });
    }

    pub(super) fn give_toasts_time_after_hover(&self) {
        self.state.borrow_mut().deadlines.extend_after_hover(Instant::now());
    }

    fn end_expired_toasts(&self) -> glib::ControlFlow {
        let expired = {
            let state = self.state.borrow();
            if state.hovered { Vec::new() } else { state.deadlines.expired(Instant::now()) }
        };
        for notice_id in expired {
            self.end_toast(notice_id);
        }
        let mut state = self.state.borrow_mut();
        if state.deadlines.is_empty() {
            state.checking_deadlines = false;
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    }
}
