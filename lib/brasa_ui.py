"""What makes a GTK window one of Brasa's: the panel's look and, under Hyprland or sway, no minimize,
maximize or close buttons (Escape closes it)."""
import os

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gdk, Gtk

STYLE = os.path.join(os.path.dirname(os.path.realpath(__file__)), "brasa.css")


def in_brasa():
    return bool(os.environ.get("HYPRLAND_INSTANCE_SIGNATURE") or os.environ.get("SWAYSOCK"))


def load_style():
    """Call once the application has started (the display must exist)."""
    Adw.StyleManager.get_default().set_color_scheme(Adw.ColorScheme.FORCE_DARK)
    provider = Gtk.CssProvider()
    provider.load_from_path(STYLE)
    Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), provider,
                                              Gtk.STYLE_PROVIDER_PRIORITY_USER + 1)


def header(title_widget=None):
    bar = Adw.HeaderBar(title_widget=title_widget)
    if in_brasa():
        bar.set_show_start_title_buttons(False)
        bar.set_show_end_title_buttons(False)
    return bar


def make_brasa(window):
    """Brasa's class on the window, and Escape to close it."""
    window.add_css_class("brasa")
    shortcuts = Gtk.ShortcutController(scope=Gtk.ShortcutScope.GLOBAL)
    shortcuts.add_shortcut(Gtk.Shortcut.new(Gtk.ShortcutTrigger.parse_string("Escape"),
                                            Gtk.CallbackAction.new(lambda *_: window.close())))
    window.add_controller(shortcuts)
