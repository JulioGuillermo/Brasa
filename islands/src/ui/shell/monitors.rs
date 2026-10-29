//! The monitors as GTK sees them, and which one has the focus (the compositor reader keeps it current).

use std::cell::RefCell;

use gtk4::gdk;
use gtk4::prelude::*;

thread_local! {
    static FOCUSED: RefCell<String> = const { RefCell::new(String::new()) };
}

pub fn set_focused(connector: String) {
    FOCUSED.with(|focused| *focused.borrow_mut() = connector);
}

pub fn focused_connector() -> String {
    FOCUSED.with(|focused| focused.borrow().clone())
}

pub fn find(connector: &str) -> Option<gdk::Monitor> {
    let display = gdk::Display::default()?;
    let monitors = display.monitors();
    (0..monitors.n_items())
        .filter_map(|index| monitors.item(index)?.downcast::<gdk::Monitor>().ok())
        .find(|monitor| monitor.connector().as_deref() == Some(connector))
}

pub fn focused_or_first() -> Option<gdk::Monitor> {
    find(&focused_connector()).or_else(|| {
        let monitors = gdk::Display::default()?.monitors();
        monitors.item(0)?.downcast::<gdk::Monitor>().ok()
    })
}
