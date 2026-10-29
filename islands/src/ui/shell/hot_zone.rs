//! A few invisible pixels on a screen edge or corner of one monitor: resting the pointer there opens a
//! panel on that monitor. One per monitor lets a panel without a monitor of its own open on any of them.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const DWELL: Duration = Duration::from_millis(120);

/// Where the zone sits: the edges it is anchored to, its size, and its margins from them (negative
/// past the screen edge, where a fractional scale can leave a window's own border half drawn).
#[derive(Clone, Debug)]
pub struct Spot {
    pub edges: [(Edge, bool); 4],
    pub width: i32,
    pub height: i32,
    pub margins: Vec<(Edge, i32)>,
}

pub struct HotZone {
    window: gtk4::Window,
}

impl HotZone {
    /// on_rest: the pointer rested there. on_leave: it left again.
    pub fn new(
        app: &gtk4::Application,
        monitor: &gdk::Monitor,
        spot: Spot,
        on_rest: impl Fn(&gdk::Monitor) + 'static,
        on_leave: impl Fn() + 'static,
    ) -> HotZone {
        let window = gtk4::Window::builder()
            .application(app)
            .css_classes(["brasa-hot-zone"])
            .default_width(spot.width)
            .default_height(spot.height)
            .child(&sized_box(spot.width, spot.height))
            .build();
        window.init_layer_shell();
        window.set_namespace(Some("brasa-hot-zone"));
        window.set_layer(Layer::Overlay);
        window.set_exclusive_zone(-1);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_monitor(Some(monitor));
        for (edge, anchored) in spot.edges {
            window.set_anchor(edge, anchored);
        }
        for (edge, pixels) in spot.margins {
            window.set_margin(edge, pixels);
        }
        window.add_controller(pointer_controller(monitor.clone(), on_rest, on_leave));
        window.present();
        HotZone { window }
    }
}

impl Drop for HotZone {
    fn drop(&mut self) {
        self.window.destroy();
    }
}

/// Without a child of that size GTK makes the window 200 pixels a side.
fn sized_box(width: i32, height: i32) -> gtk4::Box {
    gtk4::Box::builder().width_request(width).height_request(height).build()
}

fn pointer_controller(
    monitor: gdk::Monitor,
    on_rest: impl Fn(&gdk::Monitor) + 'static,
    on_leave: impl Fn() + 'static,
) -> gtk4::EventControllerMotion {
    let waiting: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let on_rest = Rc::new(on_rest);
    let pointer = gtk4::EventControllerMotion::new();

    let for_enter = Rc::clone(&waiting);
    pointer.connect_enter(move |_, _, _| {
        let on_rest = Rc::clone(&on_rest);
        let monitor = monitor.clone();
        let waiting = Rc::clone(&for_enter);
        let source = glib::timeout_add_local_once(DWELL, move || {
            waiting.borrow_mut().take();
            on_rest(&monitor);
        });
        if let Some(old) = for_enter.borrow_mut().replace(source) {
            old.remove();
        }
    });
    pointer.connect_leave(move |_| {
        if let Some(source) = waiting.borrow_mut().take() {
            source.remove();
        }
        on_leave();
    });
    pointer
}

/// The monitors a panel gets zones on: its own, or every one when it has none.
pub fn monitors_for(connector: &str) -> Vec<gdk::Monitor> {
    if connector.is_empty() {
        return super::monitors::all();
    }
    super::monitors::find(connector).into_iter().collect()
}
