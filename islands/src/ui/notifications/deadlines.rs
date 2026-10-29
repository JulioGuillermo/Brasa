//! When each toast ends: how long a notice stays as a toast, and the clock of those that are timed.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::services::notifications::notice::{LOW, Notice};

const LOW_SECONDS: u64 = 4;
const NORMAL_SECONDS: u64 = 6;
const AFTER_HOVER_SECONDS: u64 = 2;

/// How long it stays as a toast; None: until it is answered or closed.
pub fn time_shown(notice: &Notice) -> Option<Duration> {
    if notice.expire_ms > 0 {
        return Some(Duration::from_millis(notice.expire_ms as u64));
    }
    if notice.is_critical() || !notice.buttons.is_empty() {
        return None;
    }
    let seconds = if notice.urgency == LOW { LOW_SECONDS } else { NORMAL_SECONDS };
    Some(Duration::from_secs(seconds))
}

#[derive(Default)]
pub struct Deadlines {
    due: HashMap<u32, Instant>,
}

impl Deadlines {
    pub fn start(&mut self, notice: &Notice, now: Instant) {
        if let Some(time) = time_shown(notice) {
            self.due.insert(notice.id, now + time);
        }
    }

    pub fn cancel(&mut self, notice_id: u32) {
        self.due.remove(&notice_id);
    }

    pub fn is_empty(&self) -> bool {
        self.due.is_empty()
    }

    pub fn expired(&self, now: Instant) -> Vec<u32> {
        self.due.iter().filter(|(_, due)| **due <= now).map(|(id, _)| *id).collect()
    }

    /// The pointer left: every toast gets at least a little more time.
    pub fn extend_after_hover(&mut self, now: Instant) {
        let latest_start = now + Duration::from_secs(AFTER_HOVER_SECONDS);
        for due in self.due.values_mut() {
            *due = (*due).max(latest_start);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::notifications::notice::{Button, CRITICAL};

    fn notice(id: u32) -> Notice {
        Notice { id, ..Notice::default() }
    }

    #[test]
    fn normal_lasts_six_seconds_and_low_four() {
        assert_eq!(time_shown(&notice(1)), Some(Duration::from_secs(6)));
        assert_eq!(time_shown(&Notice { urgency: LOW, ..notice(1) }), Some(Duration::from_secs(4)));
    }

    #[test]
    fn critical_and_buttons_wait_for_an_answer() {
        assert_eq!(time_shown(&Notice { urgency: CRITICAL, ..notice(1) }), None);
        let button = Button { key: "reply".into(), label: "Reply".into() };
        assert_eq!(time_shown(&Notice { buttons: vec![button], ..notice(1) }), None);
    }

    #[test]
    fn the_app_s_expire_time_wins() {
        let critical_but_timed = Notice { urgency: CRITICAL, expire_ms: 1500, ..notice(1) };
        assert_eq!(time_shown(&critical_but_timed), Some(Duration::from_millis(1500)));
    }

    #[test]
    fn a_toast_expires_when_its_time_is_up() {
        let start = Instant::now();
        let mut deadlines = Deadlines::default();
        deadlines.start(&notice(1), start);
        assert!(deadlines.expired(start + Duration::from_secs(5)).is_empty());
        assert_eq!(deadlines.expired(start + Duration::from_secs(6)), vec![1]);
    }

    #[test]
    fn leaving_the_pointer_gives_two_more_seconds() {
        let start = Instant::now();
        let mut deadlines = Deadlines::default();
        deadlines.start(&notice(1), start);
        let left_at = start + Duration::from_secs(5);
        deadlines.extend_after_hover(left_at);
        assert!(deadlines.expired(left_at + Duration::from_secs(1)).is_empty());
        assert_eq!(deadlines.expired(left_at + Duration::from_secs(2)), vec![1]);
    }

    #[test]
    fn a_far_deadline_is_not_shortened_after_hover() {
        let start = Instant::now();
        let mut deadlines = Deadlines::default();
        deadlines.start(&Notice { expire_ms: 60_000, ..notice(1) }, start);
        deadlines.extend_after_hover(start);
        assert!(deadlines.expired(start + Duration::from_secs(30)).is_empty());
    }
}
