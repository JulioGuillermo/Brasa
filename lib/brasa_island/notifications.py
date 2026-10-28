"""The island's notifications section, and the notification server behind it. Closed, the island shows only
the ones that just arrived (toasts), each leaving on its own or waiting for an answer. Open, it is the
notification centre: every notification kept, newest first, with Do Not Disturb, Clear all and a button
that closes the island."""
import time

from gi.repository import GLib, Gtk

from .bell import ring
from .notice import LOW, Notice
from .notice_row import NoticeRow
from .notify_server import CLOSED_BY_APP, DISMISSED, EXPIRED, NotifyServer
from .sound import chime

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
        self.last_id = 0
        self.dnd = False
        self.centre = False
        self.hovered = False
        self.newest_first = True
        self.check_source = 0
        self.server = NotifyServer(self.on_notify, self.on_take_back)
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
        clear = header_button("edit-clear-all-symbolic", "Clear all", self.clear_all)
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
        self.server.start()
        self.refresh()

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
        GLib.idle_add(self.scroll_to_newest)
        ring(count, self.dnd)
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

    def on_notify(self, notice: Notice) -> int:
        notice.id = self.replaced_id(notice) or self.new_id()
        row = self.rows.get(notice.id)
        if row is None:
            row = NoticeRow(notice, self.delete, self.press)
            self.rows[notice.id] = row
            self.place(row)
        else:
            self.deadlines.pop(notice.id, None)
            row.show_notice(notice)
        if notice.critical or not self.dnd:
            self.toasts.add(notice.id)
            self.set_deadline(notice)
            chime(notice)
        else:
            self.toasts.discard(notice.id)
        self.refresh()
        return notice.id

    def new_id(self) -> int:
        self.last_id += 1
        return self.last_id

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
        """It stops being a toast and stays in the centre, unless its app asked not to keep it."""
        self.deadlines.pop(notice_id, None)
        self.toasts.discard(notice_id)
        if self.rows[notice_id].notice.transient:
            self.close(notice_id, EXPIRED)
        self.refresh()

    def delete(self, notice_id):
        self.close(notice_id, DISMISSED)
        self.refresh()

    def press(self, notice_id, action_key):
        self.server.invoked(notice_id, action_key)
        if not self.rows[notice_id].notice.resident:
            self.close(notice_id, DISMISSED)
        self.refresh()

    def on_take_back(self, notice_id):
        if notice_id in self.rows:
            self.close(notice_id, CLOSED_BY_APP)
            self.refresh()

    def clear_all(self):
        for notice_id in list(self.rows):
            self.close(notice_id, DISMISSED)
        self.refresh()

    def close(self, notice_id, reason):
        row = self.rows.pop(notice_id)
        self.toasts.discard(notice_id)
        self.deadlines.pop(notice_id, None)
        self.list.remove(row)
        self.server.closed(notice_id, reason)

    # ---- Do Not Disturb

    def toggle_dnd(self):
        self.dnd = not self.dnd
        if self.dnd:
            self.dnd_button.add_css_class("lit")
        else:
            self.dnd_button.remove_css_class("lit")
        self.refresh()
