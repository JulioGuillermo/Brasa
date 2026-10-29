use gtk4::prelude::*;

/// A round icon button of the islands (play, next, close…).
pub fn round_button(icon: &str, css: &[&str], on_click: impl Fn() + 'static) -> gtk4::Button {
    let mut classes = vec!["island-button"];
    classes.extend_from_slice(css);
    let button = gtk4::Button::builder().icon_name(icon).css_classes(classes).valign(gtk4::Align::Center).build();
    button.connect_clicked(move |_| on_click());
    button
}
