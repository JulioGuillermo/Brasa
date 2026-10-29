//! The one-line ellipsized label every view of the music uses.

use gtk4::pango;

pub fn label(css: &[&str], xalign: f32, max_chars: i32) -> gtk4::Label {
    let label = gtk4::Label::builder().xalign(xalign).css_classes(css.to_vec()).ellipsize(pango::EllipsizeMode::End).build();
    if max_chars > 0 {
        label.set_max_width_chars(max_chars);
    }
    label
}
