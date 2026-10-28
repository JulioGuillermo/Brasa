"""The quick settings panel: a card on the left or right edge of the focused monitor, over a transparent
window that covers the screen, so a click anywhere else closes it. Built again at every opening, so it
always shows this machine's tiles and panel.conf's."""
from gi.repository import Gdk, Gtk
from gi.repository import Gtk4LayerShell as LayerShell

from brasa_island import config as island_config
from brasa_island.focus import focused_connector

from . import catalog, hardware, shell
from .menu_view import MenuView
from .pager import Pager
from .sliders import sliders

SLIDE_MS = 200
EXIT = "~/.local/bin/brasa-exit"
HYPRMOD = "hyprmod"


def find_monitor(connector):
    for monitor in Gdk.Display.get_default().get_monitors():
        if monitor.get_connector() == connector:
            return monitor
    return None


def round_button(icon, tooltip, on_click):
    button = Gtk.Button(icon_name=icon, tooltip_text=tooltip, css_classes=["quick-round"])
    button.connect("clicked", lambda _button: on_click())
    return button


class Panel(Gtk.Window):
    def __init__(self, app):
        super().__init__(application=app, css_classes=["brasa-quick"])
        LayerShell.init_for_window(self)
        LayerShell.set_namespace(self, "brasa-quick")
        LayerShell.set_layer(self, LayerShell.Layer.OVERLAY)
        for edge in (LayerShell.Edge.TOP, LayerShell.Edge.BOTTOM, LayerShell.Edge.LEFT, LayerShell.Edge.RIGHT):
            LayerShell.set_anchor(self, edge, True)
        self.revealer = Gtk.Revealer(transition_duration=SLIDE_MS, valign=Gtk.Align.START)
        self.revealer.connect("notify::child-revealed", self.on_revealed)
        self.set_child(self.revealer)
        outside = Gtk.GestureClick()
        outside.connect("released", self.on_click)
        self.add_controller(outside)
        escape = Gtk.ShortcutController(scope=Gtk.ShortcutScope.GLOBAL)
        escape.add_shortcut(Gtk.Shortcut.new(Gtk.ShortcutTrigger.parse_string("Escape"),
                                             Gtk.CallbackAction.new(lambda *_: self.close_panel())))
        self.add_controller(escape)

    @property
    def is_open(self) -> bool:
        return self.revealer.get_reveal_child()

    def toggle(self):
        if self.is_open:
            self.close_panel()
        else:
            self.open_panel()

    def open_panel(self):
        on_right = island_config.load().quick_side == "right"
        self.revealer.set_halign(Gtk.Align.END if on_right else Gtk.Align.START)
        slide = Gtk.RevealerTransitionType.SLIDE_LEFT if on_right else Gtk.RevealerTransitionType.SLIDE_RIGHT
        self.revealer.set_transition_type(slide)
        self.revealer.set_child(self.build())
        monitor = find_monitor(focused_connector())
        if monitor is not None:
            LayerShell.set_monitor(self, monitor)
        LayerShell.set_keyboard_mode(self, LayerShell.KeyboardMode.EXCLUSIVE)
        self.present()
        self.pager.refresh()
        for slider in self.sliders:
            slider.refresh()
        self.revealer.set_reveal_child(True)

    def close_panel(self):
        LayerShell.set_keyboard_mode(self, LayerShell.KeyboardMode.NONE)
        self.revealer.set_reveal_child(False)

    def on_revealed(self, *_):
        if not self.revealer.get_child_revealed() and not self.is_open:
            self.set_visible(False)

    def on_click(self, _gesture, _presses, x, y):
        picked = self.pick(x, y, Gtk.PickFlags.DEFAULT)
        card = self.revealer.get_child()
        if picked is None or not (picked == card or picked.is_ancestor(card)):
            self.close_panel()

    # ---- The card

    def build(self):
        self.pager = Pager(catalog.tiles(), self.run_action, self.show_menu)
        self.menu = MenuView(self.show_tiles, self.chosen)
        self.views = Gtk.Stack(transition_type=Gtk.StackTransitionType.SLIDE_LEFT_RIGHT,
                               transition_duration=SLIDE_MS, vhomogeneous=False, interpolate_size=True)
        self.views.add_named(self.pager, "tiles")
        self.views.add_named(self.menu, "menu")
        self.sliders = sliders(hardware.has_backlight())
        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14, css_classes=["quick-card"])
        card.append(self.views)
        for slider in self.sliders:
            card.append(slider)
        card.append(self.footer())
        return card

    def footer(self):
        footer = Gtk.Box(spacing=6, css_classes=["quick-footer"])
        footer.append(Gtk.Image(icon_name="battery-level-80-symbolic", visible=hardware.has_battery(),
                                 css_classes=["quick-battery"]))
        footer.append(Gtk.Label(label=hardware.battery_percent(), xalign=0, hexpand=True))
        if hardware.has_hyprmod() and hardware.in_hyprland():
            footer.append(round_button("emblem-system-symbolic", "Hyprland settings", lambda: self.close_and_run(HYPRMOD)))
        footer.append(round_button("system-shutdown-symbolic", "Log out, lock, power off…",
                                   lambda: self.close_and_run(EXIT)))
        return footer

    # ---- Clicks

    def run_action(self, action):
        self.close_and_run(action.command)

    def close_and_run(self, command):
        self.close_panel()
        shell.run(command)

    def show_menu(self, menu):
        self.menu.show_menu(menu)
        self.views.set_visible_child_name("menu")

    def show_tiles(self):
        self.views.set_visible_child_name("tiles")

    def chosen(self, _choice):
        self.show_tiles()
