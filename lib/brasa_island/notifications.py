"""The island's notifications section, and the notification server behind it. Closed, the island shows only
the ones that just arrived (toasts), each leaving on its own or waiting for an answer. Open, it is the
notification centre: every notification kept, newest first, with Do Not Disturb, Clear all and a button
that closes the island. One its app takes back (a chat read in the browser) stays, folded away under
"Closed by their app". The centre is kept on disk across restarts."""
import time

from gi.repository import Gio, GLib, Gtk

from . import notice_store
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
MAX_KEPT = 50
FOLD_SLIDE_MS = 200


def seconds_shown(notice: Notice):
    """How long it stays as a toast; None: until it is answered or closed."""
    if notice.expire_ms > 0:
        return notice.expire_ms / 1000
    if notice.critical or notice.buttons:
        return None
    return LOW_SECONDS if notice.urgency == LOW else NORMAL_SECONDS


def open_app(desktop_entry: str):
    try:
        app = Gio.DesktopAppInfo.new(f"{desktop_entry.removesuffix('.desktop')}.desktop")
    except TypeError:
        app = None
    if app is not None:
        app.launch([], None)


def header_button(icon, tooltip, on_click):
    button = Gtk.Button(icon_name=icon, tooltip_text=tooltip, css_classes=["island-button", "island-header-button"])
    button.connect("clicked", lambda _button: on_click())
    return button


class Notifications:
    """on_change() whenever what it shows changes; on_close_island() for the header's close button."""

    def __init__(self, on_change, on_close_island):
        self.on_change = on_change
        self.rows = {}
        self.closed_by_app = set()
        self.toasts = set()
        self.deadlines = {}
        self.last_id = 0
        self.dnd = False
        self.centre = False
        self.hovered = False
        self.newest_first = True
        self.check_source = 0
        self.save_source = 0
        self.server = NotifyServer(self.on_notify, self.on_take_back)
        self.widget = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.header = self.build_header(on_close_island)
        self.list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.lists = self.build_lists()
        self.scroller = Gtk.ScrolledWindow(child=self.lists, hscrollbar_policy=Gtk.PolicyType.NEVER,
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

    def build_lists(self):
        """The live notifications, then the fold with the ones their apps took back."""
        self.fold_label = Gtk.Label(xalign=0, hexpand=True)
        self.fold_arrow = Gtk.Image(icon_name="pan-end-symbolic")
        fold_face = Gtk.Box(spacing=6)
        fold_face.append(self.fold_label)
        fold_face.append(self.fold_arrow)
        self.fold = Gtk.Button(child=fold_face, css_classes=["island-fold"])
        self.fold.connect("clicked", lambda _button: self.toggle_fold())
        self.earlier_list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.earlier = Gtk.Revealer(child=self.earlier_list, transition_duration=FOLD_SLIDE_MS)
        self.earlier.connect("notify::child-revealed", lambda *_: self.on_change())
        lists = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        for child in (self.list, self.fold, self.earlier):
            lists.append(child)
        return lists

    def toggle_fold(self):
        unfolded = not self.earlier.get_reveal_child()
        self.earlier.set_reveal_child(unfolded)
        self.fold_arrow.set_from_icon_name("pan-down-symbolic" if unfolded else "pan-end-symbolic")
        self.on_change()

    # ---- What it shows

    @property
    def has_toasts(self) -> bool:
        return bool(self.toasts)

    def start(self):
        self.restore()
        self.server.start()
        self.refresh()

    def restore(self):
        """The centre as it was. Their apps no longer know these notifications, so they are stale."""
        saved = notice_store.load()
        for notice in saved.notices:
            row = NoticeRow(notice, self.delete, self.press)
            row.set_stale(True)
            self.rows[notice.id] = row
            if notice.id in saved.closed_by_app:
                self.closed_by_app.add(notice.id)
            self.place(row)
        self.last_id = max(self.rows, default=0)
        if saved.dnd:
            self.toggle_dnd()

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
        lists = [self.list, self.fold, self.earlier] if at_top else [self.earlier, self.fold, self.list]
        for before, child in zip([None, *lists], lists):
            self.lists.reorder_child_after(child, before)
        if self.newest_first == at_top:
            return
        self.newest_first = at_top
        rows = list(self.rows.values())
        for row in rows:
            row.get_parent().remove(row)
        for row in rows:
            self.place(row)

    def refresh(self):
        live = len(self.rows) - len(self.closed_by_app)
        self.title.set_text(f"Notifications · {live}" if live else "Notifications")
        self.header.set_visible(self.centre)
        self.nothing.set_visible(self.centre and not self.rows)
        self.scroller.set_visible(bool(self.rows) if self.centre else bool(self.toasts))
        self.fold.set_visible(self.centre and bool(self.closed_by_app))
        self.fold_label.set_text(f"Closed by their app · {len(self.closed_by_app)}")
        self.earlier.set_visible(self.centre)
        for notice_id, row in self.rows.items():
            row.set_visible(self.centre or notice_id in self.toasts)
        GLib.idle_add(self.scroll_to_newest)
        ring(live, self.dnd)
        self.save_soon()
        self.on_change()

    def place(self, row):
        """In the live list or under the fold, newest nearest the edge."""
        target = self.earlier_list if row.notice.id in self.closed_by_app else self.list
        if self.newest_first:
            target.prepend(row)
        else:
            target.append(row)

    def save_soon(self):
        if not self.save_source:
            self.save_source = GLib.idle_add(self.save)

    def save(self):
        self.save_source = 0
        notices = [row.notice for row in self.rows.values()]
        try:
            notice_store.save(notices, self.closed_by_app, self.dnd)
        except OSError:
            pass
        return GLib.SOURCE_REMOVE

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
            self.bring_back(row)
        if notice.critical or not self.dnd:
            self.toasts.add(notice.id)
            self.set_deadline(notice)
            chime(notice)
        else:
            self.toasts.discard(notice.id)
        self.keep_at_most(MAX_KEPT)
        self.refresh()
        return notice.id

    def bring_back(self, row):
        """An app sent again a notification it had taken back: it is live again."""
        row.set_stale(False)
        if row.notice.id not in self.closed_by_app:
            return
        self.closed_by_app.discard(row.notice.id)
        row.get_parent().remove(row)
        self.place(row)

    def keep_at_most(self, count):
        """The oldest of those their apps took back go first, then the oldest of all."""
        while len(self.rows) > count:
            taken_back = [notice_id for notice_id in self.rows if notice_id in self.closed_by_app]
            oldest = taken_back[0] if taken_back else next(iter(self.rows))
            self.close(oldest, DISMISSED)

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
        """A stale one's app no longer answers: the click opens the app instead."""
        row = self.rows[notice_id]
        if row.stale:
            open_app(row.notice.desktop_entry)
            self.close(notice_id, DISMISSED)
        else:
            self.server.invoked(notice_id, action_key)
            if not row.notice.resident:
                self.close(notice_id, DISMISSED)
        self.refresh()

    def on_take_back(self, notice_id):
        """Its app closed it: it leaves the live list and the toasts, and waits under the fold."""
        if notice_id not in self.rows or notice_id in self.closed_by_app:
            return
        row = self.rows[notice_id]
        self.toasts.discard(notice_id)
        self.deadlines.pop(notice_id, None)
        row.get_parent().remove(row)
        self.closed_by_app.add(notice_id)
        row.set_stale(True)
        self.place(row)
        self.server.closed(notice_id, CLOSED_BY_APP)
        self.refresh()

    def clear_all(self):
        for notice_id in list(self.rows):
            self.close(notice_id, DISMISSED)
        self.refresh()

    def close(self, notice_id, reason):
        """Deletes it; its app hears of it, unless it closed it itself."""
        row = self.rows.pop(notice_id)
        self.toasts.discard(notice_id)
        self.deadlines.pop(notice_id, None)
        row.get_parent().remove(row)
        if notice_id in self.closed_by_app:
            self.closed_by_app.discard(notice_id)
        else:
            self.server.closed(notice_id, reason)

    # ---- Do Not Disturb

    def toggle_dnd(self):
        self.dnd = not self.dnd
        if self.dnd:
            self.dnd_button.add_css_class("lit")
        else:
            self.dnd_button.remove_css_class("lit")
        self.refresh()
