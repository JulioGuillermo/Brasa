//! The desktop entry an app named in its notification: its icon, and launching it.

use gio_unix::DesktopAppInfo;
use gtk4::gio;
use gtk4::gio::prelude::*;

fn app_named(desktop_entry: &str) -> Option<DesktopAppInfo> {
    let name = desktop_entry.strip_suffix(".desktop").unwrap_or(desktop_entry);
    DesktopAppInfo::new(&format!("{name}.desktop"))
}

pub fn icon_of(desktop_entry: &str) -> Option<gio::Icon> {
    app_named(desktop_entry)?.icon()
}

pub fn open_app(desktop_entry: &str) {
    if let Some(app) = app_named(desktop_entry) {
        let _ = app.launch(&[], None::<&gio::AppLaunchContext>);
    }
}
