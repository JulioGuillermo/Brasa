//! The bottom row: battery level, then round buttons for the island settings, Brasa's, Hyprland's and power.

use std::rc::Rc;

use gtk4::prelude::*;

use super::glyphs;
use crate::services::hardware;

const EXIT: &str = "~/.local/bin/brasa-exit";
const ISLAND_SETTINGS: &str = "~/.local/bin/brasa-island-settings";
const BRASA_SETTINGS: &str = "~/.local/bin/brasa-settings";
const HYPRMOD: &str = "hyprmod";

pub struct Footer {
    pub root: gtk4::Box,
    pub buttons: Vec<gtk4::Button>,
}

impl Footer {
    /// on_command(command): closes the panel and runs it.
    pub fn new(on_command: Rc<dyn Fn(&str)>) -> Footer {
        let root = gtk4::Box::builder().spacing(6).css_classes(["quick-footer"]).build();
        root.append(
            &gtk4::Label::builder()
                .label(glyphs::BATTERY)
                .visible(hardware::has_battery())
                .css_classes(["quick-battery"])
                .build(),
        );
        root.append(&gtk4::Label::builder().label(hardware::battery_percent()).xalign(0.0).hexpand(true).build());

        let mut buttons = vec![round_button(glyphs::SETTINGS, "Panel and island settings: size, corner…", &on_command, ISLAND_SETTINGS)];
        buttons.push(round_button(glyphs::KEYS_SETTINGS, "Your key bindings and the apps the keys open", &on_command, BRASA_SETTINGS));
        if hardware::has_hyprmod() && hardware::in_hyprland() {
            buttons.push(round_button(glyphs::HYPRLAND, "Hyprland settings", &on_command, HYPRMOD));
        }
        buttons.push(round_button(glyphs::POWER_OFF, "Log out, lock, power off…", &on_command, EXIT));
        for button in &buttons {
            root.append(button);
        }
        Footer { root, buttons }
    }
}

fn round_button(glyph: &str, tooltip: &str, on_command: &Rc<dyn Fn(&str)>, command: &'static str) -> gtk4::Button {
    let button = gtk4::Button::builder().label(glyph).tooltip_text(tooltip).css_classes(["quick-round"]).build();
    let on_command = Rc::clone(on_command);
    button.connect_clicked(move |_| on_command(command));
    button
}
