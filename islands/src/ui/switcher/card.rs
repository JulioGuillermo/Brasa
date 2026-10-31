//! A window's card: its app's icon, its title, and the workspace it is on with the app's name. A
//! title that only repeats the app's name (a terminal's "foot") gives way to what runs in it.

use gio_unix::DesktopAppInfo;
use gtk4::prelude::*;
use gtk4::{gdk, gio, pango};

use super::carousel::{CARD_HEIGHT, CARD_WIDTH};
use crate::services::compositor::Window;
use crate::services::terminal;

const ICON_PIXELS: i32 = 56;
const TITLE_CHARACTERS: i32 = 24;
const FALLBACK_ICON: &str = "application-x-executable";

pub fn card(window: &Window) -> gtk4::Widget {
    let app = desktop_entry(&window.app_id);
    let app_name = app.as_ref().map(|app| app.display_name().to_string()).unwrap_or_else(|| window.app_id.clone());

    let icon = icon(app.as_ref(), &window.app_id);
    let title = line(&shown_title(window, &app_name), "switcher-title");
    let about = line(&format!("{}  ·  {app_name}", window.workspace), "switcher-about");

    let card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .css_classes(["switcher-card"])
        .width_request(CARD_WIDTH)
        .height_request(CARD_HEIGHT)
        .build();
    icon.set_margin_top(14);
    card.append(&icon);
    card.append(&title);
    card.append(&about);
    card.upcast()
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
    image.set_pixel_size(ICON_PIXELS);
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
