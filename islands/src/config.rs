//! The islands' settings, from ~/.config/brasa/island.conf: KEY=value lines, # for comments.
//! The Island window (brasa-island-settings) writes it; this only reads it.

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Edge {
    Top,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Place {
    pub edge: Edge,
    pub align: Align,
    /// A connector such as DP-2; empty: the monitor that has the focus.
    pub monitor: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LauncherView {
    Grid,
    List,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub place: Place,
    pub hover: bool,
    pub media: bool,
    pub notification_centre: bool,
    pub peek_seconds: f64,
    pub color: String,
    pub launcher: bool,
    pub launcher_place: Place,
    pub launcher_view: LauncherView,
    pub launcher_size: Size,
    pub launcher_hover: bool,
    pub quick_side: Side,
    pub quick_size: Size,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            place: Place { edge: Edge::Top, align: Align::Center, monitor: String::new() },
            hover: true,
            media: true,
            notification_centre: true,
            peek_seconds: 2.5,
            color: "#0c1838".into(),
            launcher: true,
            launcher_place: Place { edge: Edge::Bottom, align: Align::Center, monitor: String::new() },
            launcher_view: LauncherView::Grid,
            launcher_size: Size::Medium,
            launcher_hover: true,
            quick_side: Side::Right,
            quick_size: Size::Small,
        }
    }
}

pub fn path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".config/brasa/island.conf")
}

pub fn load() -> Config {
    match std::fs::read_to_string(path()) {
        Ok(text) => parse(&text),
        Err(_) => Config::default(),
    }
}

pub fn parse(text: &str) -> Config {
    let pairs = read_pairs(text);
    let get = |key: &str| pairs.get(key).map(String::as_str).unwrap_or("");
    let default = Config::default();

    let hover = switched_on(get("hover"), default.hover);
    Config {
        place: place_from(&pairs, "", &default.place),
        hover,
        media: switched_on(get("media"), default.media),
        notification_centre: switched_on(get("notification_centre"), default.notification_centre),
        peek_seconds: get("peek_seconds").parse().unwrap_or(default.peek_seconds),
        color: if get("color").starts_with('#') { get("color").to_string() } else { default.color },
        launcher: switched_on(get("launcher"), default.launcher),
        launcher_place: place_from(&pairs, "launcher_", &default.launcher_place),
        launcher_view: match get("launcher_view") {
            "list" => LauncherView::List,
            "grid" => LauncherView::Grid,
            _ => default.launcher_view,
        },
        launcher_size: size_from(get("launcher_size"), default.launcher_size),
        launcher_hover: switched_on(get("launcher_hover"), hover),
        quick_side: match get("quick_side") {
            "left" => Side::Left,
            "right" => Side::Right,
            _ => default.quick_side,
        },
        quick_size: size_from(get("quick_size"), default.quick_size),
    }
}

fn read_pairs(text: &str) -> HashMap<String, String> {
    let mut pairs = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = without_comment(value).trim().trim_matches(|c| c == '"' || c == '\'');
        pairs.insert(key.trim().to_lowercase(), value.to_string());
    }
    pairs
}

/// "value   # note" -> "value": a comment needs whitespace before its #, so a colour stays whole.
fn without_comment(value: &str) -> &str {
    let mut previous_is_space = false;
    for (index, character) in value.char_indices() {
        if character == '#' && previous_is_space {
            return &value[..index];
        }
        previous_is_space = character.is_whitespace();
    }
    value
}

fn switched_on(value: &str, default: bool) -> bool {
    if value.is_empty() {
        return default;
    }
    !matches!(value.to_lowercase().as_str(), "off" | "no" | "false" | "0")
}

fn size_from(value: &str, default: Size) -> Size {
    match value {
        "small" => Size::Small,
        "medium" => Size::Medium,
        "large" => Size::Large,
        _ => default,
    }
}

fn place_from(pairs: &HashMap<String, String>, prefix: &str, default: &Place) -> Place {
    let get = |key: &str| pairs.get(&format!("{prefix}{key}")).map(String::as_str);
    Place {
        edge: match get("edge") {
            Some("top") => Edge::Top,
            Some("bottom") => Edge::Bottom,
            _ => default.edge,
        },
        align: match get("align") {
            Some("left") => Align::Left,
            Some("center") => Align::Center,
            Some("right") => Align::Right,
            _ => default.align,
        },
        monitor: get("monitor").map(str::to_string).unwrap_or_else(|| default.monitor.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_settings_window_file() {
        let config = parse(
            "# comment\nedge=bottom\nalign=left\nmonitor=DP-2\nhover=off\npeek_seconds=1.5\n\
             launcher_edge=top\nlauncher_view=list\nlauncher_size=large\ncolor=#112233\nquick_side=left\n",
        );
        assert_eq!(config.place.edge, Edge::Bottom);
        assert_eq!(config.place.align, Align::Left);
        assert_eq!(config.place.monitor, "DP-2");
        assert!(!config.hover);
        assert_eq!(config.peek_seconds, 1.5);
        assert_eq!(config.launcher_place.edge, Edge::Top);
        assert_eq!(config.launcher_view, LauncherView::List);
        assert_eq!(config.launcher_size, Size::Large);
        assert_eq!(config.color, "#112233");
        assert_eq!(config.quick_side, Side::Left);
    }

    #[test]
    fn launcher_hover_follows_hover_when_missing() {
        assert!(!parse("hover=off\n").launcher_hover);
        assert!(parse("hover=off\nlauncher_hover=on\n").launcher_hover);
    }

    #[test]
    fn a_comment_after_a_value_is_dropped_but_a_colour_is_kept() {
        let config = parse("hover=on   # off: only Super+m\ncolor=#0c1838\n");
        assert!(config.hover);
        assert_eq!(config.color, "#0c1838");
    }
}
