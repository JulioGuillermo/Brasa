"""One tile: a square button with its icon and its name inside, centred. Lit (red) while what it switches
is on. A menu tile has a chevron in its corner and a red line along its bottom."""
from gi.repository import Gtk, Pango

from . import glyphs, shell
from .items import Action, Menu, Switch

NAME_CHARACTERS = 11


def icon_widget(icon: str) -> Gtk.Widget:
    """A Nerd Font glyph, or a themed icon name (…-symbolic) from panel.conf."""
    if icon.endswith("-symbolic"):
        return Gtk.Image(icon_name=icon, pixel_size=22, css_classes=["quick-tile-icon"])
    return Gtk.Label(label=icon, css_classes=["quick-tile-icon"])


class Tile(Gtk.Box):
    """on_action(action) and on_menu(menu) are the panel's: closing it, or showing the menu."""

    def __init__(self, item, on_action, on_menu):
        super().__init__(css_classes=["quick-tile"])
        self.item = item
        self.lit = False
        face = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6,
                       halign=Gtk.Align.CENTER, valign=Gtk.Align.CENTER)
        face.append(icon_widget(item.icon))
        face.append(Gtk.Label(label=item.name, ellipsize=Pango.EllipsizeMode.END, max_width_chars=NAME_CHARACTERS,
                              justify=Gtk.Justification.CENTER, css_classes=["quick-tile-name"]))
        layers = Gtk.Overlay(child=face)
        classes = ["quick-tile-button"]
        if isinstance(item, Menu):
            classes.append("menu")
            layers.add_overlay(Gtk.Label(label=glyphs.CHEVRON, halign=Gtk.Align.END, valign=Gtk.Align.START,
                                         css_classes=["quick-chevron"]))
        self.button = Gtk.Button(child=layers, css_classes=classes, hexpand=True,
                                 tooltip_text=f"{item.name} ›" if isinstance(item, Menu) else item.name)
        self.button.connect("clicked", lambda _button: self.clicked(on_action, on_menu))
        self.append(self.button)

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
