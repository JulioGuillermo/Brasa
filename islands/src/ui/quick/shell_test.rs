//! Asks a shell test on a worker thread; the answer comes back as an Event::Tested.

use crate::runtime::events::{self, Event, EventSender};
use crate::runtime::workers;
use crate::services::process;

pub fn ask(events: &EventSender, test: &str) {
    let events = events.clone();
    let test = test.to_string();
    workers::run(move || {
        let passed = process::passes(&test);
        events::send(&events, Event::Tested { test, passed });
    });
}
