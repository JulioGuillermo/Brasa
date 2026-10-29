//! How one notification looks in the island: icon, app, title, text and its buttons.
//! A stale row is one its app no longer listens to (it closed it, or the island restarted): greyed
//! buttons, and a click opens the app.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::pango;
use gtk4::prelude::*;

use super::icon::show_icon;
use super::markup::safe_markup;
use crate::services::notifications::notice::{DEFAULT_ACTION, Notice};
use crate::ui::components::round_button::round_button;

const ICON_SIZE: i32 = 36;
const BODY_LINES: i32 = 4;
const SUMMARY_LINES: i32 = 2;
const STALE_HINT: &str = "Its app no longer answers this one";

/// on_close(notice id) deletes it; on_press(notice id, action key) is for its buttons and for a click on it.
pub struct Callbacks {
    pub on_close: Box<dyn Fn(u32)>,
    pub on_press: Box<dyn Fn(u32, &str)>,
}

pub struct NoticeRow {
    widget: gtk4::Box,
    icon: gtk4::Image,
    app: gtk4::Label,
    summary: gtk4::Label,
    body: gtk4::Label,
    buttons: gtk4::Box,
    notice: RefCell<Notice>,
    stale: Cell<bool>,
    callbacks: Callbacks,
}

impl NoticeRow {
    pub fn new(notice: Notice, callbacks: Callbacks) -> Rc<NoticeRow> {
        let icon = gtk4::Image::builder()
            .pixel_size(ICON_SIZE)
            .valign(gtk4::Align::Start)
            .css_classes(["island-notice-icon"])
            .build();
        let app = gtk4::Label::builder()
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .css_classes(["island-notice-app"])
            .build();
        let summary = wrapping_label(SUMMARY_LINES, "island-title");
        let body = wrapping_label(BODY_LINES, "island-notice-body");
        let buttons = gtk4::Box::builder().spacing(6).homogeneous(true).css_classes(["island-notice-buttons"]).build();
        let row = Rc::new(NoticeRow {
            widget: gtk4::Box::builder().spacing(12).css_classes(["island-notice"]).build(),
            icon,
            app,
            summary,
            body,
            buttons,
            notice: RefCell::new(notice.clone()),
            stale: Cell::new(false),
            callbacks,
        });
        row.assemble();
        row.show_notice(notice);
        row
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.widget
    }

    pub fn id(&self) -> u32 {
        self.notice.borrow().id
    }

    pub fn notice(&self) -> Notice {
        self.notice.borrow().clone()
    }

    pub fn is_stale(&self) -> bool {
        self.stale.get()
    }

    pub fn remove_from_parent(&self) {
        let Some(parent) = self.widget.parent().and_downcast::<gtk4::Box>() else { return };
        parent.remove(&self.widget);
    }

    pub fn show_notice(self: &Rc<Self>, notice: Notice) {
        show_icon(&self.icon, &notice);
        self.widget.set_css_classes(if notice.is_critical() { &["island-notice", "critical"] } else { &["island-notice"] });
        self.app.set_text(&notice.app);
        self.summary.set_text(&notice.summary);
        self.summary.set_visible(!notice.summary.is_empty());
        self.body.set_markup(&safe_markup(&notice.body));
        self.body.set_visible(!notice.body.is_empty());
        self.show_buttons(&notice);
        *self.notice.borrow_mut() = notice;
        self.set_stale(self.stale.get());
    }

    pub fn set_stale(&self, stale: bool) {
        self.stale.set(stale);
        if stale {
            self.widget.add_css_class("stale");
        } else {
            self.widget.remove_css_class("stale");
        }
        let mut button = self.buttons.first_child();
        while let Some(current) = button {
            current.set_sensitive(!stale);
            current.set_tooltip_text(stale.then_some(STALE_HINT));
            button = current.next_sibling();
        }
    }

    fn assemble(self: &Rc<Self>) {
        let id_of = Rc::downgrade(self);
        let close = round_button("window-close-symbolic", &["island-close"], move || {
            if let Some(row) = id_of.upgrade() {
                (row.callbacks.on_close)(row.id());
            }
        });
        close.set_valign(gtk4::Align::Start);
        close.set_tooltip_text(Some("Delete"));

        let header = gtk4::Box::builder().spacing(6).build();
        header.append(&self.app);
        header.append(&close);
        let texts = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(2).hexpand(true).build();
        for child in [header.upcast_ref::<gtk4::Widget>(), self.summary.upcast_ref(), self.body.upcast_ref(), self.buttons.upcast_ref()] {
            texts.append(child);
        }
        self.widget.append(&self.icon);
        self.widget.append(&texts);

        let click = gtk4::GestureClick::new();
        let clicked = Rc::downgrade(self);
        click.connect_released(move |gesture, _presses, _x, _y| {
            let Some(row) = clicked.upgrade() else { return };
            if row.is_clickable() {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                (row.callbacks.on_press)(row.id(), DEFAULT_ACTION);
            }
        });
        self.widget.add_controller(click);
    }

    fn show_buttons(self: &Rc<Self>, notice: &Notice) {
        while let Some(child) = self.buttons.first_child() {
            self.buttons.remove(&child);
        }
        for button in &notice.buttons {
            let widget = gtk4::Button::builder().label(&button.label).css_classes(["island-action"]).build();
            let row = Rc::downgrade(self);
            let key = button.key.clone();
            widget.connect_clicked(move |_| {
                if let Some(row) = row.upgrade() {
                    (row.callbacks.on_press)(row.id(), &key);
                }
            });
            self.buttons.append(&widget);
        }
        self.buttons.set_visible(!notice.buttons.is_empty());
    }

    fn is_clickable(&self) -> bool {
        let notice = self.notice.borrow();
        if self.stale.get() { !notice.desktop_entry.is_empty() } else { notice.has_default }
    }
}

fn wrapping_label(lines: i32, css_class: &str) -> gtk4::Label {
    gtk4::Label::builder()
        .xalign(0.0)
        .wrap(true)
        .wrap_mode(pango::WrapMode::WordChar)
        .lines(lines)
        .ellipsize(pango::EllipsizeMode::End)
        .css_classes([css_class])
        .build()
}
