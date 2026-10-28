"""One tile: a rounded button with its icon, the name below. Lit while what it switches is on; a menu
tile carries a chevron."""
from gi.repository import Gtk, Pango

from . import shell
from .items import Action, Menu, Switch

ICON_SIZE = 20


def icon_widget(icon: str) -> Gtk.Widget:
    """A themed icon, or a text glyph (Nerd Font) for panel.conf's own tiles."""
    if icon.endswith("-symbolic"):
        return Gtk.Image(icon_name=icon, pixel_size=ICON_SIZE)
    return Gtk.Label(label=icon, css_classes=["quick-glyph"])


class Tile(Gtk.Box):
    """on_action(action) and on_menu(menu) are the panel's: closing it, or showing the menu."""

    def __init__(self, item, on_action, on_menu):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=6, css_classes=["quick-tile"])
        self.item = item
        self.lit = False
        face = Gtk.Box(spacing=6, halign=Gtk.Align.CENTER)
        face.append(icon_widget(item.icon))
        if isinstance(item, Menu):
            face.append(Gtk.Image(icon_name="pan-end-symbolic", pixel_size=12, css_classes=["quick-chevron"]))
        self.button = Gtk.Button(child=face, css_classes=["quick-tile-button"], tooltip_text=item.name)
        self.button.connect("clicked", lambda _button: self.clicked(on_action, on_menu))
        name = Gtk.Label(label=item.name, ellipsize=Pango.EllipsizeMode.END, max_width_chars=10,
                         css_classes=["quick-tile-name"])
        self.append(self.button)
        self.append(name)

    def clicked(self, on_action, on_menu):
        if isinstance(self.item, Switch):
            shell.run(self.item.off if self.lit else self.item.on)
            self.show_lit(not self.lit)
        elif isinstance(self.item, Action):
            on_action(self.item)
        else:
            on_menu(self.item)

    def refresh(self):
        if isinstance(self.item, Switch):
            shell.test(self.item.state, self.show_lit)

    def show_lit(self, lit: bool):
        self.lit = lit
        if lit:
            self.button.add_css_class("lit")
        else:
            self.button.remove_css_class("lit")
