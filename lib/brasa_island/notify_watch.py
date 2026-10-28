"""Watches the notifications the notification server (swaync) receives, without being it: a monitor
connection to the session bus sees every Notify call, the id the server answers with, and the
NotificationClosed / ActionInvoked signals."""
from gi.repository import Gio, GLib

from .notice import from_notify

SERVER = "org.freedesktop.Notifications"
RULES = [
    f"type='method_call',interface='{SERVER}',member='Notify'",
    f"type='method_return',sender='{SERVER}'",
    f"type='signal',interface='{SERVER}'",
]
FINISHED_SIGNALS = ("NotificationClosed", "ActionInvoked")


class NotifyWatch:
    """on_notice(notice) for each notification the server accepted; on_finished(id) when the server
    closed one or an action of it was used."""

    def __init__(self, on_notice, on_finished):
        self.on_notice = on_notice
        self.on_finished = on_finished
        self.pending_calls = {}
        self.connection = None

    def start(self):
        address = Gio.dbus_address_get_for_bus_sync(Gio.BusType.SESSION, None)
        flags = Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT | Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION
        self.connection = Gio.DBusConnection.new_for_address_sync(address, flags, None, None)
        self.connection.add_filter(self.on_message)
        self.connection.call_sync("org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus.Monitoring",
                                  "BecomeMonitor", GLib.Variant("(asu)", (RULES, 0)), None,
                                  Gio.DBusCallFlags.NONE, -1, None)

    def stop(self):
        if self.connection is not None:
            self.connection.close_sync(None)
            self.connection = None
        self.pending_calls.clear()

    def on_message(self, _connection, message, incoming):
        """Runs in GDBus's worker thread. Of what comes in, only the bus's own answers go on to GDBus:
        a monitor must never answer the calls it sees."""
        if not incoming or message.get_sender() == "org.freedesktop.DBus":
            return message
        GLib.idle_add(self.handle, message)
        return None

    def handle(self, message):
        kind = message.get_message_type()
        if kind == Gio.DBusMessageType.METHOD_CALL:
            self.pending_calls[(message.get_sender(), message.get_serial())] = message.get_body()
        elif kind == Gio.DBusMessageType.METHOD_RETURN:
            self.answered(message)
        elif kind == Gio.DBusMessageType.SIGNAL and message.get_member() in FINISHED_SIGNALS:
            self.on_finished(message.get_body().get_child_value(0).get_uint32())
        return GLib.SOURCE_REMOVE

    def answered(self, message):
        call = self.pending_calls.pop((message.get_destination(), message.get_reply_serial()), None)
        body = message.get_body()
        if call is None or body is None or body.get_type_string() != "(u)":
            return
        self.on_notice(from_notify(call, body.get_child_value(0).get_uint32()))
