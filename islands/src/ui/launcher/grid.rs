//! One app as a tile of the grid: icon over name.

use gio_unix::DesktopAppInfo;
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;

use super::icon::app_icon;
use super::metrics::Metrics;

const NAME_WIDTH_CHARS: i32 = 10;

pub fn grid_tile(app: &DesktopAppInfo, metrics: &Metrics) -> gtk4::Widget {
    let name = gtk4::Label::builder()
        .label(app.display_name())
        .ellipsize(EllipsizeMode::End)
        .max_width_chars(NAME_WIDTH_CHARS)
        .justify(gtk4::Justification::Center)
        .css_classes(["island-app-name"])
        .build();
    let tile = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(6).css_classes(["island-app"]).build();
    tile.append(&app_icon(app, metrics.grid_icon));
    tile.append(&name);
    tile.upcast()
}
