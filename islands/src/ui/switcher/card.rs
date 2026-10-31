//! A window's card: its app's icon at the top left, its title and the workspace it is on with the
//! app's name at the bottom left. A title that only repeats the app's name (a terminal's "foot")
//! gives way to what runs in it. Once a picture of the window comes, it shows behind, and the icon
//! shrinks.

use gio_unix::DesktopAppInfo;
use gtk4::prelude::*;
use gtk4::{gdk, gio, pango};

use super::carousel::{CARD_HEIGHT, CARD_WIDTH};
use super::shot::Shot;
use crate::services::compositor::Window;
use crate::services::terminal;

const MARGIN: i32 = 14;
const TITLE_CHARACTERS: i32 = 24;
const FALLBACK_ICON: &str = "application-x-executable";

pub struct Card {
    pub root: gtk4::Widget,
    pub capture_id: String,
    shot: Shot,
}

impl Card {
    pub fn new(window: &Window) -> Card {
        let app = desktop_entry(&window.app_id);
        let app_name = app.as_ref().map(|app| app.display_name().to_string()).unwrap_or_else(|| window.app_id.clone());

        let texts = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .margin_start(MARGIN)
            .margin_end(MARGIN)
            .margin_top(MARGIN)
            .margin_bottom(MARGIN)
            .build();
        let icon = icon(app.as_ref(), &window.app_id);
        icon.set_halign(gtk4::Align::Start);
        icon.set_valign(gtk4::Align::Start);
        icon.set_vexpand(true);
        texts.append(&icon);
        texts.append(&line(&shown_title(window, &app_name), "switcher-title"));
        texts.append(&line(&format!("{}  ·  {app_name}", window.workspace), "switcher-about"));

        let shot = Shot::default();
        let root = gtk4::Overlay::builder()
            .css_classes(["switcher-card"])
            .overflow(gtk4::Overflow::Hidden)
            .width_request(CARD_WIDTH)
            .height_request(CARD_HEIGHT)
            .child(&shot)
            .build();
        root.add_overlay(&texts);
        Card { root: root.upcast(), capture_id: window.capture_id.clone(), shot }
    }

    pub fn show_shot(&self, picture: &gdk::Texture) {
        self.shot.show(picture);
        self.root.add_css_class("with-shot");
    }
}

fn shown_title(window: &Window, app_name: &str) -> String {
    if !only_names_the_app(&window.title, &window.app_id, app_name) {
        return window.title.clone();
    }
    let running = window.pid.and_then(terminal::running_in);
    running.unwrap_or_else(|| app_name.to_string())
}

fn only_names_the_app(title: &str, app_id: &str, app_name: &str) -> bool {
    let title = title.trim().to_lowercase();
    title.is_empty() || title == app_id.to_lowercase() || title == app_name.to_lowercase()
}

fn line(text: &str, class: &str) -> gtk4::Label {
    gtk4::Label::builder()
        .label(text)
        .css_classes([class])
        .xalign(0.0)
        .ellipsize(pango::EllipsizeMode::End)
        .max_width_chars(TITLE_CHARACTERS)
        .single_line_mode(true)
        .build()
}

/// The app's own .desktop file when one is named after it, or else the one that says it opens windows
/// of that class, or whose name ends in it (org.gnome.Nautilus for nautilus).
fn desktop_entry(app_id: &str) -> Option<DesktopAppInfo> {
    if app_id.is_empty() {
        return None;
    }
    for name in [app_id.to_string(), app_id.to_lowercase()] {
        if let Some(app) = DesktopAppInfo::new(&format!("{name}.desktop")) {
            return Some(app);
        }
    }
    let wanted = app_id.to_lowercase();
    let installed = gio::AppInfo::all().into_iter().filter_map(|info| info.downcast::<DesktopAppInfo>().ok());
    installed.into_iter().find(|app| opens_windows_of(app, &wanted))
}

fn opens_windows_of(app: &DesktopAppInfo, lowercase_app_id: &str) -> bool {
    let window_class = app.startup_wm_class().map(|class| class.to_lowercase());
    if window_class.as_deref() == Some(lowercase_app_id) {
        return true;
    }
    let id = app.id().map(|id| id.to_lowercase()).unwrap_or_default();
    let name = id.strip_suffix(".desktop").unwrap_or(&id);
    name.rsplit('.').next() == Some(lowercase_app_id)
}

fn icon(app: Option<&DesktopAppInfo>, app_id: &str) -> gtk4::Image {
    let image = match app.and_then(|app| app.icon()) {
        Some(icon) => gtk4::Image::from_gicon(&icon),
        None => gtk4::Image::from_gicon(&gio::ThemedIcon::new(&themed_icon_name(app_id))),
    };
    image.add_css_class("switcher-icon");
    image
}

fn themed_icon_name(app_id: &str) -> String {
    let lowercase = app_id.to_lowercase();
    let theme = gdk::Display::default().map(|display| gtk4::IconTheme::for_display(&display));
    let has_it = theme.is_some_and(|theme| !lowercase.is_empty() && theme.has_icon(&lowercase));
    if has_it { lowercase } else { FALLBACK_ICON.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_that_repeats_the_app_says_nothing() {
        assert!(only_names_the_app("foot", "foot", "Foot"));
        assert!(only_names_the_app("Foot", "foot", "Foot"));
        assert!(only_names_the_app("", "zen", "Zen Browser"));
        assert!(!only_names_the_app("~/brasa", "foot", "Foot"));
    }
}
