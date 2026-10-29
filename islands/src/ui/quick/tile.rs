//! One tile: a square button with its icon and its name inside, centred. Lit (red) while what it
//! switches is on. A menu tile has a chevron after its name.

use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use gtk4::pango;

use super::items::{Item, Menu};
use super::{glyphs, shell_test};
use crate::runtime::events::EventSender;
use crate::services::process;

const NAME_CHARACTERS: i32 = 11;
const ICON_PIXELS: i32 = 22;
const CLICK_HOLD: Duration = Duration::from_secs(2);

pub struct Tile {
    pub root: gtk4::Box,
    pub button: gtk4::Button,
    item: Item,
    events: EventSender,
    lit: Cell<bool>,
    clicked_at: Cell<Option<Instant>>,
}

impl Tile {
    /// on_command(command): closes the panel and runs it. on_menu(menu): shows the menu.
    pub fn new(
        item: Item,
        events: &EventSender,
        on_command: Rc<dyn Fn(&str)>,
        on_menu: Rc<dyn Fn(&Menu)>,
    ) -> Rc<Tile> {
        let face = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();
        face.append(&icon_widget(item.icon()));
        face.append(&name_row(&item));
        let button = gtk4::Button::builder()
            .child(&face)
            .css_classes(["quick-tile-button"])
            .hexpand(true)
            .tooltip_text(item.name())
            .build();
        let root = gtk4::Box::builder().css_classes(["quick-tile"]).build();
        root.append(&button);

        let tile = Rc::new(Tile {
            root,
            button,
            item,
            events: events.clone(),
            lit: Cell::new(false),
            clicked_at: Cell::new(None),
        });
        let me = Rc::downgrade(&tile);
        tile.button.connect_clicked(move |_| click(&me, &on_command, &on_menu));
        tile
    }

    /// The test of the switch this tile is, if it is one.
    pub fn state_test(&self) -> Option<&str> {
        match &self.item {
            Item::Switch(switch) => Some(&switch.state),
            _ => None,
        }
    }

    pub fn refresh(&self) {
        if let Some(test) = self.state_test() {
            shell_test::ask(&self.events, test);
        }
    }

    /// A click a moment ago wins over a test that still saw the old state.
    pub fn show_state(&self, lit: bool) {
        let just_clicked = self.clicked_at.get().is_some_and(|at| at.elapsed() < CLICK_HOLD);
        if !just_clicked {
            self.show_lit(lit);
        }
    }

    fn show_lit(&self, lit: bool) {
        self.lit.set(lit);
        if lit {
            self.button.add_css_class("lit");
        } else {
            self.button.remove_css_class("lit");
        }
    }
}

fn click(me: &Weak<Tile>, on_command: &Rc<dyn Fn(&str)>, on_menu: &Rc<dyn Fn(&Menu)>) {
    let Some(tile) = me.upgrade() else { return };
    match &tile.item {
        Item::Switch(switch) => {
            process::detach(if tile.lit.get() { &switch.off } else { &switch.on });
            tile.clicked_at.set(Some(Instant::now()));
            tile.show_lit(!tile.lit.get());
        }
        Item::Action(action) => on_command(&action.command),
        Item::Menu(menu) => on_menu(menu),
    }
}

/// A Nerd Font glyph, or a themed icon name (…-symbolic) from panel.conf.
fn icon_widget(icon: &str) -> gtk4::Widget {
    if icon.ends_with("-symbolic") {
        return gtk4::Image::builder()
            .icon_name(icon)
            .pixel_size(ICON_PIXELS)
            .css_classes(["quick-tile-icon"])
            .build()
            .upcast();
    }
    gtk4::Label::builder().label(icon).css_classes(["quick-tile-icon"]).build().upcast()
}

fn name_row(item: &Item) -> gtk4::Box {
    let row = gtk4::Box::builder().spacing(2).halign(gtk4::Align::Center).build();
    let name = gtk4::Label::builder()
        .label(item.name())
        .ellipsize(pango::EllipsizeMode::End)
        .max_width_chars(NAME_CHARACTERS)
        .css_classes(["quick-tile-name"])
        .build();
    row.append(&name);
    if matches!(item, Item::Menu(_)) {
        row.append(&gtk4::Label::builder().label(glyphs::CHEVRON).css_classes(["quick-chevron"]).build());
    }
    row
}
