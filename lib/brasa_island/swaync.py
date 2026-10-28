"""Talks to swaync, the notification server, which keeps every notification for the panel. While the
island shows notifications, swaync's own popups still exist but are invisible (gen-waybar-swaync.py styles
them so): swaync only lets another program press a button of its newest popup, so the island keeps a copy
of swaync's popup stack in step with it."""
from gi.repository import Gio, GLib

NAME = "org.erikreider.swaync.cc"
PATH = "/org/erikreider/swaync/cc"


class Swaync:
    """on_state(quiet: bool, panel_open: bool) when Do Not Disturb, an inhibitor or the panel changes."""

    def __init__(self, on_state):
        self.on_state = on_state
        self.bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        self.popups = []
        self.abandoned = set()
        self.dnd = False
        self.inhibited = False
        self.panel_open = False
        self.subscription = 0

    def start(self):
        self.subscription = self.bus.signal_subscribe(None, NAME, "SubscribeV2", PATH, None,
                                                      Gio.DBusSignalFlags.NONE, self.on_subscribe)
        self.dnd = self.ask("GetDnd", False)
        self.inhibited = self.ask("IsInhibited", False)
        self.panel_open = self.ask("GetVisibility", False)
        self.hide_all()

    def stop(self):
        if self.subscription:
            self.bus.signal_unsubscribe(self.subscription)
            self.subscription = 0
        self.hide_all()

    @property
    def quiet(self) -> bool:
        return self.dnd or self.inhibited

    def floats(self, critical: bool) -> bool:
        """Whether swaync makes a popup for a new notification right now."""
        return not self.panel_open and (critical or not self.quiet)

    # ---- The popup stack, newest last

    def added(self, notice_id: int, replaces_id: int):
        if replaces_id in self.popups:
            self.popups[self.popups.index(replaces_id)] = notice_id
            return
        self.popups.append(notice_id)

    def gone(self, notice_id: int):
        """swaync closed it (an action, the app, the panel)."""
        if notice_id in self.popups:
            self.popups.remove(notice_id)
        self.abandoned.discard(notice_id)
        self.hide_abandoned_on_top()

    def dismiss(self, notice_id: int):
        """The island stopped showing it. swaync can only hide its newest popup, so one further down
        waits until the ones above it go."""
        if notice_id in self.popups:
            self.abandoned.add(notice_id)
        self.hide_abandoned_on_top()

    def hide_abandoned_on_top(self):
        while self.popups and self.popups[-1] in self.abandoned:
            self.abandoned.discard(self.popups.pop())
            self.call("HideLatestNotifications", GLib.Variant("(b)", (False,)))

    def can_press(self, notice_id: int) -> bool:
        return notice_id in self.popups

    def press(self, notice_id: int, button_index: int) -> list[int]:
        """Presses a button of a popup, which swaync then closes. The popups above it are hidden first,
        so their buttons stop working; returns the ones of those the island still shows."""
        if notice_id not in self.popups:
            return []
        position = self.popups.index(notice_id)
        above = self.popups[position + 1:]
        del self.popups[position + 1:]
        for _ in above:
            self.call("HideLatestNotifications", GLib.Variant("(b)", (False,)))
        self.call("LatestInvokeAction", GLib.Variant("(u)", (button_index,)))
        self.popups.remove(notice_id)
        still_shown = [above_id for above_id in above if above_id not in self.abandoned]
        self.abandoned.difference_update(above)
        return still_shown

    def close(self, notice_id: int):
        """Deletes it in swaync too, which then announces it closed."""
        self.call("CloseNotification", GLib.Variant("(u)", (notice_id,)))

    def close_all(self):
        self.call("CloseAllNotifications", None)

    def set_dnd(self, on: bool):
        self.dnd = on
        self.call("SetDnd", GLib.Variant("(b)", (on,)))

    def hide_all(self):
        self.popups.clear()
        self.abandoned.clear()
        self.call("HideAllNotifications", None)

    # ---- D-Bus

    def on_subscribe(self, _bus, _sender, _path, _interface, _signal, values):
        _count, self.dnd, self.panel_open, self.inhibited = values.unpack()
        if self.panel_open:
            self.popups.clear()
            self.abandoned.clear()
        self.on_state(self.quiet, self.panel_open)

    def ask(self, method, default):
        try:
            return self.bus.call_sync(NAME, PATH, NAME, method, None, None, Gio.DBusCallFlags.NONE, 1000,
                                      None).unpack()[0]
        except GLib.Error:
            return default

    def call(self, method, arguments):
        self.bus.call(NAME, PATH, NAME, method, arguments, None, Gio.DBusCallFlags.NONE, 1000, None, None)
