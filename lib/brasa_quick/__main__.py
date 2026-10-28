"""brasa-quick [toggle|start]: starts the quick settings panel hidden, or opens and closes the running one."""
import os
import sys

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
gi.require_version("Gtk4LayerShell", "1.0")
from gi.repository import Gdk, Gio, Gtk

from .panel import Panel

STYLE = os.path.join(os.path.dirname(os.path.realpath(__file__)), "quick.css")


def app_id():
    session = os.environ.get("WAYLAND_DISPLAY", "wayland-0").replace("-", "_")
    return f"dev.brasa.quick.{session}"


class QuickApp(Gtk.Application):
    def __init__(self):
        super().__init__(application_id=app_id(), flags=Gio.ApplicationFlags.HANDLES_COMMAND_LINE)
        self.panel = None

    def start(self):
        self.hold()
        provider = Gtk.CssProvider()
        provider.load_from_path(STYLE)
        Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), provider,
                                                  Gtk.STYLE_PROVIDER_PRIORITY_USER)
        self.panel = Panel(self)

    def do_command_line(self, command_line):
        if self.panel is None:
            self.start()
        command = (command_line.get_arguments()[1:] or ["toggle"])[0]
        if command == "toggle":
            self.panel.toggle()
        return 0


sys.exit(QuickApp().run(sys.argv))
