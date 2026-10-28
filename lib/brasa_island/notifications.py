"""The island's notifications section. Closed, the island shows only the ones that just arrived (toasts),
each leaving on its own or waiting for an answer. Open, it is a notification centre: every notification
swaync still holds, newest first, with Do Not Disturb, Clear all and a button that closes the island."""
import time

from gi.repository import GLib, Gtk

from .notice import LOW, Notice
from .notice_row import NoticeRow
from .notify_watch import NotifyWatch
from .swaync import Swaync

TOASTS_MAX_HEIGHT = 320
CENTRE_MAX_HEIGHT = 300
LOW_SECONDS = 4
NORMAL_SECONDS = 6
AFTER_HOVER_SECONDS = 2
CHECK_MS = 250


def seconds_shown(notice: Notice):
    """How long it stays as a toast; None: until it is answered or closed."""
    if notice.expire_ms > 0:
        return notice.expire_ms / 1000
    if notice.critical or notice.buttons:
        return None
    return LOW_SECONDS if notice.urgency == LOW else NORMAL_SECONDS


def header_button(icon, tooltip, on_click):
    button = Gtk.Button(icon_name=icon, tooltip_text=tooltip, css_classes=["island-button", "island-header-button"])
    button.connect("clicked", lambda _button: on_click())
    return button


class Notifications:
    """on_change() whenever what it shows changes; on_close_island() for the header's close button."""

    def __init__(self, on_change, on_close_island):
        self.on_change = on_change
        self.rows = {}
        self.toasts = set()
        self.deadlines = {}
        self.centre = False
        self.hovered = False
        self.newest_first = True
        self.check_source = 0
        self.running = False
        self.swaync = Swaync(self.on_swaync_state)
        self.watch = NotifyWatch(self.on_notice, self.on_finished)
        self.widget = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.header = self.build_header(on_close_island)
        self.list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.scroller = Gtk.ScrolledWindow(child=self.list, hscrollbar_policy=Gtk.PolicyType.NEVER,
                                           propagate_natural_height=True, max_content_height=TOASTS_MAX_HEIGHT,
                                           css_classes=["island-notices"])
        self.nothing = Gtk.Label(label="No notifications", css_classes=["island-time"], margin_top=6, margin_bottom=6)
        for child in (self.header, self.scroller, self.nothing):
            self.widget.append(child)

    def build_header(self, on_close_island):
        self.title = Gtk.Label(xalign=0, hexpand=True, css_classes=["island-title"])
        self.dnd_button = header_button("notifications-disabled-symbolic", "Do not disturb", self.toggle_dnd)
        clear = header_button("edit-clear-all-symbolic", "Clear all", self.swaync.close_all)
        close = header_button("go-up-symbolic", "Close the island", on_close_island)
        self.close_icon = close
        header = Gtk.Box(spacing=4, css_classes=["island-notices-header"])
        for child in (self.title, self.dnd_button, clear, close):
            header.append(child)
        return header

    # ---- What it shows

    @property
    def has_toasts(self) -> bool:
        return bool(self.toasts)

    def start(self):
        if self.running:
            return
        self.running = True
        self.swaync.start()
        self.watch.start()
        self.show_dnd()

    def stop(self):
        if not self.running:
            return
        self.running = False
        self.watch.stop()
        for notice_id in list(self.rows):
            self.forget(notice_id)
        self.swaync.stop()

    def set_centre(self, centre: bool):
        self.centre = centre
        self.scroller.set_max_content_height(CENTRE_MAX_HEIGHT if centre else TOASTS_MAX_HEIGHT)
        self.refresh()

    def set_edge(self, at_top: bool):
        """Newest nearest the edge's side of the list, header between it and the music."""
        self.close_icon.set_icon_name("go-up-symbolic" if at_top else "go-down-symbolic")
        order = [self.header, self.scroller, self.nothing] if at_top else [self.nothing, self.scroller, self.header]
        for before, child in zip([None, *order], order):
            self.widget.reorder_child_after(child, before)
        if self.newest_first == at_top:
            return
        self.newest_first = at_top
        rows = list(self.rows.values())
        for row in rows:
            self.list.remove(row)
        for row in rows:
            self.place(row)

    def refresh(self):
        count = len(self.rows)
        self.title.set_text(f"Notifications · {count}" if count else "Notifications")
        self.header.set_visible(self.centre)
        self.nothing.set_visible(self.centre and not count)
        self.scroller.set_visible(bool(self.rows) if self.centre else bool(self.toasts))
        for notice_id, row in self.rows.items():
            row.set_visible(self.centre or notice_id in self.toasts)
            row.set_buttons_live(self.swaync.can_press(notice_id))
        GLib.idle_add(self.scroll_to_newest)
        self.on_change()

    def place(self, row):
        if self.newest_first:
            self.list.prepend(row)
        else:
            self.list.append(row)

    def scroll_to_newest(self):
        scroll = self.scroller.get_vadjustment()
        scroll.set_value(0 if self.newest_first else scroll.get_upper())
        return GLib.SOURCE_REMOVE

    # ---- Arrivals

    def on_notice(self, notice: Notice):
        if not self.running:
            return
        replaced = self.replaced_id(notice)
        if replaced:
            row = self.rows.pop(replaced)
            self.toasts.discard(replaced)
            self.deadlines.pop(replaced, None)
            row.show_notice(notice)
        else:
            row = NoticeRow(notice, self.delete, self.press)
            self.place(row)
        self.rows[notice.id] = row
        if self.swaync.floats(notice.critical):
            self.swaync.added(notice.id, replaced)
            self.toasts.add(notice.id)
            self.set_deadline(notice)
        self.refresh()

    def replaced_id(self, notice: Notice) -> int:
        if notice.replaces_id in self.rows:
            return notice.replaces_id
        for notice_id, row in self.rows.items():
            if notice.synchronous and row.notice.synchronous == notice.synchronous:
                return notice_id
        return 0

    def set_deadline(self, notice: Notice):
        seconds = seconds_shown(notice)
        if seconds is None:
            return
        self.deadlines[notice.id] = time.monotonic() + seconds
        if not self.check_source:
            self.check_source = GLib.timeout_add(CHECK_MS, self.check_deadlines)

    def check_deadlines(self):
        if not self.hovered:
            now = time.monotonic()
            for notice_id in [n for n, deadline in self.deadlines.items() if deadline <= now]:
                self.end_toast(notice_id)
        if self.deadlines:
            return GLib.SOURCE_CONTINUE
        self.check_source = 0
        return GLib.SOURCE_REMOVE

    def set_hovered(self, hovered: bool):
        self.hovered = hovered
        if hovered:
            return
        later = time.monotonic() + AFTER_HOVER_SECONDS
        for notice_id, deadline in self.deadlines.items():
            self.deadlines[notice_id] = max(deadline, later)

    # ---- Departures

    def end_toast(self, notice_id):
        """It stops being a toast; it stays in the centre and in the panel."""
        self.deadlines.pop(notice_id, None)
        self.toasts.discard(notice_id)
        self.swaync.dismiss(notice_id)
        self.refresh()

    def delete(self, notice_id):
        self.swaync.close(notice_id)

    def press(self, notice_id, button_index):
        if not self.swaync.can_press(notice_id):
            return
        for hidden_id in self.swaync.press(notice_id, button_index):
            self.toasts.discard(hidden_id)
            self.deadlines.pop(hidden_id, None)
        self.refresh()

    def on_finished(self, notice_id):
        """swaync closed it: answered, closed by its app, deleted here or in the panel."""
        self.swaync.gone(notice_id)
        self.forget(notice_id)
        self.refresh()

    def forget(self, notice_id):
        row = self.rows.pop(notice_id, None)
        self.toasts.discard(notice_id)
        self.deadlines.pop(notice_id, None)
        if row is not None:
            self.list.remove(row)

    def on_swaync_state(self, _quiet, panel_open):
        if panel_open:
            self.toasts.clear()
            self.deadlines.clear()
        self.show_dnd()
        self.refresh()

    # ---- Do Not Disturb

    def toggle_dnd(self):
        self.swaync.set_dnd(not self.swaync.dnd)
        self.show_dnd()

    def show_dnd(self):
        if self.swaync.dnd:
            self.dnd_button.add_css_class("lit")
        else:
            self.dnd_button.remove_css_class("lit")
