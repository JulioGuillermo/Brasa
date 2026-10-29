//! The centre's header: the title with the count, Do Not Disturb, Clear all and the button that closes the island.

use gtk4::prelude::*;

use crate::ui::components::round_button::round_button;

pub struct Header {
    widget: gtk4::Box,
    title: gtk4::Label,
    dnd_button: gtk4::Button,
    close_button: gtk4::Button,
}

impl Header {
    pub fn new(on_dnd: impl Fn() + 'static, on_clear_all: impl Fn() + 'static, on_close_island: impl Fn() + 'static) -> Header {
        let title = gtk4::Label::builder().xalign(0.0).hexpand(true).css_classes(["island-title"]).build();
        let dnd_button = header_button("notifications-disabled-symbolic", "Do not disturb", on_dnd);
        let clear_button = header_button("edit-clear-all-symbolic", "Clear all", on_clear_all);
        let close_button = header_button("go-up-symbolic", "Close the island", on_close_island);
        let widget = gtk4::Box::builder().spacing(4).css_classes(["island-notices-header"]).build();
        widget.append(&title);
        widget.append(&dnd_button);
        widget.append(&clear_button);
        widget.append(&close_button);
        Header { widget, title, dnd_button, close_button }
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.widget
    }

    pub fn show_count(&self, live: usize) {
        if live > 0 {
            self.title.set_text(&format!("Notifications · {live}"));
        } else {
            self.title.set_text("Notifications");
        }
    }

    pub fn show_dnd(&self, on: bool) {
        if on {
            self.dnd_button.add_css_class("lit");
        } else {
            self.dnd_button.remove_css_class("lit");
        }
    }

    /// The close button points to the edge the island hangs from.
    pub fn show_edge(&self, at_top: bool) {
        self.close_button.set_icon_name(if at_top { "go-up-symbolic" } else { "go-down-symbolic" });
    }
}

fn header_button(icon: &str, tooltip: &str, on_click: impl Fn() + 'static) -> gtk4::Button {
    let button = round_button(icon, &["island-header-button"], on_click);
    button.set_tooltip_text(Some(tooltip));
    button
}
