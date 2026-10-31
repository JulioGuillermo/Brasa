//! While a panel the user opened is showing, an invisible sheet covers every monitor under it: a click
//! anywhere off the panel lands on the sheet and closes the panel. That click does not reach the window
//! underneath.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use super::monitors;

pub struct ClickAway {
    app: gtk4::Application,
    on_click: Rc<dyn Fn()>,
    sheets: RefCell<Vec<gtk4::Window>>,
}

impl ClickAway {
    pub fn new(app: &gtk4::Application, on_click: impl Fn() + 'static) -> ClickAway {
        ClickAway { app: app.clone(), on_click: Rc::new(on_click), sheets: RefCell::new(Vec::new()) }
    }

    pub fn show(&self) {
        if !self.sheets.borrow().is_empty() {
            return;
        }
        let sheets = monitors::all().iter().map(|monitor| self.sheet_on(monitor)).collect();
        *self.sheets.borrow_mut() = sheets;
    }

    pub fn hide(&self) {
        for sheet in self.sheets.borrow_mut().drain(..) {
            sheet.destroy();
        }
    }

    /// On the Top layer, so the panels, on Overlay, stay above it.
    fn sheet_on(&self, monitor: &gtk4::gdk::Monitor) -> gtk4::Window {
        let sheet = gtk4::Window::builder().application(&self.app).css_classes(["brasa-click-away"]).build();
        sheet.init_layer_shell();
        sheet.set_namespace(Some("brasa-click-away"));
        sheet.set_layer(Layer::Top);
        sheet.set_exclusive_zone(-1);
        sheet.set_keyboard_mode(KeyboardMode::None);
        sheet.set_monitor(Some(monitor));
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            sheet.set_anchor(edge, true);
        }
        let click = gtk4::GestureClick::new();
        click.set_button(0);
        let on_click = Rc::clone(&self.on_click);
        click.connect_pressed(move |_, _, _, _| {
            let on_click = Rc::clone(&on_click);
            glib::idle_add_local_once(move || on_click());
        });
        sheet.add_controller(click);
        sheet.present();
        sheet
    }
}

impl Drop for ClickAway {
    fn drop(&mut self) {
        self.hide();
    }
}
