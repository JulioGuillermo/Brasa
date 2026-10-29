//! A menu tile opened: its choices in a list, the current one lit, and a way back to the tiles.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::pango;
use gtk4::prelude::*;

use super::items::Menu;
use super::{glyphs, shell_test};
use crate::runtime::events::EventSender;
use crate::services::process;

const MAX_LIST_HEIGHT: i32 = 300;

struct ChoiceButton {
    button: gtk4::Button,
    current_test: String,
}

pub struct MenuView {
    pub root: gtk4::Box,
    back: gtk4::Button,
    title: gtk4::Label,
    list: gtk4::Box,
    choices: RefCell<Vec<ChoiceButton>>,
    events: EventSender,
    on_leave: Rc<dyn Fn()>,
}

impl MenuView {
    /// on_leave(): the back arrow was pressed, or a choice ran.
    pub fn new(events: &EventSender, on_leave: Rc<dyn Fn()>) -> Rc<MenuView> {
        let back = gtk4::Button::builder().label(glyphs::BACK).css_classes(["quick-round"]).tooltip_text("Back").build();
        let leave = Rc::clone(&on_leave);
        back.connect_clicked(move |_| leave());
        let title = gtk4::Label::builder().xalign(0.0).hexpand(true).css_classes(["quick-title"]).build();
        let header = gtk4::Box::builder().spacing(8).build();
        header.append(&back);
        header.append(&title);

        let list = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(4).build();
        let scroller = gtk4::ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(MAX_LIST_HEIGHT)
            .build();
        let root = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .css_classes(["quick-menu"])
            .build();
        root.append(&header);
        root.append(&scroller);
        Rc::new(MenuView { root, back, title, list, choices: RefCell::new(Vec::new()), events: events.clone(), on_leave })
    }

    pub fn show_menu(&self, menu: &Menu) {
        self.title.set_text(&menu.name);
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        let mut buttons = Vec::new();
        for choice in &menu.choices {
            let button = self.choice_button(&choice.label, choice.command.clone());
            shell_test::ask(&self.events, &choice.current);
            self.list.append(&button);
            buttons.push(ChoiceButton { button, current_test: choice.current.clone() });
        }
        *self.choices.borrow_mut() = buttons;
    }

    fn choice_button(&self, label: &str, command: String) -> gtk4::Button {
        let label = gtk4::Label::builder().label(label).xalign(0.0).ellipsize(pango::EllipsizeMode::End).build();
        let button = gtk4::Button::builder().child(&label).css_classes(["quick-choice"]).build();
        let leave = Rc::clone(&self.on_leave);
        button.connect_clicked(move |_| {
            process::detach(&command);
            leave();
        });
        button
    }

    /// A shell test finished: the choices whose test it is light up.
    pub fn on_tested(&self, test: &str, passed: bool) {
        if !passed {
            return;
        }
        let focus_was_on_choices = self.focus_is_on_choices();
        for choice in self.choices.borrow().iter().filter(|choice| choice.current_test == test) {
            choice.button.add_css_class("lit");
            if focus_was_on_choices {
                choice.button.grab_focus();
            }
        }
    }

    fn focus_is_on_choices(&self) -> bool {
        let Some(focus) = self.root.root().and_then(|root| root.focus()) else { return false };
        self.choices.borrow().iter().any(|choice| choice.button.upcast_ref::<gtk4::Widget>() == &focus)
    }

    pub fn focus_first(&self) {
        if let Some(first) = self.choices.borrow().first() {
            first.button.grab_focus();
        }
    }

    /// Up and down through the back arrow and the choices.
    pub fn move_focus(&self, focus: Option<&gtk4::Widget>, step: isize) {
        let mut order = vec![self.back.clone()];
        order.extend(self.choices.borrow().iter().map(|choice| choice.button.clone()));
        let current = focus.and_then(|focus| order.iter().position(|button| button.upcast_ref::<gtk4::Widget>() == focus));
        let Some(current) = current else {
            self.focus_first();
            return;
        };
        let target = (current as isize + step).clamp(0, order.len() as isize - 1);
        order[target as usize].grab_focus();
    }
}
