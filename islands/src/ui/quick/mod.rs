//! The quick settings panel: it grows out of a bottom corner of the focused monitor.

mod card;
mod catalog;
mod corner;
mod custom;
mod footer;
mod glyphs;
mod grid;
mod items;
mod keys;
mod menu_view;
mod pager;
mod panel;
mod shell_test;
mod sliders;
mod tile;

use std::rc::Rc;

use crate::runtime::events::EventSender;
use panel::Panel;

pub struct Quick {
    panel: Rc<Panel>,
}

impl Quick {
    /// events: for the shell tests (is Wi-Fi on?) it has the workers run.
    pub fn new(app: &gtk4::Application, events: &EventSender) -> Rc<Quick> {
        Rc::new(Quick { panel: Panel::new(app, events) })
    }

    pub fn toggle(&self) {
        self.panel.toggle();
    }

    /// A shell test it asked for finished on a worker.
    pub fn on_tested(&self, test: &str, passed: bool) {
        self.panel.on_tested(test, passed);
    }
}
