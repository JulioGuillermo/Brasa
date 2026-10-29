//! The picture of a notice: what the app sent as pixels, a file, an icon name, its desktop entry's icon, or a fallback.

use gtk4::{gdk, glib};

use super::desktop;
use crate::services::notifications::notice::Notice;

const FALLBACK_ICON: &str = "dialog-information-symbolic";

pub fn show_icon(image: &gtk4::Image, notice: &Notice) {
    if let Some(picture) = &notice.image_data {
        image.set_paintable(Some(&picture.to_texture()));
        return;
    }
    let from_source = [&notice.image_path, &notice.app_icon].into_iter().any(|source| show_source(image, source));
    if from_source || (!notice.desktop_entry.is_empty() && show_desktop_icon(image, &notice.desktop_entry)) {
        return;
    }
    image.set_icon_name(Some(FALLBACK_ICON));
}

/// source: a file, a file:// address or an icon name.
fn show_source(image: &gtk4::Image, source: &str) -> bool {
    let path = if source.starts_with("file://") {
        glib::filename_from_uri(source).map(|(path, _host)| path).ok()
    } else {
        Some(source.into())
    };
    let Some(path) = path else { return false };
    if path.starts_with("/") {
        return match gdk::Texture::from_filename(&path) {
            Ok(texture) => {
                image.set_paintable(Some(&texture));
                true
            }
            Err(_) => false,
        };
    }
    if source.is_empty() || !has_icon(source) {
        return false;
    }
    image.set_icon_name(Some(source));
    true
}

fn has_icon(name: &str) -> bool {
    let Some(display) = gdk::Display::default() else { return false };
    gtk4::IconTheme::for_display(&display).has_icon(name)
}

fn show_desktop_icon(image: &gtk4::Image, desktop_entry: &str) -> bool {
    let Some(icon) = desktop::icon_of(desktop_entry) else { return false };
    image.set_from_gicon(&icon);
    true
}
