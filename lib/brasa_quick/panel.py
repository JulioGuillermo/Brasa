"""The quick settings panel: it grows out of a bottom corner of the focused monitor, joined to the bottom
edge and to the side edge, like the island. Its window covers the monitor, so a click anywhere else
closes it; the outline is drawn in the corner and springs drive its size. Built again at every opening,
so it always shows this machine's tiles and panel.conf's."""
from gi.repository import Gdk, GLib, Gtk
from gi.repository import Gtk4LayerShell as LayerShell

from brasa_island import config as island_config
from brasa_island.clip import Clip
from brasa_island.focus import focused_connector
from brasa_island.spring import Spring

from . import catalog, glyphs, hardware, shell
from .corner import Corner, paint, to_corner_frame
from .keys import Keys
from .menu_view import MenuView
from .pager import Pager
from .sliders import sliders

PADDING = 22
RADIUS = 34
SHOULDER = 90
CONTENT_FADE_SECONDS = 0.15
MAX_FRAME_SECONDS = 1 / 30
EXIT = "~/.local/bin/brasa-exit"
ISLAND_SETTINGS = "~/.local/bin/brasa-island-settings"
BODY_DARKNESS = 0.5
HYPRMOD = "hyprmod"


def find_monitor(connector):
    for monitor in Gdk.Display.get_default().get_monitors():
        if monitor.get_connector() == connector:
            return monitor
    return None


def rgba(color):
    parsed = Gdk.RGBA()
    parsed.parse(color)
    return parsed


def darker(color: Gdk.RGBA) -> Gdk.RGBA:
    shade = Gdk.RGBA()
    shade.red, shade.green, shade.blue = (part * BODY_DARKNESS for part in (color.red, color.green, color.blue))
    shade.alpha = 1
    return shade


def round_button(glyph, tooltip, on_click):
    button = Gtk.Button(label=glyph, tooltip_text=tooltip, css_classes=["quick-round"])
    button.connect("clicked", lambda _button: on_click())
    return button


class Panel(Gtk.Window):
    def __init__(self, app):
        super().__init__(application=app, css_classes=["brasa-quick"])
        self.place_on_screen()
        self.is_open = False
        self.on_right = True
        self.color = rgba("#0c1838")
        self.menu_tile = 0
        self.ticking = False
        self.last_frame = None
        self.width, self.height, self.shoulder, self.radius = Spring(0), Spring(0), Spring(0), Spring(0)
        self.springs = (self.width, self.height, self.shoulder, self.radius)
        self.canvas = Gtk.DrawingArea(hexpand=True, vexpand=True)
        self.canvas.set_draw_func(self.draw)
        self.layers = Gtk.Overlay(child=self.canvas)
        self.layers.connect("get-child-position", self.place_content)
        self.clip = None
        self.set_child(self.layers)
        self.listen()

    def place_on_screen(self):
        LayerShell.init_for_window(self)
        LayerShell.set_namespace(self, "brasa-quick")
        LayerShell.set_layer(self, LayerShell.Layer.OVERLAY)
        LayerShell.set_exclusive_zone(self, -1)
        for edge in (LayerShell.Edge.TOP, LayerShell.Edge.BOTTOM, LayerShell.Edge.LEFT, LayerShell.Edge.RIGHT):
            LayerShell.set_anchor(self, edge, True)

    def listen(self):
        outside = Gtk.GestureClick()
        outside.connect("pressed", self.on_press)
        self.add_controller(outside)
        self.keys = Keys(self)
        typing = Gtk.EventControllerKey(propagation_phase=Gtk.PropagationPhase.CAPTURE)
        typing.connect("key-pressed", lambda _controller, keyval, _code, _state: self.keys.press(keyval))
        self.add_controller(typing)

    # ---- Opening and closing

    def toggle(self):
        if self.is_open:
            self.close_panel()
        else:
            self.open_panel()

    def open_panel(self):
        config = island_config.load()
        self.on_right = config.quick_side == "right"
        self.color = darker(rgba(config.color))
        self.size = config.quick_size
        self.show_card(self.build())
        monitor = find_monitor(focused_connector())
        if monitor is not None:
            LayerShell.set_monitor(self, monitor)
        LayerShell.set_keyboard_mode(self, LayerShell.KeyboardMode.EXCLUSIVE)
        self.is_open = True
        self.present()
        self.pager.refresh()
        for slider in self.sliders:
            slider.refresh()
        self.pager.focus_tile(0)
        self.fit()

    def close_panel(self):
        self.is_open = False
        LayerShell.set_keyboard_mode(self, LayerShell.KeyboardMode.NONE)
        self.fit()

    def back_or_close(self):
        if self.showing_menu:
            self.show_tiles()
        else:
            self.close_panel()

    def on_press(self, _gesture, _presses, x, y):
        if not self.corner().holds(*self.to_corner(x, y)):
            self.close_panel()

    # ---- The card

    def show_card(self, card):
        if self.clip is not None:
            self.layers.remove_overlay(self.clip)
        card.set_opacity(0)
        self.card = card
        self.clip = Clip(card)
        self.layers.add_overlay(self.clip)

    def build(self):
        self.pager = Pager(catalog.tiles(), self.size, self.run_action, self.show_menu)
        self.menu = MenuView(self.show_tiles, self.chosen)
        self.views = Gtk.Stack(vhomogeneous=False, hhomogeneous=True)
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
        footer.append(Gtk.Label(label=glyphs.BATTERY, visible=hardware.has_battery(), css_classes=["quick-battery"]))
        footer.append(Gtk.Label(label=hardware.battery_percent(), xalign=0, hexpand=True))
        self.footer_buttons = [round_button(glyphs.SETTINGS, "Panel and island settings: size, corner…",
                                            lambda: self.close_and_run(ISLAND_SETTINGS))]
        if hardware.has_hyprmod() and hardware.in_hyprland():
            self.footer_buttons.append(round_button(glyphs.HYPRLAND, "Hyprland settings",
                                                    lambda: self.close_and_run(HYPRMOD)))
        self.footer_buttons.append(round_button(glyphs.POWER_OFF, "Log out, lock, power off…",
                                                lambda: self.close_and_run(EXIT)))
        for button in self.footer_buttons:
            footer.append(button)
        return footer

    @property
    def showing_menu(self) -> bool:
        return self.views.get_visible_child_name() == "menu"

    def run_action(self, action):
        self.close_and_run(action.command)

    def close_and_run(self, command):
        self.close_panel()
        shell.run(command)

    def show_menu(self, menu):
        self.menu_tile = self.tile_of(menu)
        self.menu.show_menu(menu)
        self.views.set_visible_child_name("menu")
        self.menu.focus_first()
        self.fit()

    def tile_of(self, menu) -> int:
        return next(index for index, tile in enumerate(self.pager.tiles) if tile.item is menu)

    def show_tiles(self):
        self.views.set_visible_child_name("tiles")
        self.pager.focus_tile(self.menu_tile)
        self.fit()

    def chosen(self, _choice):
        self.show_tiles()

    # ---- Shape and animation

    def card_size(self) -> tuple[int, int]:
        width = self.card.measure(Gtk.Orientation.HORIZONTAL, -1)[1]
        height = self.card.measure(Gtk.Orientation.VERTICAL, width)[1]
        return width, height

    def corner(self) -> Corner:
        return Corner(self.width.value, self.height.value, self.shoulder.value, self.radius.value)

    def to_corner(self, x, y) -> tuple[float, float]:
        frame_x = self.canvas.get_width() - x if self.on_right else x
        return frame_x, self.canvas.get_height() - y

    def place_content(self, _overlay, _clip, place):
        width, height = self.card_size()
        screen_width, screen_height = self.canvas.get_width(), self.canvas.get_height()
        place.x = screen_width - PADDING - width if self.on_right else PADDING
        place.y = screen_height - PADDING - height
        place.width, place.height = width, height
        body_left = screen_width - self.width.value if self.on_right else 0
        body_top = screen_height - self.height.value
        self.clip.show_only(body_left - place.x, body_top - place.y, self.width.value, self.height.value)
        return True

    def draw(self, _area, cr, width, height):
        to_corner_frame(cr, width, height, self.on_right)
        paint(cr, self.corner(), self.color)

    def retarget(self):
        if self.is_open:
            width, height = self.card_size()
            targets = (width + 2 * PADDING, height + 2 * PADDING, SHOULDER, RADIUS)
        else:
            targets = (0, 0, 0, 0)
        for spring, target in zip(self.springs, targets):
            spring.target = target

    def fit(self):
        if not self.ticking:
            self.ticking = True
            self.last_frame = None
            self.canvas.add_tick_callback(self.on_frame)

    def on_frame(self, _widget, frame_clock):
        now = frame_clock.get_frame_time() / 1e6
        seconds = min(now - self.last_frame, MAX_FRAME_SECONDS) if self.last_frame else 0.0
        self.last_frame = now
        self.retarget()
        for spring in self.springs:
            spring.step(seconds)
        fading = self.fade_card(seconds)
        self.layers.queue_allocate()
        self.canvas.queue_draw()
        if fading or not all(spring.at_rest() for spring in self.springs):
            return GLib.SOURCE_CONTINUE
        self.ticking = False
        if not self.is_open:
            self.set_visible(False)
        return GLib.SOURCE_REMOVE

    def fade_card(self, seconds) -> bool:
        target = 1.0 if self.is_open else 0.0
        step = seconds / CONTENT_FADE_SECONDS
        opacity = self.card.get_opacity()
        opacity = min(opacity + step, target) if target > opacity else max(opacity - step, target)
        self.card.set_opacity(opacity)
        return opacity != target
