//! The quick settings panel: it grows out of a bottom corner of its monitor, or of the focused one.
//! With quick_hover, resting the pointer in that corner opens it too.

mod card;
mod catalog;
mod corner;
mod custom;
mod footer;
mod glyphs;
mod grid;
mod hot_corner;
mod items;
mod keys;
mod menu_view;
mod pager;
mod panel;
mod shell_test;
mod sliders;
mod tile;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk;
use gtk4::prelude::*;

use crate::config::{self, Side};
use crate::runtime::events::EventSender;
use crate::ui::shell::monitors;
use hot_corner::HotCorner;
use panel::Panel;

pub struct Quick {
    app: gtk4::Application,
    panel: Rc<Panel>,
    hot_corners: RefCell<Vec<HotCorner>>,
}

impl Quick {
    /// events: for the shell tests (is Wi-Fi on?) it has the workers run.
    pub fn new(app: &gtk4::Application, events: &EventSender) -> Rc<Quick> {
        let quick = Rc::new(Quick { app: app.clone(), panel: Panel::new(app, events), hot_corners: RefCell::new(Vec::new()) });
        quick.place_hot_corners();
        quick.follow_monitors();
        quick
    }

    /// island.conf again: the hot corners go where it now says.
    pub fn reload(&self) {
        self.place_hot_corners();
    }

    /// On its own monitor, or on every one when it has none.
    fn place_hot_corners(&self) {
        self.hot_corners.borrow_mut().clear();
        let settings = config::load();
        if !settings.quick_hover {
            return;
        }
        let on_right = settings.quick_side == Side::Right;
        let chosen = monitors::find(&settings.quick_monitor);
        let corners_on = match chosen {
            Some(monitor) => vec![monitor],
            None => monitors::all(),
        };
        for monitor in corners_on {
            let panel = Rc::clone(&self.panel);
            let corner = HotCorner::new(&self.app, &monitor, on_right, move |monitor: &gdk::Monitor| {
                panel.open_by_pointer(monitor);
            });
            self.hot_corners.borrow_mut().push(corner);
        }
    }

    fn follow_monitors(self: &Rc<Self>) {
        let Some(display) = gdk::Display::default() else { return };
        let me = Rc::downgrade(self);
        display.monitors().connect_items_changed(move |_, _, _, _| {
            if let Some(quick) = me.upgrade() {
                quick.place_hot_corners();
            }
        });
    }

    pub fn toggle(&self) {
        self.panel.toggle();
    }

    /// A shell test it asked for finished on a worker.
    pub fn on_tested(&self, test: &str, passed: bool) {
        self.panel.on_tested(test, passed);
    }
}
