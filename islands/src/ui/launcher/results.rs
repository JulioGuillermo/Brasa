//! The scrolling grid or list of matching apps. One card per installed app is built once and reused,
//! so a search only takes cards out of the flow box and puts the matching ones back.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;

use super::grid::grid_tile;
use super::list::list_row;
use super::metrics::Metrics;
use crate::config::LauncherView;
use crate::services::apps::InstalledApp;

const CHOSEN: &str = "chosen";

pub struct Results {
    view: LauncherView,
    metrics: Metrics,
    flow: gtk4::FlowBox,
    scroller: gtk4::ScrolledWindow,
    nothing_found: gtk4::Label,
    cards: RefCell<Vec<gtk4::FlowBoxChild>>,
    shown: RefCell<Vec<usize>>,
    chosen: Cell<usize>,
}

impl Results {
    /// on_pick: a card was clicked; gets the position of its app among the installed ones.
    pub fn new(view: LauncherView, metrics: Metrics, on_pick: impl Fn(usize) + 'static) -> Rc<Results> {
        let columns = if view == LauncherView::Grid { metrics.columns } else { 1 };
        let flow = gtk4::FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .homogeneous(true)
            .valign(gtk4::Align::Start)
            .row_spacing(4)
            .column_spacing(4)
            .min_children_per_line(columns)
            .max_children_per_line(columns)
            .build();
        let scroller = gtk4::ScrolledWindow::builder()
            .child(&flow)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .css_classes(["island-apps"])
            .build();
        let nothing_found = gtk4::Label::builder().label("No app matches").css_classes(["island-time"]).vexpand(true).build();
        scroller.set_size_request(-1, metrics.list_height);
        nothing_found.set_size_request(-1, metrics.list_height);

        let results = Rc::new(Results {
            view,
            metrics,
            flow,
            scroller,
            nothing_found,
            cards: RefCell::new(Vec::new()),
            shown: RefCell::new(Vec::new()),
            chosen: Cell::new(0),
        });
        let weak = Rc::downgrade(&results);
        results.flow.connect_child_activated(move |_flow, card| {
            let Some(results) = weak.upgrade() else { return };
            if let Some(app_position) = results.app_position_of(card) {
                on_pick(app_position);
            }
        });
        results
    }

    pub fn widgets(&self) -> [gtk4::Widget; 2] {
        [self.scroller.clone().upcast(), self.nothing_found.clone().upcast()]
    }

    pub fn columns(&self) -> u32 {
        self.flow.max_children_per_line()
    }

    /// Builds a card for every app; the slow part, done once per change of the installed apps.
    pub fn build_cards(&self, apps: &[InstalledApp]) {
        self.flow.remove_all();
        let make = match self.view {
            LauncherView::Grid => grid_tile,
            LauncherView::List => list_row,
        };
        let cards = apps
            .iter()
            .map(|app| {
                let card = gtk4::FlowBoxChild::new();
                card.set_child(Some(&make(&app.info, &self.metrics)));
                card
            })
            .collect();
        *self.cards.borrow_mut() = cards;
        self.shown.borrow_mut().clear();
    }

    /// Shows the cards of these apps, in this order, and chooses the first.
    pub fn show(&self, app_positions: Vec<usize>) {
        self.unmark_chosen();
        self.flow.remove_all();
        let cards = self.cards.borrow();
        for position in &app_positions {
            self.flow.append(&cards[*position]);
        }
        let found_something = !app_positions.is_empty();
        self.scroller.set_visible(found_something);
        self.nothing_found.set_visible(!found_something);
        *self.shown.borrow_mut() = app_positions;
        self.choose(0);
        self.scroller.vadjustment().set_value(0.0);
    }

    pub fn move_choice(&self, step: i32) {
        let target = self.chosen.get() as i32 + step;
        self.choose(target.max(0) as usize);
    }

    /// Position among the installed apps of the chosen one, if any is shown.
    pub fn chosen_app_position(&self) -> Option<usize> {
        self.shown.borrow().get(self.chosen.get()).copied()
    }

    fn choose(&self, position: usize) {
        let shown = self.shown.borrow();
        if shown.is_empty() {
            return;
        }
        self.unmark_chosen();
        let position = position.min(shown.len() - 1);
        self.chosen.set(position);
        let card = self.cards.borrow()[shown[position]].clone();
        card.add_css_class(CHOSEN);
        self.scroll_to(&card);
    }

    fn unmark_chosen(&self) {
        if let Some(card) = self.chosen_card() {
            card.remove_css_class(CHOSEN);
        }
    }

    fn chosen_card(&self) -> Option<gtk4::FlowBoxChild> {
        let app_position = self.chosen_app_position()?;
        self.cards.borrow().get(app_position).cloned()
    }

    fn app_position_of(&self, card: &gtk4::FlowBoxChild) -> Option<usize> {
        self.cards.borrow().iter().position(|candidate| candidate == card)
    }

    fn scroll_to(&self, card: &gtk4::FlowBoxChild) {
        let Some(bounds) = card.compute_bounds(&self.flow) else { return };
        let scroll = self.scroller.vadjustment();
        let top = f64::from(bounds.y());
        let bottom = top + f64::from(bounds.height());
        if top < scroll.value() {
            scroll.set_value(top);
        } else if bottom > scroll.value() + scroll.page_size() {
            scroll.set_value(bottom - scroll.page_size());
        }
    }
}
