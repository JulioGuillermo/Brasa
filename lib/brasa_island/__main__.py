"""brasa-island [toggle|centre|dnd|launcher|peek|hide|reload]: starts the islands, or tells the running ones what to do.
One island for the music and notifications and one for the app launcher, or a single one when both
sit in the same place."""
import os
import sys
import time

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, Gio, Gtk

from . import config
from .island import Island
from .launcher import Launcher
from .media import Media
from .notifications import Notifications

DOUBLE_TOGGLE_SECONDS = 0.6
STYLE = os.path.join(os.path.dirname(os.path.realpath(__file__)), "island.css")


def app_id():
    session = os.environ.get("WAYLAND_DISPLAY", "wayland-0").replace("-", "_")
    return f"dev.brasa.island.{session}"


class IslandApp(Gtk.Application):
    def __init__(self):
        super().__init__(application_id=app_id(), flags=Gio.ApplicationFlags.HANDLES_COMMAND_LINE)
        self.started = False
        self.main_island = None
        self.launcher_island = None
        self.last_launcher_toggle = 0.0

    def start(self):
        self.hold()
        self.started = True
        provider = Gtk.CssProvider()
        provider.load_from_path(STYLE)
        Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), provider,
                                                  Gtk.STYLE_PROVIDER_PRIORITY_USER)
        self.media = Media(on_peek=lambda: self.tell_main("peek"), on_resize=lambda: self.tell_main("fit"))
        self.notifications = Notifications(on_change=lambda: self.tell_main("relayout"),
                                           on_close_island=lambda: self.tell_main("close_view"))
        self.launcher = Launcher(on_done=self.close_launcher)
        self.notifications.start()
        self.build(config.load())

    def build(self, settings: config.Config):
        self.launcher.set_style(settings.launcher_view, settings.launcher_size)
        main = settings.place
        launcher = settings.launcher_place if settings.launcher else None
        if main == launcher:
            shared = Island(self, main, settings, self.media, self.notifications, self.launcher)
            self.main_island = self.launcher_island = shared
        else:
            self.main_island = Island(self, main, settings, self.media, self.notifications)
            self.launcher_island = Island(self, launcher, settings, launcher=self.launcher) if launcher else None
        for island in self.islands():
            island.present()

    def islands(self):
        return [island for island in {self.main_island, self.launcher_island} if island is not None]

    def reload(self):
        for island in self.islands():
            island.close_all()
            island.release_sections()
            island.destroy()
        self.build(config.load())
        for island in self.islands():
            island.later("show", 150, island.show_off)

    def tell_main(self, method):
        if self.main_island is not None:
            getattr(self.main_island, method)()

    def toggle_launcher(self):
        """Super+d then releasing Super asks twice in a row: the second one is ignored."""
        now = time.monotonic()
        if now - self.last_launcher_toggle < DOUBLE_TOGGLE_SECONDS:
            return
        self.last_launcher_toggle = now
        if self.launcher_island is not None:
            self.launcher_island.toggle_launcher()

    def close_launcher(self):
        if self.launcher_island is not None:
            self.launcher_island.close_view()

    def hide(self):
        for island in self.islands():
            island.close_all()

    def do_command_line(self, command_line):
        if not self.started:
            self.start()
        command = (command_line.get_arguments()[1:] or ["start"])[0]
        actions = {
            "toggle": lambda: self.tell_main("toggle"),
            "centre": lambda: self.tell_main("toggle_centre"),
            "dnd": self.notifications.toggle_dnd,
            "launcher": self.toggle_launcher,
            "peek": lambda: self.tell_main("peek"),
            "hide": self.hide,
            "reload": self.reload,
        }
        if command in actions:
            actions[command]()
        return 0


sys.exit(IslandApp().run(sys.argv))
