//! An app's icon at a given size.

use gio_unix::DesktopAppInfo;
use gtk4::prelude::*;

const FALLBACK_ICON: &str = "application-x-executable";

pub fn app_icon(app: &DesktopAppInfo, pixel_size: i32) -> gtk4::Image {
    let image = match app.icon() {
        Some(icon) => gtk4::Image::from_gicon(&icon),
        None => gtk4::Image::from_icon_name(FALLBACK_ICON),
    };
    image.set_pixel_size(pixel_size);
    image
}
