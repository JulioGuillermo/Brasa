//! Which of the notifications are shown, and where: live ones, then the fold, newest nearest the edge.

use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use super::Notices;
use super::row::NoticeRow;
use crate::services::notifications::bell;

impl Notices {
    pub(super) fn tell_island(&self) {
        (self.on_change)();
    }

    /// Newest nearest the edge's side of the list, header between it and the music.
    pub fn set_edge(&self, at_top: bool) {
        self.header.show_edge(at_top);
        self.order_children(at_top);
        let rows = {
            let mut state = self.state.borrow_mut();
            if state.newest_first == at_top {
                return;
            }
            state.newest_first = at_top;
            state.rows.clone()
        };
        for row in &rows {
            row.remove_from_parent();
        }
        for row in &rows {
            self.place(row);
        }
    }

    /// In the live list or under the fold, newest nearest the edge.
    pub(super) fn place(&self, row: &Rc<NoticeRow>) {
        let (taken_back, newest_first) = {
            let state = self.state.borrow();
            (state.closed_by_app.contains(&row.id()), state.newest_first)
        };
        let target = if taken_back { self.fold.list() } else { &self.live_list };
        if newest_first {
            target.prepend(row.widget());
        } else {
            target.append(row.widget());
        }
    }

    pub(super) fn refresh(&self) {
        let (centre, dnd, rows, toasts, closed_by_app) = {
            let state = self.state.borrow();
            (state.centre, state.dnd, state.rows.clone(), state.toasts.clone(), state.closed_by_app.clone())
        };
        let live = rows.len() - closed_by_app.len();
        self.header.show_count(live);
        self.header.widget().set_visible(centre);
        self.nothing.set_visible(centre && rows.is_empty());
        self.scroller.set_visible(if centre { !rows.is_empty() } else { !toasts.is_empty() });
        self.fold.button().set_visible(centre && !closed_by_app.is_empty());
        self.fold.show_count(closed_by_app.len());
        self.fold.revealer().set_visible(centre);
        for row in &rows {
            row.widget().set_visible(centre || toasts.contains(&row.id()));
        }
        self.scroll_to_newest_soon();
        bell::ring(live, dnd);
        self.save_soon();
        self.tell_island();
    }

    fn order_children(&self, at_top: bool) {
        let fold_button = self.fold.button();
        let fold_list = self.fold.revealer();
        if at_top {
            put_in_order(&self.widget, &[self.header.widget().upcast_ref(), self.scroller.upcast_ref(), self.nothing.upcast_ref()]);
            put_in_order(&self.lists, &[self.live_list.upcast_ref(), fold_button.upcast_ref(), fold_list.upcast_ref()]);
        } else {
            put_in_order(&self.widget, &[self.nothing.upcast_ref(), self.scroller.upcast_ref(), self.header.widget().upcast_ref()]);
            put_in_order(&self.lists, &[fold_list.upcast_ref(), fold_button.upcast_ref(), self.live_list.upcast_ref()]);
        }
    }

    fn scroll_to_newest_soon(&self) {
        let me = self.me.clone();
        glib::idle_add_local_once(move || {
            let Some(notices) = me.upgrade() else { return };
            let scroll = notices.scroller.vadjustment();
            let newest_first = notices.state.borrow().newest_first;
            scroll.set_value(if newest_first { 0.0 } else { scroll.upper() });
        });
    }
}

fn put_in_order(parent: &gtk4::Box, children: &[&gtk4::Widget]) {
    let mut before: Option<&gtk4::Widget> = None;
    for child in children {
        parent.reorder_child_after(*child, before);
        before = Some(child);
    }
}
