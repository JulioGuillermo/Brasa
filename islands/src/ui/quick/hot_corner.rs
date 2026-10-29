//! A few pixels in a bottom corner of one monitor: resting the pointer there opens the quick settings
//! on that monitor.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use gtk4_layer_shell::{Edge as LayerEdge, KeyboardMode, Layer, LayerShell};

const SIDE_PIXELS: i32 = 24;
const DWELL: Duration = Duration::from_millis(150);

pub struct HotCorner {
    window: gtk4::Window,
}

impl HotCorner {
    pub fn new(
        app: &gtk4::Application,
        monitor: &gdk::Monitor,
        on_right: bool,
        on_rest: impl Fn(&gdk::Monitor) + 'static,
    ) -> HotCorner {
        let window = gtk4::Window::builder()
            .application(app)
            .css_classes(["brasa-quick-corner"])
            .default_width(SIDE_PIXELS)
            .default_height(SIDE_PIXELS)
            .child(&gtk4::Box::builder().width_request(SIDE_PIXELS).height_request(SIDE_PIXELS).build())
            .build();
        window.init_layer_shell();
        window.set_namespace(Some("brasa-quick-corner"));
        window.set_layer(Layer::Overlay);
        window.set_exclusive_zone(-1);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_monitor(Some(monitor));
        window.set_anchor(LayerEdge::Bottom, true);
        window.set_anchor(LayerEdge::Right, on_right);
        window.set_anchor(LayerEdge::Left, !on_right);

        let waiting: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        let pointer = gtk4::EventControllerMotion::new();
        let on_rest = Rc::new(on_rest);
        let monitor = monitor.clone();
        let for_enter = Rc::clone(&waiting);
        pointer.connect_enter(move |_, _, _| {
            let on_rest = Rc::clone(&on_rest);
            let monitor = monitor.clone();
            let done = Rc::clone(&for_enter);
            let source = glib::timeout_add_local_once(DWELL, move || {
                done.borrow_mut().take();
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
        });
        window.add_controller(pointer);
        window.present();
        HotCorner { window }
    }
}

impl Drop for HotCorner {
    fn drop(&mut self) {
        self.window.destroy();
    }
}
