//! The quick settings panel: it grows out of a bottom corner of its monitor, or of the focused one.
//! With quick_hover, resting the pointer in that corner opens it too.

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

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk;
use gtk4_layer_shell::Edge;

use crate::config::{self, Side};
use crate::runtime::events::EventSender;
use crate::ui::shell::hot_zone::{self, HotZone, Spot};
use panel::Panel;

const HOT_CORNER_PIXELS: i32 = 8;
const OVERHANG: i32 = corner::OVERHANG as i32;

pub struct Quick {
    app: gtk4::Application,
    panel: Rc<Panel>,
    hot_corners: RefCell<Vec<HotZone>>,
}

impl Quick {
    /// events: for the shell tests (is Wi-Fi on?) it has the workers run.
    pub fn new(app: &gtk4::Application, events: &EventSender) -> Rc<Quick> {
        let quick = Rc::new(Quick { app: app.clone(), panel: Panel::new(app, events), hot_corners: RefCell::new(Vec::new()) });
        quick.place_hot_corners();
        quick
    }

    /// island.conf again, or the monitors changed: the hot corners go where it now says.
    pub fn place_hot_corners(&self) {
        self.hot_corners.borrow_mut().clear();
        let settings = config::load();
        if !settings.quick_hover {
            return;
        }
        let side = if settings.quick_side == Side::Right { Edge::Right } else { Edge::Left };
        let spot = Spot {
            edges: [(Edge::Bottom, true), (side, true), (Edge::Top, false), (opposite(side), false)],
            width: HOT_CORNER_PIXELS,
            height: HOT_CORNER_PIXELS,
            margins: vec![(Edge::Bottom, -OVERHANG), (side, -OVERHANG)],
        };
        for monitor in hot_zone::monitors_for(&settings.quick_monitor) {
            let on_rest = Rc::clone(&self.panel);
            let on_leave = Rc::clone(&self.panel);
            let corner = HotZone::new(
                &self.app,
                &monitor,
                spot.clone(),
                move |monitor: &gdk::Monitor| on_rest.open_by_pointer(monitor),
                move || on_leave.close_soon_if_opened_by_pointer(),
            );
            self.hot_corners.borrow_mut().push(corner);
        }
    }

    pub fn is_open(&self) -> bool {
        self.panel.is_open()
    }

    pub fn close(&self) {
        self.panel.close();
    }

    pub fn toggle(&self) {
        self.panel.toggle();
    }

    /// A shell test it asked for finished on a worker.
    pub fn on_tested(&self, test: &str, passed: bool) {
        self.panel.on_tested(test, passed);
    }
}

fn opposite(side: Edge) -> Edge {
    if side == Edge::Right { Edge::Left } else { Edge::Right }
}
