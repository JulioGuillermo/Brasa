//! Every tile of the panel, in order, for this machine.

use super::items::{self, Choice, Item};
use super::{custom, glyphs};
use crate::services::hardware;

const BIN: &str = "~/.local/bin/";
const HYPRLAND_LAYOUTS: [(&str, &str); 6] = [
    ("dwindle", "Dwindle · split the focused window"),
    ("master", "Master · one big + a column"),
    ("scrolling", "Scroll → · half-screen columns"),
    ("scrolling-down", "Scroll ↓ · half-screen rows"),
    ("focus", "Focus → · the focused column big"),
    ("focus-down", "Focus ↓ · the focused row big"),
];
const SWAY_LAYOUTS: [(&str, &str); 4] = [
    ("splith", "Side by side"),
    ("splitv", "One above the other"),
    ("tabbed", "Tabs"),
    ("stacking", "Stacked"),
];
const POWER_PROFILES: [(&str, &str); 3] =
    [("power-saver", "Power saver"), ("balanced", "Balanced"), ("performance", "Performance")];
const CHARGE_LIMITS: [u32; 4] = [60, 80, 90, 100];

fn bin(program: &str) -> String {
    format!("{BIN}{program}")
}

pub fn tiles() -> Vec<Item> {
    let mut tiles = radios();
    tiles.push(items::switch(
        glyphs::MUTE,
        "Mute",
        "wpctl set-mute @DEFAULT_AUDIO_SINK@ 1",
        "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0",
        "wpctl get-volume @DEFAULT_AUDIO_SINK@ | grep -q MUTED",
    ));
    let fx = bin("brasa-fx all");
    tiles.push(items::switch(glyphs::EFFECTS, "Effects", &fx, &fx, "! test -e \"$XDG_RUNTIME_DIR/brasa-fx/blur-off\""));
    tiles.push(items::action(glyphs::NETWORKS, "Networks", "nm-connection-editor"));
    tiles.push(layout_menu());
    tiles.extend(system_menus());
    tiles.push(items::action(glyphs::KEYBOARD, "Keyboard", &bin("brasa-keyboard")));
    tiles.push(items::action(glyphs::SESSIONS, "Sessions", &bin("brasa-sessions")));
    tiles.push(items::action(glyphs::ISLAND, "Island", &bin("brasa-island-settings")));
    tiles.push(items::action(glyphs::KEYS, "Keys", &bin("brasa-keys")));
    tiles.push(items::action(glyphs::LOCK, "Lock", &bin("brasa-lock")));
    tiles.extend(custom::tiles());
    tiles
}

fn radios() -> Vec<Item> {
    let mut tiles = vec![items::switch(
        glyphs::WIFI,
        "Wi-Fi",
        "nmcli radio wifi on",
        "nmcli radio wifi off",
        "nmcli radio wifi | grep -qx enabled",
    )];
    if hardware::has_bluetooth() {
        tiles.push(items::switch(
            glyphs::BLUETOOTH,
            "Bluetooth",
            "bluetoothctl power on",
            "bluetoothctl power off",
            "bluetoothctl show | grep -q 'Powered: yes'",
        ));
    }
    let radio = bin("brasa-radio");
    tiles.push(items::switch(
        glyphs::AIRPLANE,
        "Airplane",
        &radio,
        &radio,
        "rfkill list | grep -q 'Soft blocked: yes' && ! rfkill list | grep -q 'Soft blocked: no'",
    ));
    tiles
}

fn layout_menu() -> Item {
    let layouts: &[(&str, &str)] = if hardware::in_hyprland() { &HYPRLAND_LAYOUTS } else { &SWAY_LAYOUTS };
    let choices = layouts
        .iter()
        .map(|(layout, label)| {
            items::choice(
                label,
                &format!("{BIN}brasa-wm layout-set {layout}"),
                &format!("{BIN}brasa-wm layout-current | grep -qx {layout}"),
            )
        })
        .collect();
    items::menu(glyphs::LAYOUT, "Layout", choices)
}

fn system_menus() -> Vec<Item> {
    let mut menus = Vec::new();
    if hardware::has_power_profiles() {
        menus.push(items::menu(glyphs::POWER_PROFILE, "Power", power_profile_choices()));
    }
    if hardware::has_charge_limit() {
        menus.push(items::menu(glyphs::CHARGE_LIMIT, "Charge limit", charge_limit_choices()));
    }
    let levels = hardware::keyboard_light_levels();
    if !levels.is_empty() {
        menus.push(items::menu(glyphs::KEY_LIGHT, "Key light", key_light_choices(&levels)));
    }
    menus.push(items::menu(glyphs::CLEAN, "Clean", clean_choices()));
    menus
}

fn power_profile_choices() -> Vec<Choice> {
    POWER_PROFILES
        .iter()
        .map(|(profile, label)| {
            items::choice(label, &format!("{BIN}brasa-power {profile}"), &format!("powerprofilesctl get | grep -qx {profile}"))
        })
        .collect()
}

fn charge_limit_choices() -> Vec<Choice> {
    CHARGE_LIMITS
        .iter()
        .map(|limit| {
            items::choice(
                &format!("Charge up to {limit} %"),
                &format!("{BIN}brasa-battery {limit}"),
                &format!("grep -qx {limit} /sys/class/power_supply/BAT*/charge_control_end_threshold"),
            )
        })
        .collect()
}

fn key_light_choices(levels: &[(u32, String)]) -> Vec<Choice> {
    levels
        .iter()
        .map(|(value, name)| {
            items::choice(name, &format!("{BIN}brasa-kbd set {value}"), &format!("{BIN}brasa-kbd get | grep -qx {value}"))
        })
        .collect()
}

fn clean_choices() -> Vec<Choice> {
    vec![
        items::choice("Clean /tmp…", &format!("{BIN}brasa-clean tmp"), "false"),
        items::choice("Free RAM (page cache)", &format!("{BIN}brasa-clean ram"), "false"),
    ]
}
