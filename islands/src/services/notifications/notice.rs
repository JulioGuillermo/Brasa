//! One notification as an app sent it (org.freedesktop.Notifications.Notify), read from the D-Bus message.

use std::sync::Arc;

use gtk4::glib;

use super::picture::{IMAGE_DATA_TYPE, Picture};

pub const LOW: u8 = 0;
pub const NORMAL: u8 = 1;
pub const CRITICAL: u8 = 2;
pub const DEFAULT_ACTION: &str = "default";

const SYNCHRONOUS_HINTS: [&str; 3] = ["synchronous", "private-synchronous", "x-canonical-private-synchronous"];
const IMAGE_DATA_HINTS: [&str; 3] = ["image-data", "image_data", "icon_data"];
const IMAGE_PATH_HINTS: [&str; 2] = ["image-path", "image_path"];

#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    pub key: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub id: u32,
    pub app: String,
    pub summary: String,
    pub body: String,
    pub app_icon: String,
    pub image_path: String,
    pub image_data: Option<Arc<Picture>>,
    pub desktop_entry: String,
    pub buttons: Vec<Button>,
    pub urgency: u8,
    pub expire_ms: i32,
    pub synchronous: String,
    pub replaces_id: u32,
    pub has_default: bool,
    pub resident: bool,
    pub transient: bool,
}

impl Default for Notice {
    fn default() -> Notice {
        Notice {
            id: 0,
            app: String::new(),
            summary: String::new(),
            body: String::new(),
            app_icon: String::new(),
            image_path: String::new(),
            image_data: None,
            desktop_entry: String::new(),
            buttons: Vec::new(),
            urgency: NORMAL,
            expire_ms: -1,
            synchronous: String::new(),
            replaces_id: 0,
            has_default: false,
            resident: false,
            transient: false,
        }
    }
}

impl Notice {
    pub fn is_critical(&self) -> bool {
        self.urgency == CRITICAL
    }

    /// arguments: the Notify call's (susssasa{sv}i). The id is given later.
    pub fn from_notify(arguments: &glib::Variant) -> Notice {
        let text = |position: usize| arguments.child_value(position).str().unwrap_or_default().to_string();
        let hints = glib::VariantDict::new(Some(&arguments.child_value(6)));
        let actions = arguments.child_value(5).get::<Vec<String>>().unwrap_or_default();
        Notice {
            id: 0,
            app: text(0),
            replaces_id: arguments.child_value(1).get::<u32>().unwrap_or(0),
            app_icon: text(2),
            summary: text(3),
            body: text(4),
            buttons: buttons_of(&actions),
            has_default: actions.iter().step_by(2).any(|key| key == DEFAULT_ACTION),
            image_data: picture_hint(&hints),
            image_path: string_hint(&hints, &IMAGE_PATH_HINTS),
            desktop_entry: string_hint(&hints, &["desktop-entry"]),
            urgency: hint::<u8>(&hints, &["urgency"], "y").unwrap_or(NORMAL),
            synchronous: string_hint(&hints, &SYNCHRONOUS_HINTS),
            resident: hint::<bool>(&hints, &["resident"], "b").unwrap_or(false),
            transient: hint::<bool>(&hints, &["transient"], "b").unwrap_or(false),
            expire_ms: arguments.child_value(7).get::<i32>().unwrap_or(-1),
        }
    }
}

fn buttons_of(actions: &[String]) -> Vec<Button> {
    actions
        .chunks_exact(2)
        .filter(|pair| pair[0] != DEFAULT_ACTION)
        .map(|pair| Button { key: pair[0].clone(), label: pair[1].clone() })
        .collect()
}

/// The first of the names the app used, if its value has the type.
fn hint<T: glib::variant::FromVariant>(hints: &glib::VariantDict, names: &[&str], kind: &str) -> Option<T> {
    let kind = glib::VariantTy::new(kind).ok()?;
    names.iter().find_map(|name| hints.lookup_value(name, Some(kind))).and_then(|value| value.get::<T>())
}

fn string_hint(hints: &glib::VariantDict, names: &[&str]) -> String {
    hint::<String>(hints, names, "s").unwrap_or_default()
}

fn picture_hint(hints: &glib::VariantDict) -> Option<Arc<Picture>> {
    let kind = glib::VariantTy::new(IMAGE_DATA_TYPE).ok()?;
    let value = IMAGE_DATA_HINTS.iter().find_map(|name| hints.lookup_value(name, Some(kind)))?;
    Picture::from_hint(&value).map(Arc::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::prelude::*;

    fn notify(actions: &[&str], hints: &[(&str, glib::Variant)], expire_ms: i32) -> glib::Variant {
        let dict = glib::VariantDict::new(None);
        for (name, value) in hints {
            dict.insert_value(name, value);
        }
        let actions: Vec<String> = actions.iter().map(|action| action.to_string()).collect();
        glib::Variant::tuple_from_iter([
            "Files".to_variant(),
            7u32.to_variant(),
            "folder".to_variant(),
            "Done".to_variant(),
            "Copied <b>3</b> files".to_variant(),
            actions.to_variant(),
            dict.end(),
            expire_ms.to_variant(),
        ])
    }

    #[test]
    fn reads_the_plain_arguments() {
        let notice = Notice::from_notify(&notify(&[], &[], 5000));
        assert_eq!(notice.app, "Files");
        assert_eq!(notice.replaces_id, 7);
        assert_eq!(notice.app_icon, "folder");
        assert_eq!(notice.summary, "Done");
        assert_eq!(notice.body, "Copied <b>3</b> files");
        assert_eq!(notice.expire_ms, 5000);
        assert_eq!(notice.urgency, NORMAL);
    }

    #[test]
    fn the_default_action_is_not_a_button() {
        let notice = Notice::from_notify(&notify(&["default", "Open", "reply", "Reply"], &[], -1));
        assert!(notice.has_default);
        assert_eq!(notice.buttons, vec![Button { key: "reply".into(), label: "Reply".into() }]);
    }

    #[test]
    fn reads_the_hints() {
        let hints = [
            ("urgency", 2u8.to_variant()),
            ("desktop-entry", "org.gnome.Nautilus".to_variant()),
            ("image_path", "/tmp/a.png".to_variant()),
            ("x-canonical-private-synchronous", "volume".to_variant()),
            ("resident", true.to_variant()),
            ("transient", true.to_variant()),
        ];
        let notice = Notice::from_notify(&notify(&[], &hints, -1));
        assert!(notice.is_critical());
        assert_eq!(notice.desktop_entry, "org.gnome.Nautilus");
        assert_eq!(notice.image_path, "/tmp/a.png");
        assert_eq!(notice.synchronous, "volume");
        assert!(notice.resident && notice.transient);
    }

    #[test]
    fn a_hint_of_the_wrong_type_is_ignored() {
        let notice = Notice::from_notify(&notify(&[], &[("urgency", "high".to_variant())], -1));
        assert_eq!(notice.urgency, NORMAL);
    }

    #[test]
    fn reads_a_picture_under_any_of_its_names() {
        let pixels = (1i32, 1i32, 4i32, true, 8i32, 4i32, vec![255u8; 4]).to_variant();
        let notice = Notice::from_notify(&notify(&[], &[("icon_data", pixels)], -1));
        assert!(notice.image_data.is_some());
    }
}
