//! The tiles in pages, as many per page as the panel's size holds. One turn of the mouse wheel, or one
//! swipe on the touchpad, moves exactly one page; the dots show where you are.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use gtk4::gdk;

use super::grid::{Arrow, Grid};
use super::items::{Item, Menu};
use super::tile::Tile;
use crate::config::Size;
use crate::runtime::events::EventSender;

const SLIDE_MS: u32 = 220;
const WHEEL_PAUSE: Duration = Duration::from_millis(350);
const SWIPE_PIXELS: f64 = 30.0;

pub struct Pager {
    pub root: gtk4::Box,
    pub tiles: Vec<Rc<Tile>>,
    grid: Grid,
    stack: gtk4::Stack,
    dots: Vec<gtk4::Button>,
    current: Cell<usize>,
    last_wheel_turn: Cell<Option<Instant>>,
    swipe: Cell<f64>,
    swipe_used: Cell<bool>,
}

impl Pager {
    /// on_command(command): closes the panel and runs it. on_menu(index, menu): shows the menu of tile `index`.
    pub fn new(
        items: Vec<Item>,
        size: Size,
        events: &EventSender,
        on_command: Rc<dyn Fn(&str)>,
        on_menu: Rc<dyn Fn(usize, &Menu)>,
    ) -> Rc<Pager> {
        let tiles: Vec<Rc<Tile>> = items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let on_menu = Rc::clone(&on_menu);
                Tile::new(item, events, Rc::clone(&on_command), Rc::new(move |menu| on_menu(index, menu)))
            })
            .collect();
        let grid = Grid::new(size, tiles.len());
        let stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::SlideUpDown)
            .transition_duration(SLIDE_MS)
            .hexpand(true)
            .build();
        let dots = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .css_classes(["quick-dots"])
            .visible(grid.pages() > 1)
            .build();
        let root = gtk4::Box::builder().spacing(10).css_classes(["quick-pager"]).build();
        root.append(&stack);
        root.append(&dots);

        let mut dot_buttons = Vec::new();
        for (number, page_tiles) in tiles.chunks(grid.per_page()).enumerate() {
            stack.add_named(&page(&grid, page_tiles), Some(&number.to_string()));
            let dot = gtk4::Button::builder()
                .css_classes(["quick-dot"])
                .tooltip_text(format!("Page {}", number + 1))
                .focusable(false)
                .build();
            dots.append(&dot);
            dot_buttons.push(dot);
        }

        let pager = Rc::new(Pager {
            root,
            tiles,
            grid,
            stack,
            dots: dot_buttons,
            current: Cell::new(0),
            last_wheel_turn: Cell::new(None),
            swipe: Cell::new(0.0),
            swipe_used: Cell::new(false),
        });
        pager.connect_dots();
        pager.listen_to_wheel();
        pager.show_page(0);
        pager
    }

    pub fn current_page(&self) -> usize {
        self.current.get()
    }

    pub fn per_page(&self) -> usize {
        self.grid.per_page()
    }

    pub fn neighbour(&self, index: usize, arrow: Arrow) -> Option<usize> {
        self.grid.neighbour(index, arrow)
    }

    pub fn show_page(&self, number: usize) {
        self.current.set(number.min(self.dots.len() - 1));
        self.stack.set_visible_child_name(&self.current.get().to_string());
        for (index, dot) in self.dots.iter().enumerate() {
            if index == self.current.get() {
                dot.add_css_class("current");
            } else {
                dot.remove_css_class("current");
            }
        }
    }

    pub fn refresh(&self) {
        for tile in &self.tiles {
            tile.refresh();
        }
    }

    /// A shell test finished: the switches whose state it is show it.
    pub fn on_tested(&self, test: &str, passed: bool) {
        for tile in self.tiles.iter().filter(|tile| tile.state_test() == Some(test)) {
            tile.show_state(passed);
        }
    }

    // ---- Keyboard

    pub fn focused_index(&self, focus: Option<&gtk4::Widget>) -> Option<usize> {
        let focus = focus?;
        self.tiles.iter().position(|tile| tile.button.upcast_ref::<gtk4::Widget>() == focus)
    }

    pub fn focus_tile(&self, index: usize) {
        let index = index.min(self.tiles.len() - 1);
        self.show_page(index / self.per_page());
        self.tiles[index].button.grab_focus();
    }

    // ---- Dots, wheel and touchpad

    fn connect_dots(self: &Rc<Self>) {
        for (number, dot) in self.dots.iter().enumerate() {
            let me = Rc::downgrade(self);
            dot.connect_clicked(move |_| {
                if let Some(pager) = me.upgrade() {
                    pager.show_page(number);
                }
            });
        }
    }

    fn listen_to_wheel(self: &Rc<Self>) {
        let wheel = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
        let me = Rc::downgrade(self);
        wheel.connect_scroll_begin(move |_| {
            if let Some(pager) = me.upgrade() {
                pager.swipe.set(0.0);
                pager.swipe_used.set(false);
            }
        });
        let me = Rc::downgrade(self);
        wheel.connect_scroll(move |controller, _dx, dy| {
            let Some(pager) = me.upgrade() else { return gtk4::glib::Propagation::Proceed };
            if controller.unit() == gdk::ScrollUnit::Wheel {
                pager.on_wheel_step(dy);
            } else {
                pager.on_swipe_step(dy);
            }
            gtk4::glib::Propagation::Stop
        });
        self.root.add_controller(wheel);
    }

    fn on_wheel_step(&self, dy: f64) {
        let paused = self.last_wheel_turn.get().is_none_or(|at| at.elapsed() >= WHEEL_PAUSE);
        if dy == 0.0 || !paused {
            return;
        }
        self.last_wheel_turn.set(Some(Instant::now()));
        self.turn(dy > 0.0);
    }

    fn on_swipe_step(&self, dy: f64) {
        if self.swipe_used.get() {
            return;
        }
        self.swipe.set(self.swipe.get() + dy);
        if self.swipe.get().abs() >= SWIPE_PIXELS {
            self.swipe_used.set(true);
            self.turn(self.swipe.get() > 0.0);
        }
    }

    fn turn(&self, forward: bool) {
        let target = if forward { self.current.get() + 1 } else { self.current.get().saturating_sub(1) };
        self.show_page(target);
        let focus = self.root.root().and_then(|root| root.focus());
        if self.focused_index(focus.as_ref()).is_some() {
            self.tiles[self.current.get() * self.per_page()].button.grab_focus();
        }
    }
}

fn page(grid: &Grid, tiles: &[Rc<Tile>]) -> gtk4::Grid {
    let page = gtk4::Grid::builder()
        .column_homogeneous(true)
        .row_homogeneous(true)
        .column_spacing(10)
        .row_spacing(12)
        .build();
    for place in 0..grid.per_page() {
        let cell: gtk4::Widget = match tiles.get(place) {
            Some(tile) => tile.root.clone().upcast(),
            None => gtk4::Box::builder().css_classes(["quick-tile"]).build().upcast(),
        };
        page.attach(&cell, (place % grid.columns) as i32, (place / grid.columns) as i32, 1, 1);
    }
    page
}
