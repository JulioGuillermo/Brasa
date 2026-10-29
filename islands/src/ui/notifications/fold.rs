//! "Closed by their app": the notices their apps took back, folded away under a button.

use std::rc::Rc;

use gtk4::prelude::*;

const SLIDE_MS: u32 = 200;

pub struct Fold {
    button: gtk4::Button,
    label: gtk4::Label,
    arrow: gtk4::Image,
    revealer: gtk4::Revealer,
    list: gtk4::Box,
}

impl Fold {
    /// on_change: the fold opened, or finished sliding.
    pub fn new(on_change: impl Fn() + 'static) -> Rc<Fold> {
        let on_change = Rc::new(on_change);
        let label = gtk4::Label::builder().xalign(0.0).hexpand(true).build();
        let arrow = gtk4::Image::from_icon_name("pan-end-symbolic");
        let face = gtk4::Box::builder().spacing(6).build();
        face.append(&label);
        face.append(&arrow);
        let button = gtk4::Button::builder().child(&face).css_classes(["island-fold"]).build();
        let list = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).build();
        let revealer = gtk4::Revealer::builder().child(&list).transition_duration(SLIDE_MS).build();

        let fold = Rc::new(Fold { button, label, arrow, revealer, list });
        let toggled = Rc::downgrade(&fold);
        let after_toggle = Rc::clone(&on_change);
        fold.button.connect_clicked(move |_| {
            if let Some(fold) = toggled.upgrade() {
                fold.toggle();
                after_toggle();
            }
        });
        fold.revealer.connect_child_revealed_notify(move |_| on_change());
        fold
    }

    pub fn button(&self) -> &gtk4::Button {
        &self.button
    }

    pub fn revealer(&self) -> &gtk4::Revealer {
        &self.revealer
    }

    pub fn list(&self) -> &gtk4::Box {
        &self.list
    }

    pub fn show_count(&self, count: usize) {
        self.label.set_text(&format!("Closed by their app · {count}"));
    }

    fn toggle(&self) {
        let unfolded = !self.revealer.reveals_child();
        self.revealer.set_reveal_child(unfolded);
        self.arrow.set_icon_name(Some(if unfolded { "pan-down-symbolic" } else { "pan-end-symbolic" }));
    }
}
