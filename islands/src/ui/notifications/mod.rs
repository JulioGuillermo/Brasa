//! The notifications section. Closed, the island shows only the ones that just arrived (toasts); open,
//! it is the notification centre: every notification kept, newest first, with Do Not Disturb, Clear all
//! and a button that closes the island. One its app takes back stays, folded away under "Closed by their app".

mod arrivals;
mod deadlines;
mod departures;
mod desktop;
mod fold;
mod header;
mod icon;
mod layout;
mod markup;
mod persistence;
mod row;
mod timers;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::{Rc, Weak};

use gtk4::prelude::*;

use crate::services::notifications::server::Server;
use deadlines::Deadlines;
use fold::Fold;
use header::Header;
use row::NoticeRow;

const TOASTS_MAX_HEIGHT: i32 = 320;
const CENTRE_MAX_HEIGHT: i32 = 300;
const MAX_KEPT: usize = 50;

struct State {
    rows: Vec<Rc<NoticeRow>>,
    closed_by_app: HashSet<u32>,
    toasts: HashSet<u32>,
    deadlines: Deadlines,
    last_id: u32,
    dnd: bool,
    centre: bool,
    hovered: bool,
    newest_first: bool,
    checking_deadlines: bool,
    save_pending: bool,
}

pub struct Notices {
    me: Weak<Notices>,
    widget: gtk4::Box,
    header: Header,
    scroller: gtk4::ScrolledWindow,
    lists: gtk4::Box,
    live_list: gtk4::Box,
    fold: Rc<Fold>,
    nothing: gtk4::Label,
    server: Rc<Server>,
    on_change: Box<dyn Fn()>,
    state: RefCell<State>,
}

impl Notices {
    /// on_change: what it shows changed. on_close_island: the header's close button.
    pub fn new(on_change: impl Fn() + 'static, on_close_island: impl Fn() + 'static) -> Rc<Notices> {
        let notices = Rc::new_cyclic(|me: &Weak<Notices>| Notices::build(me, Box::new(on_change), Box::new(on_close_island)));
        notices.restore();
        notices.refresh();
        notices
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.widget.clone().upcast()
    }

    pub fn has_toasts(&self) -> bool {
        !self.state.borrow().toasts.is_empty()
    }

    pub fn set_centre(&self, centre: bool) {
        self.state.borrow_mut().centre = centre;
        self.scroller.set_max_content_height(if centre { CENTRE_MAX_HEIGHT } else { TOASTS_MAX_HEIGHT });
        self.refresh();
    }

    /// While the pointer is on the island the toasts wait.
    pub fn set_hovered(&self, hovered: bool) {
        self.state.borrow_mut().hovered = hovered;
        if !hovered {
            self.give_toasts_time_after_hover();
        }
    }

    pub fn toggle_dnd(&self) {
        let dnd = {
            let mut state = self.state.borrow_mut();
            state.dnd = !state.dnd;
            state.dnd
        };
        self.header.show_dnd(dnd);
        self.refresh();
    }

    fn build(me: &Weak<Notices>, on_change: Box<dyn Fn()>, on_close_island: Box<dyn Fn()>) -> Notices {
        let header = Header::new(
            tell(me, Notices::toggle_dnd),
            tell(me, Notices::clear_all),
            on_close_island,
        );
        let fold = Fold::new(tell(me, Notices::tell_island));
        let live_list = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).build();
        let lists = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).build();
        lists.append(&live_list);
        lists.append(fold.button());
        lists.append(fold.revealer());
        let scroller = gtk4::ScrolledWindow::builder()
            .child(&lists)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(TOASTS_MAX_HEIGHT)
            .css_classes(["island-notices"])
            .build();
        let nothing = gtk4::Label::builder()
            .label("No notifications")
            .css_classes(["island-time"])
            .margin_top(6)
            .margin_bottom(6)
            .build();
        let widget = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(8).build();
        widget.append(header.widget());
        widget.append(&scroller);
        widget.append(&nothing);

        let on_notify = me.clone();
        let on_take_back = me.clone();
        let server = Server::start(
            move |notice| on_notify.upgrade().map_or(0, |notices| notices.on_notify(notice)),
            move |notice_id| {
                if let Some(notices) = on_take_back.upgrade() {
                    notices.on_take_back(notice_id);
                }
            },
        );
        Notices {
            me: me.clone(),
            widget,
            header,
            scroller,
            lists,
            live_list,
            fold,
            nothing,
            server,
            on_change,
            state: RefCell::new(State::new()),
        }
    }

    fn row(&self, notice_id: u32) -> Option<Rc<NoticeRow>> {
        self.state.borrow().rows.iter().find(|row| row.id() == notice_id).cloned()
    }
}

impl State {
    fn new() -> State {
        State {
            rows: Vec::new(),
            closed_by_app: HashSet::new(),
            toasts: HashSet::new(),
            deadlines: Deadlines::default(),
            last_id: 0,
            dnd: false,
            centre: false,
            hovered: false,
            newest_first: true,
            checking_deadlines: false,
            save_pending: false,
        }
    }
}

/// A callback for a widget that runs one of Notices' methods, if Notices is still there.
fn tell(me: &Weak<Notices>, action: fn(&Notices)) -> impl Fn() + 'static {
    let me = me.clone();
    move || {
        if let Some(notices) = me.upgrade() {
            action(&notices);
        }
    }
}
