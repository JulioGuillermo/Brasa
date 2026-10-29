//! One app as a row of the list: icon, then name over what it is.

use gio_unix::DesktopAppInfo;
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;

use super::icon::app_icon;
use super::metrics::Metrics;

pub fn list_row(app: &DesktopAppInfo, metrics: &Metrics) -> gtk4::Widget {
    let name = gtk4::Label::builder()
        .label(app.display_name())
        .xalign(0.0)
        .ellipsize(EllipsizeMode::End)
        .css_classes(["island-app-name"])
        .build();
    let about = gtk4::Label::builder()
        .label(about_text(app))
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(EllipsizeMode::End)
        .css_classes(["island-app-about"])
        .build();
    let texts = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).valign(gtk4::Align::Center).hexpand(true).build();
    texts.append(&name);
    texts.append(&about);
    let row = gtk4::Box::builder().spacing(12).css_classes(["island-app", "island-app-row"]).build();
    row.append(&app_icon(app, metrics.list_icon));
    row.append(&texts);
    row.upcast()
}

fn about_text(app: &DesktopAppInfo) -> String {
    match app.description() {
        Some(description) if !description.is_empty() => description.to_string(),
        _ => app.generic_name().map(|name| name.to_string()).unwrap_or_default(),
    }
}
