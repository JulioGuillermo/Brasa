//! The app launcher: a search field over every installed app, as a grid of icons or a list.

mod grid;
mod icon;
mod keys;
mod list;
mod metrics;
mod results;
mod search;

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use gtk4::gio::AppInfoMonitor;
use gtk4::prelude::*;
use gtk4::{gdk, glib};

use crate::config::{LauncherView, Size};
use crate::services::apps::{self, InstalledApp, SearchText};
use keys::KeyAction;
use metrics::metrics_for;
use results::Results;

pub struct Launcher {
    widget: gtk4::Box,
    view: LauncherView,
    search: gtk4::SearchEntry,
    results: Rc<Results>,
    installed: RefCell<Vec<InstalledApp>>,
    search_texts: RefCell<Vec<SearchText>>,
    cards_are_stale: Cell<bool>,
    on_done: Box<dyn Fn()>,
}

impl Launcher {
    /// on_done: an app was launched, or Escape.
    pub fn new(view: LauncherView, size: Size, on_done: impl Fn() + 'static) -> Rc<Launcher> {
        Rc::new_cyclic(|weak: &Weak<Launcher>| {
            let on_pick = weak.clone();
            let results = Results::new(view, metrics_for(size), move |app_position| {
                if let Some(launcher) = on_pick.upgrade() {
                    launcher.launch(app_position);
                }
            });
            let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
            let search = search::search_entry();
            widget.append(&search);
            for part in results.widgets() {
                widget.append(&part);
            }
            Launcher {
                widget,
                view,
                search,
                results,
                installed: RefCell::new(Vec::new()),
                search_texts: RefCell::new(Vec::new()),
                cards_are_stale: Cell::new(true),
                on_done: Box::new(on_done),
            }
        })
        .connected()
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.widget.clone().upcast()
    }

    /// Clears the search and takes the keyboard focus.
    pub fn opened(&self) {
        self.search.set_text("");
        self.refill();
        self.search.grab_focus();
    }

    fn connected(self: Rc<Self>) -> Rc<Self> {
        let weak = Rc::downgrade(&self);
        self.search.connect_search_changed({
            let weak = weak.clone();
            move |_entry| with(&weak, Launcher::refill)
        });
        self.search.connect_activate({
            let weak = weak.clone();
            move |_entry| with(&weak, Launcher::launch_chosen)
        });
        let keys = gtk4::EventControllerKey::new();
        keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        keys.connect_key_pressed({
            let weak = weak.clone();
            move |_keys, key, _code, _state| press(&weak, key)
        });
        self.widget.add_controller(keys);
        AppInfoMonitor::get().connect_changed({
            let weak = weak.clone();
            move |_monitor| with(&weak, Launcher::apps_changed)
        });
        glib::idle_add_local_once(move || with(&weak, Launcher::refill));
        self
    }

    fn apps_changed(&self) {
        self.cards_are_stale.set(true);
        self.refill();
    }

    fn refill(&self) {
        if self.cards_are_stale.replace(false) {
            let installed = apps::installed();
            self.results.build_cards(&installed);
            *self.search_texts.borrow_mut() = installed.iter().map(|app| app.search_text.clone()).collect();
            *self.installed.borrow_mut() = installed;
        }
        let matches = apps::matching(&self.search_texts.borrow(), &self.search.text());
        self.results.show(matches);
    }

    fn launch_chosen(&self) {
        if let Some(app_position) = self.results.chosen_app_position() {
            self.launch(app_position);
        }
    }

    fn launch(&self, app_position: usize) {
        apps::launch(&self.installed.borrow()[app_position].info);
        (self.on_done)();
    }

    fn handle_key(&self, key: gdk::Key) -> glib::Propagation {
        match keys::action_for(key, self.view, self.results.columns()) {
            Some(KeyAction::MoveChoice(step)) => self.results.move_choice(step),
            Some(KeyAction::Close) => (self.on_done)(),
            None => return glib::Propagation::Proceed,
        }
        glib::Propagation::Stop
    }
}

fn with(weak: &Weak<Launcher>, action: impl FnOnce(&Launcher)) {
    if let Some(launcher) = weak.upgrade() {
        action(&launcher);
    }
}

fn press(weak: &Weak<Launcher>, key: gdk::Key) -> glib::Propagation {
    match weak.upgrade() {
        Some(launcher) => launcher.handle_key(key),
        None => glib::Propagation::Proceed,
    }
}
