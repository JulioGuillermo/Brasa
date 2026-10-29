//! What this machine has (bluetooth, a backlight, a battery…), so the panel leaves out what it cannot
//! use, and the few values it reads and writes for the sliders and the footer.

use std::fs;
use std::path::{Path, PathBuf};

use super::process;

const SYS: &str = "/sys/class";

/// The entries of a /sys/class directory whose name passes `wanted`, sorted.
fn devices(class: &str, wanted: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(Path::new(SYS).join(class)) else { return Vec::new() };
    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .filter(|entry| wanted(&entry.file_name().to_string_lossy()))
        .map(|entry| entry.path())
        .collect();
    found.sort();
    found
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).map(|text| text.trim().to_string()).unwrap_or_default()
}

fn read_number(path: &Path) -> Option<u32> {
    read(path).parse().ok()
}

fn batteries() -> Vec<PathBuf> {
    devices("power_supply", |name| name.starts_with("BAT"))
}

fn first_backlight() -> Option<PathBuf> {
    devices("backlight", |_| true).into_iter().next()
}

pub fn has_bluetooth() -> bool {
    !devices("bluetooth", |_| true).is_empty()
}

pub fn has_backlight() -> bool {
    first_backlight().is_some()
}

pub fn has_battery() -> bool {
    !batteries().is_empty()
}

pub fn has_charge_limit() -> bool {
    batteries().iter().any(|battery| battery.join("charge_control_end_threshold").exists())
}

pub fn has_power_profiles() -> bool {
    on_path("powerprofilesctl")
}

pub fn has_hyprmod() -> bool {
    on_path("hyprmod")
}

pub fn in_hyprland() -> bool {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok_and(|signature| !signature.is_empty())
}

fn on_path(program: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else { return false };
    std::env::split_paths(&path).any(|directory| directory.join(program).is_file())
}

/// The keyboard backlight's levels as (value, name); none without one.
pub fn keyboard_light_levels() -> Vec<(u32, String)> {
    let Some(led) = devices("leds", |name| name.contains("kbd_backlight")).into_iter().next() else {
        return Vec::new();
    };
    key_light_levels_up_to(read_number(&led.join("max_brightness")).unwrap_or(0))
}

fn key_light_levels_up_to(top: u32) -> Vec<(u32, String)> {
    let names: &[&str] = match top {
        0 => return Vec::new(),
        1 => &["On"],
        2 => &["Low", "High"],
        3 => &["Low", "Medium", "High"],
        4 => &["Low", "Medium", "High", "Max"],
        _ => return percent_levels(top),
    };
    let mut levels = vec![(0, "Off".to_string())];
    levels.extend(names.iter().enumerate().map(|(index, name)| (index as u32 + 1, name.to_string())));
    levels
}

fn percent_levels(top: u32) -> Vec<(u32, String)> {
    [0, 25, 50, 75, 100]
        .into_iter()
        .map(|percent| {
            let name = if percent == 0 { "Off".to_string() } else { format!("{percent} %") };
            ((top as f64 * percent as f64 / 100.0).round() as u32, name)
        })
        .collect()
}

/// "73 %", or empty without a battery.
pub fn battery_percent() -> String {
    let Some(battery) = batteries().into_iter().next() else { return String::new() };
    let level = read(&battery.join("capacity"));
    if level.is_empty() { level } else { format!("{level} %") }
}

/// Of the first backlight, the one brightnessctl changes: 0.0 to 1.0.
pub fn screen_brightness() -> f64 {
    let Some(backlight) = first_backlight() else { return 0.0 };
    let current = read_number(&backlight.join("brightness"));
    let top = read_number(&backlight.join("max_brightness"));
    match (current, top) {
        (Some(current), Some(top)) if top > 0 => current as f64 / top as f64,
        _ => 0.0,
    }
}

pub fn set_screen_brightness(level: f64) {
    let percent = ((level * 100.0).round() as i64).max(1);
    process::run("brightnessctl", &["-q", "set", &format!("{percent}%")]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_ranges_get_named_levels() {
        let names: Vec<String> = key_light_levels_up_to(2).into_iter().map(|(_, name)| name).collect();
        assert_eq!(names, ["Off", "Low", "High"]);
        assert_eq!(key_light_levels_up_to(1), [(0, "Off".to_string()), (1, "On".to_string())]);
    }

    #[test]
    fn wide_ranges_get_percentages_of_the_top() {
        let levels = key_light_levels_up_to(255);
        assert_eq!(levels.len(), 5);
        assert_eq!(levels[2], (128, "50 %".to_string()));
        assert_eq!(levels[4], (255, "100 %".to_string()));
    }

    #[test]
    fn no_range_no_levels() {
        assert!(key_light_levels_up_to(0).is_empty());
    }
}
