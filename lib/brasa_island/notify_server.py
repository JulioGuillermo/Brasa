"""The notification server: owns org.freedesktop.Notifications on the session bus, so every app's
notifications come to the island. It only speaks D-Bus; Notifications keeps them."""
from gi.repository import Gio, GLib

from .notice import Notice, from_notify

NAME = "org.freedesktop.Notifications"
PATH = "/org/freedesktop/Notifications"
CAPABILITIES = ["actions", "body", "body-markup", "icon-static", "persistence", "x-canonical-private-synchronous"]
SERVER_INFORMATION = ("brasa-island", "Brasa", "1.0", "1.2")
EXPIRED, DISMISSED, CLOSED_BY_APP = 1, 2, 3
INTERFACE = Gio.DBusNodeInfo.new_for_xml("""
<node>
  <interface name="org.freedesktop.Notifications">
    <method name="Notify">
      <arg direction="in" type="s"/><arg direction="in" type="u"/><arg direction="in" type="s"/>
      <arg direction="in" type="s"/><arg direction="in" type="s"/><arg direction="in" type="as"/>
      <arg direction="in" type="a{sv}"/><arg direction="in" type="i"/>
      <arg direction="out" type="u"/>
    </method>
    <method name="CloseNotification"><arg direction="in" type="u"/></method>
    <method name="GetCapabilities"><arg direction="out" type="as"/></method>
    <method name="GetServerInformation">
      <arg direction="out" type="s"/><arg direction="out" type="s"/>
      <arg direction="out" type="s"/><arg direction="out" type="s"/>
    </method>
    <signal name="NotificationClosed"><arg type="u"/><arg type="u"/></signal>
    <signal name="ActionInvoked"><arg type="u"/><arg type="s"/></signal>
  </interface>
</node>""").interfaces[0]


class NotifyServer:
    """on_notify(notice) -> the id it gets; on_take_back(id) when its app closes it."""

    def __init__(self, on_notify, on_take_back):
        self.on_notify = on_notify
        self.on_take_back = on_take_back
        self.bus = None
        self.registration = 0
        self.ownership = 0

    def start(self):
        self.bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        self.registration = self.bus.register_object(PATH, INTERFACE, self.on_call, None, None)
        self.ownership = Gio.bus_own_name_on_connection(self.bus, NAME, Gio.BusNameOwnerFlags.REPLACE, None, None)

    def stop(self):
        if self.ownership:
            Gio.bus_unown_name(self.ownership)
            self.ownership = 0
        if self.registration:
            self.bus.unregister_object(self.registration)
            self.registration = 0

    def closed(self, notice_id: int, reason: int):
        self.emit("NotificationClosed", GLib.Variant("(uu)", (notice_id, reason)))

    def invoked(self, notice_id: int, action_key: str):
        self.emit("ActionInvoked", GLib.Variant("(us)", (notice_id, action_key)))

    def emit(self, signal, values):
        if self.bus is not None:
            self.bus.emit_signal(None, PATH, NAME, signal, values)

    def on_call(self, _bus, _sender, _path, _interface, method, arguments, invocation):
        if method == "Notify":
            notice: Notice = from_notify(arguments)
            invocation.return_value(GLib.Variant("(u)", (self.on_notify(notice),)))
        elif method == "CloseNotification":
            self.on_take_back(arguments.get_child_value(0).get_uint32())
            invocation.return_value(None)
        elif method == "GetCapabilities":
            invocation.return_value(GLib.Variant("(as)", (CAPABILITIES,)))
        elif method == "GetServerInformation":
            invocation.return_value(GLib.Variant("(ssss)", SERVER_INFORMATION))
