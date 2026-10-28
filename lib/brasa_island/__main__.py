"""brasa-island [toggle|peek|hide|reload]: starts the island, or tells the running one what to do."""
import os
import sys

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, Gio, Gtk

from . import config
from .island import Island
from .media import Media
from .notifications import Notifications

STYLE = os.path.join(os.path.dirname(os.path.realpath(__file__)), "island.css")


def app_id():
    session = os.environ.get("WAYLAND_DISPLAY", "wayland-0").replace("-", "_")
    return f"dev.brasa.island.{session}"


class IslandApp(Gtk.Application):
    def __init__(self):
        super().__init__(application_id=app_id(), flags=Gio.ApplicationFlags.HANDLES_COMMAND_LINE)
        self.island = None

    def start(self):
        self.hold()
        provider = Gtk.CssProvider()
        provider.load_from_path(STYLE)
        Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), provider,
                                                  Gtk.STYLE_PROVIDER_PRIORITY_USER)
        settings = config.load()
        media = Media(on_peek=lambda: self.tell_island("peek"), on_resize=lambda: self.tell_island("fit"))
        notifications = Notifications(on_change=lambda: self.tell_island("relayout"),
                                      on_close_island=lambda: self.tell_island("close_view"))
        self.island = Island(self, settings, media, notifications)
        self.island.present()
        self.switch_notifications(settings.notifications)

    def tell_island(self, method):
        if self.island is not None:
            getattr(self.island, method)()

    def switch_notifications(self, on: bool):
        if on:
            self.island.notifications.start()
        else:
            self.island.notifications.stop()

    def reload(self):
        settings = config.load()
        self.switch_notifications(settings.notifications)
        self.island.apply(settings)

    def do_command_line(self, command_line):
        if self.island is None:
            self.start()
        command = (command_line.get_arguments()[1:] or ["start"])[0]
        actions = {
            "toggle": self.island.toggle,
            "peek": self.island.peek,
            "hide": self.island.close_all,
            "reload": self.reload,
        }
        if command in actions:
            actions[command]()
        return 0


sys.exit(IslandApp().run(sys.argv))
