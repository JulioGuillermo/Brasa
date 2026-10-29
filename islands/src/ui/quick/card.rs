//! What the panel shows inside its outline: the tiles or an opened menu, the sliders and the footer.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;

use super::footer::Footer;
use super::items::Menu;
use super::menu_view::MenuView;
use super::pager::Pager;
use super::sliders::{self, Slider};
use super::catalog;
use crate::config::Size;
use crate::runtime::events::EventSender;
use crate::services::hardware;

const TILES_VIEW: &str = "tiles";
const MENU_VIEW: &str = "menu";

pub struct Card {
    pub root: gtk4::Box,
    pub pager: Rc<Pager>,
    pub menu: Rc<MenuView>,
    pub sliders: Vec<Slider>,
    pub footer: Footer,
    views: gtk4::Stack,
    menu_tile: Cell<usize>,
    on_resize: Rc<dyn Fn()>,
}

impl Card {
    /// on_command(command): closes the panel and runs it. on_resize(): the card's height changed.
    pub fn new(
        size: Size,
        events: &EventSender,
        on_command: Rc<dyn Fn(&str)>,
        on_resize: Rc<dyn Fn()>,
    ) -> Rc<Card> {
        Rc::new_cyclic(|me: &std::rc::Weak<Card>| {
            let for_menu = me.clone();
            let pager = Pager::new(
                catalog::tiles(),
                size,
                events,
                Rc::clone(&on_command),
                Rc::new(move |index, menu| {
                    if let Some(card) = for_menu.upgrade() {
                        card.show_menu(index, menu);
                    }
                }),
            );
            let for_leave = me.clone();
            let menu = MenuView::new(
                events,
                Rc::new(move || {
                    if let Some(card) = for_leave.upgrade() {
                        card.show_tiles();
                    }
                }),
            );
            let views = gtk4::Stack::builder().vhomogeneous(false).hhomogeneous(true).build();
            views.add_named(&pager.root, Some(TILES_VIEW));
            views.add_named(&menu.root, Some(MENU_VIEW));

            let sliders = sliders::sliders(hardware::has_backlight());
            let footer = Footer::new(on_command);
            let root = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(14)
                .css_classes(["quick-card"])
                .build();
            root.append(&views);
            for slider in &sliders {
                root.append(&slider.root);
            }
            root.append(&footer.root);
            Card { root, pager, menu, sliders, footer, views, menu_tile: Cell::new(0), on_resize }
        })
    }

    pub fn showing_menu(&self) -> bool {
        self.views.visible_child_name().as_deref() == Some(MENU_VIEW)
    }

    fn show_menu(&self, tile_index: usize, menu: &Menu) {
        self.menu_tile.set(tile_index);
        self.menu.show_menu(menu);
        self.views.set_visible_child_name(MENU_VIEW);
        self.menu.focus_first();
        (self.on_resize)();
    }

    pub fn show_tiles(&self) {
        self.views.set_visible_child_name(TILES_VIEW);
        self.pager.focus_tile(self.menu_tile.get());
        (self.on_resize)();
    }

    pub fn refresh_sliders(&self) {
        for slider in &self.sliders {
            slider.refresh();
        }
    }

    pub fn on_tested(&self, test: &str, passed: bool) {
        self.pager.on_tested(test, passed);
        self.menu.on_tested(test, passed);
    }
}
