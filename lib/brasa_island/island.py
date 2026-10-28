"""The island window: a layer-shell surface on one screen edge that grows from nothing to a one-line peek
or to the full view, with springs driving every size so the outline flows between states."""
import cairo
from gi.repository import Gdk, GLib, Gtk
from gi.repository import Gtk4LayerShell as LayerShell

from .config import Config
from .shape import Outline, paint
from .spring import Spring

HIDDEN, PEEK, FULL = "hidden", "peek", "full"
SIZES = {  # body width, body height, shoulder reach, corner radius
    HIDDEN: (150, 0, 0, 0),
    PEEK: (340, 44, 64, 22),
    FULL: (470, 200, 120, 38),
}
PADDING = {PEEK: (8, 18), FULL: (16, 24)}  # from the edge, from the sides
SLACK = 40
FULL_WIDTH, FULL_HEIGHT, FULL_SHOULDER, _ = SIZES[FULL]
WINDOW_WIDTH = FULL_WIDTH + 2 * FULL_SHOULDER + 2 * SLACK
WINDOW_HEIGHT = FULL_HEIGHT + SLACK
HOT_ZONE_HEIGHT = 3
HOVER_DWELL_MS = 120
LEAVE_DELAY_MS = 400
SHORTCUT_IDLE_MS = 5000
CONTENT_FADE_SECONDS = 0.12
CONTENT_SHOWS_AT = 0.7
MAX_FRAME_SECONDS = 1 / 30


def find_monitor(connector):
    for monitor in Gdk.Display.get_default().get_monitors():
        if monitor.get_connector() == connector:
            return monitor
    return None


def rgba(color):
    parsed = Gdk.RGBA()
    parsed.parse(color)
    return parsed


class Island(Gtk.Window):
    def __init__(self, app, config: Config, module):
        super().__init__(application=app, css_classes=["brasa-island"],
                         default_width=WINDOW_WIDTH, default_height=WINDOW_HEIGHT)
        self.config = config
        self.color = rgba(config.color)
        self.module = module
        self.state = HIDDEN
        self.pointer_inside = False
        self.timers = {}
        self.last_frame = None
        self.ticking = False
        width, height, shoulder, radius = SIZES[HIDDEN]
        self.width, self.height = Spring(width), Spring(height)
        self.shoulder, self.radius = Spring(shoulder), Spring(radius)
        self.springs = (self.width, self.height, self.shoulder, self.radius)
        self.views = {PEEK: module.peek, FULL: module.full}
        self.place_on_screen()
        self.set_child(self.build())
        hover = Gtk.EventControllerMotion()
        hover.connect("enter", self.on_enter)
        hover.connect("leave", self.on_leave)
        self.add_controller(hover)
        self.connect("realize", lambda _window: self.update_input_region())

    @property
    def at_top(self) -> bool:
        return self.config.edge == "top"

    def place_on_screen(self):
        LayerShell.init_for_window(self)
        LayerShell.set_namespace(self, "brasa-island")
        LayerShell.set_layer(self, LayerShell.Layer.OVERLAY)
        LayerShell.set_exclusive_zone(self, -1)
        LayerShell.set_keyboard_mode(self, LayerShell.KeyboardMode.NONE)
        self.anchor()

    def anchor(self):
        anchored = {
            LayerShell.Edge.TOP: self.at_top,
            LayerShell.Edge.BOTTOM: not self.at_top,
            LayerShell.Edge.LEFT: self.config.align == "left",
            LayerShell.Edge.RIGHT: self.config.align == "right",
        }
        for edge, on in anchored.items():
            LayerShell.set_anchor(self, edge, on)
        LayerShell.set_monitor(self, find_monitor(self.config.monitor) if self.config.monitor else None)

    def apply(self, config: Config):
        """Moves the island to the edge, side and monitor of a new config, then shows it there."""
        self.hide_island()
        for spring in self.springs:
            spring.value, spring.speed = spring.target, 0.0
        self.config = config
        self.color = rgba(config.color)
        self.set_visible(False)
        self.anchor()
        self.present()
        self.canvas.queue_draw()
        self.layers.queue_resize()
        self.later("show", 150, self.toggle)

    def build(self):
        self.canvas = Gtk.DrawingArea(content_width=WINDOW_WIDTH, content_height=WINDOW_HEIGHT)
        self.canvas.set_draw_func(self.draw)
        self.layers = layers = Gtk.Overlay(child=self.canvas)
        for state, view in self.views.items():
            view.set_opacity(0)
            view.set_can_target(False)
            layers.add_overlay(view)
        layers.connect("get-child-position", self.place_view)
        return layers

    def place_view(self, _overlay, view, place):
        state = PEEK if view is self.views[PEEK] else FULL
        from_edge, from_sides = PADDING[state]
        width = SIZES[state][0] - 2 * from_sides
        height = view.measure(Gtk.Orientation.VERTICAL, width)[1]
        place.x = round((WINDOW_WIDTH - width) / 2)
        place.y = from_edge if self.at_top else WINDOW_HEIGHT - from_edge - height
        place.width, place.height = width, height
        return True

    def outline(self) -> Outline:
        return Outline(WINDOW_WIDTH / 2, self.width.value, self.height.value, self.shoulder.value, self.radius.value)

    def draw(self, _area, cr, _width, _height):
        if not self.at_top:
            cr.translate(0, WINDOW_HEIGHT)
            cr.scale(1, -1)
        paint(cr, self.outline(), self.color)

    # ---- States

    def go(self, state):
        self.state = state
        self.module.set_shown(state != HIDDEN)
        self.fit()

    def fit(self):
        """Retargets the outline to the current state's size, which follows its content's height."""
        for spring, size in zip(self.springs, self.size_of(self.state)):
            spring.target = size
        if not self.ticking:
            self.ticking = True
            self.last_frame = None
            self.canvas.add_tick_callback(self.on_frame)

    def size_of(self, state):
        width, height, shoulder, radius = SIZES[state]
        if state == HIDDEN:
            return SIZES[state]
        from_edge, from_sides = PADDING[state]
        content = self.views[state].measure(Gtk.Orientation.VERTICAL, width - 2 * from_sides)[1]
        return width, min(height, content + 2 * from_edge), shoulder, radius

    def toggle(self):
        if self.state == FULL:
            self.hide_island()
            return
        self.open_full()
        if not self.pointer_inside:
            self.later("idle", SHORTCUT_IDLE_MS, self.hide_island)

    def open_full(self):
        self.cancel("hide")
        self.go(FULL)

    def peek(self):
        if self.state == FULL or self.config.peek_seconds <= 0:
            return
        self.go(PEEK)
        self.later("hide", int(self.config.peek_seconds * 1000), self.hide_island)

    def hide_island(self):
        for name in list(self.timers):
            self.cancel(name)
        self.go(HIDDEN)

    # ---- Pointer

    def on_enter(self, _controller, _x, _y):
        self.pointer_inside = True
        self.cancel("leave")
        self.cancel("idle")
        if self.state != FULL and self.config.hover:
            self.later("dwell", HOVER_DWELL_MS, self.open_full)

    def on_leave(self, _controller):
        self.pointer_inside = False
        self.cancel("dwell")
        if self.state != HIDDEN:
            self.later("leave", LEAVE_DELAY_MS, self.hide_island)

    def later(self, name, milliseconds, action):
        self.cancel(name)

        def run():
            del self.timers[name]
            action()
            return GLib.SOURCE_REMOVE

        self.timers[name] = GLib.timeout_add(milliseconds, run)

    def cancel(self, name):
        source = self.timers.pop(name, None)
        if source:
            GLib.source_remove(source)

    # ---- Animation

    def on_frame(self, _widget, frame_clock):
        now = frame_clock.get_frame_time() / 1e6
        seconds = min(now - self.last_frame, MAX_FRAME_SECONDS) if self.last_frame else 0.0
        self.last_frame = now
        for spring in self.springs:
            spring.step(seconds)
        fading = self.fade_views(seconds)
        self.canvas.queue_draw()
        self.update_input_region()
        if fading or not all(spring.at_rest() for spring in self.springs):
            return GLib.SOURCE_CONTINUE
        self.ticking = False
        return GLib.SOURCE_REMOVE

    def fade_views(self, seconds) -> bool:
        fading = False
        for state, view in self.views.items():
            grown = self.height.value >= CONTENT_SHOWS_AT * self.size_of(state)[1]
            target = 1.0 if self.state == state and grown else 0.0
            step = seconds / CONTENT_FADE_SECONDS
            opacity = view.get_opacity()
            opacity = min(opacity + step, target) if target > opacity else max(opacity - step, target)
            view.set_opacity(opacity)
            view.set_can_target(opacity > 0)
            fading = fading or opacity != target or (self.state == state and not grown)
        return fading

    def update_input_region(self):
        surface = self.get_surface()
        if surface is None:
            return
        outline = self.outline()
        if outline.height >= 1:
            area = (outline.left, 0, outline.full_width, outline.height)
        elif self.config.hover:
            area = (WINDOW_WIDTH / 2 - SIZES[PEEK][0] / 2, 0, SIZES[PEEK][0], HOT_ZONE_HEIGHT)
        else:
            area = (0, 0, 0, 0)
        x, y, width, height = (round(value) for value in area)
        if not self.at_top:
            y = WINDOW_HEIGHT - height
        surface.set_input_region(cairo.Region(cairo.RectangleInt(x, y, width, height)))
