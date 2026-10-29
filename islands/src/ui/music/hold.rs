//! A short pause after the user touched a control, so what the system reports back does not undo it.

use std::cell::Cell;
use std::time::{Duration, Instant};

const HOLD: Duration = Duration::from_millis(800);

#[derive(Default)]
pub struct Hold {
    until: Cell<Option<Instant>>,
}

impl Hold {
    pub fn start(&self) {
        self.until.set(Some(Instant::now() + HOLD));
    }

    pub fn is_active(&self) -> bool {
        self.until.get().is_some_and(|until| Instant::now() < until)
    }
}
