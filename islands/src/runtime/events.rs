//! What the reader and worker threads tell the GTK thread. app.rs receives them and hands each to the
//! part of the interface it is for.

use gtk4::gdk;

pub enum Event {
    FocusedMonitor(String),
    /// The default output's volume, 0.0 to 1.0.
    Volume(f64),
    Cover { url: String, picture: Option<gdk::Texture> },
    /// A small picture of an open window, for the window switcher.
    WindowShot { capture_id: String, picture: gdk::Texture },
    /// A quick settings shell test finished.
    Tested { test: String, passed: bool },
}

pub type EventSender = async_channel::Sender<Event>;
pub type EventReceiver = async_channel::Receiver<Event>;

pub fn channel() -> (EventSender, EventReceiver) {
    async_channel::unbounded()
}

/// From any thread. A closed channel only happens while quitting, when nobody listens any more.
pub fn send(events: &EventSender, event: Event) {
    let _ = events.send_blocking(event);
}
