"""The island window: a layer-shell surface on one screen edge whose outline grows out of the edge to fit
what it shows. Nearest the edge the music (one line, or the full view), then the notifications. Springs
drive every size, so the outline flows between states; the content is clipped to the outline."""
import cairo
from gi.repository import Gdk, GLib, Gtk
from gi.repository import Gtk4LayerShell as LayerShell

from .clip import Clip
from .config import Config
from .shape import Outline, paint
from .spring import Spring

HIDDEN, LINE, CARD = "hidden", "line", "card"
SHAPES = {  # body width, max body height, shoulder reach, corner radius
    HIDDEN: (150, 0, 0, 0),
    LINE: (352, 44, 64, 22),
    CARD: (470, 620, 120, 38),
}
PADDING = {LINE: (8, 18), CARD: (16, 24)}  # from the edge, from the sides
SECTION_SPACING = 12
SLACK = 40
CARD_WIDTH, CARD_HEIGHT, CARD_SHOULDER, _ = SHAPES[CARD]
WINDOW_WIDTH = CARD_WIDTH + 2 * CARD_SHOULDER + 2 * SLACK
WINDOW_HEIGHT = CARD_HEIGHT + SLACK
HOT_ZONE_HEIGHT = 3
HOVER_DWELL_MS = 120
LEAVE_DELAY_MS = 400
SHORTCUT_IDLE_MS = 5000
CONTENT_FADE_SECONDS = 0.15
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
    def __init__(self, app, config: Config, media, notifications):
        super().__init__(application=app, css_classes=["brasa-island"],
                         default_width=WINDOW_WIDTH, default_height=WINDOW_HEIGHT)
        self.config = config
        self.color = rgba(config.color)
        self.media = media
        self.notifications = notifications
        self.shape = HIDDEN
        self.content_shape = LINE
        self.open = False
        self.peeking = False
        self.pointer_inside = False
        self.timers = {}
        self.last_frame = None
        self.ticking = False
        width, height, shoulder, radius = SHAPES[HIDDEN]
        self.width, self.height = Spring(width), Spring(height)
        self.shoulder, self.radius = Spring(shoulder), Spring(radius)
        self.inset = Spring(PADDING[LINE][0])
        self.springs = (self.width, self.height, self.shoulder, self.radius, self.inset)
        self.place_on_screen()
        self.set_child(self.build())
        self.order_sections()
        hover = Gtk.EventControllerMotion()
        hover.connect("enter", self.on_enter)
        hover.connect("leave", self.on_leave)
        self.add_controller(hover)
        self.connect("realize", lambda _window: self.update_input_region())
        self.relayout()

    @property
    def at_top(self) -> bool:
        return self.config.edge == "top"

    # ---- Window

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
        self.close_all()
        for spring in self.springs:
            spring.value, spring.speed = spring.target, 0.0
        self.config = config
        self.color = rgba(config.color)
        self.set_visible(False)
        self.anchor()
        self.order_sections()
        self.present()
        self.canvas.queue_draw()
        self.layers.queue_resize()
        self.later("show", 150, self.toggle)

    def build(self):
        self.canvas = Gtk.DrawingArea(content_width=WINDOW_WIDTH, content_height=WINDOW_HEIGHT)
        self.canvas.set_draw_func(self.draw)
        self.empty = Gtk.Label(label="Nothing to show here (Super+Shift+m)", css_classes=["island-time"])
        self.content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=SECTION_SPACING, opacity=0)
        for section in (self.media.widget, self.notifications.widget, self.empty):
            self.content.append(section)
        self.clip = Clip(self.content)
        self.layers = Gtk.Overlay(child=self.canvas)
        self.layers.add_overlay(self.clip)
        self.layers.connect("get-child-position", self.place_content)
        return self.layers

    def order_sections(self):
        """The music nearest the edge, then the notifications."""
        sections = [self.media.widget, self.notifications.widget, self.empty]
        if not self.at_top:
            sections.reverse()
        for before, section in zip([None, *sections], sections):
            self.content.reorder_child_after(section, before)
        self.notifications.set_edge(self.at_top)

    def content_width(self) -> int:
        return SHAPES[self.content_shape][0] - 2 * PADDING[self.content_shape][1]

    def content_height(self) -> int:
        return self.content.measure(Gtk.Orientation.VERTICAL, self.content_width())[1]

    def place_content(self, _overlay, _clip, place):
        width = self.content_width()
        height = self.content_height()
        place.x = round((WINDOW_WIDTH - width) / 2)
        place.y = round(self.inset.value if self.at_top else WINDOW_HEIGHT - self.inset.value - height)
        place.width, place.height = width, height
        body = self.outline()
        top = 0 if self.at_top else WINDOW_HEIGHT - body.height
        self.clip.show_only(body.center - body.width / 2 - place.x, top - place.y, body.width, body.height)
        return True

    def outline(self) -> Outline:
        return Outline(WINDOW_WIDTH / 2, self.width.value, self.height.value, self.shoulder.value, self.radius.value)

    def draw(self, _area, cr, _width, _height):
        if not self.at_top:
            cr.translate(0, WINDOW_HEIGHT)
            cr.scale(1, -1)
        paint(cr, self.outline(), self.color)

    # ---- What it shows

    def relayout(self):
        media_on = self.config.media
        centre_on = self.open and self.config.notification_centre
        notices_on = self.config.notifications and (centre_on or self.notifications.has_toasts)
        show_media = media_on and (self.open or self.peeking or (notices_on and self.media.playing()))
        show_empty = self.open and not show_media and not notices_on
        self.media.set_shown(show_media)
        if not (show_media or notices_on or show_empty):
            self.shape = HIDDEN
            self.fit()
            return
        self.media.widget.set_visible(show_media)
        self.media.set_view("full" if self.open else "line")
        self.notifications.widget.set_visible(notices_on)
        self.empty.set_visible(show_empty)
        only_a_line = show_media and not self.open and not notices_on
        self.shape = LINE if only_a_line else CARD
        self.content_shape = self.shape
        self.fit()

    def fit(self):
        if not self.ticking:
            self.ticking = True
            self.last_frame = None
            self.canvas.add_tick_callback(self.on_frame)

    def retarget(self):
        width, max_height, shoulder, radius = SHAPES[self.shape]
        from_edge = PADDING[self.content_shape][0]
        height = 0 if self.shape == HIDDEN else min(self.content_height() + 2 * from_edge, max_height)
        for spring, target in zip(self.springs, (width, height, shoulder, radius, from_edge)):
            spring.target = target

    def toggle(self):
        if self.open:
            self.close_view()
            return
        self.open_view()
        if not self.pointer_inside:
            self.later("idle", SHORTCUT_IDLE_MS, self.close_view)

    def open_view(self):
        self.open = True
        self.notifications.set_centre(self.config.notification_centre)

    def close_view(self):
        self.cancel("idle")
        self.open = False
        self.notifications.set_centre(False)

    def peek(self):
        if not self.config.media or self.config.peek_seconds <= 0:
            return
        self.peeking = True
        self.relayout()
        self.later("peek", int(self.config.peek_seconds * 1000), self.end_peek)

    def end_peek(self):
        self.peeking = False
        self.relayout()

    def close_all(self):
        for name in list(self.timers):
            self.cancel(name)
        self.open = False
        self.peeking = False
        self.notifications.set_centre(False)

    # ---- Pointer

    def on_enter(self, _controller, _x, _y):
        self.pointer_inside = True
        self.notifications.set_hovered(True)
        self.cancel("leave")
        self.cancel("idle")
        showing_notifications = self.shape == CARD
        if not self.open and self.config.hover and not showing_notifications:
            self.later("dwell", HOVER_DWELL_MS, self.open_view)

    def on_leave(self, _controller):
        self.pointer_inside = False
        self.notifications.set_hovered(False)
        self.cancel("dwell")
        if self.open:
            self.later("leave", LEAVE_DELAY_MS, self.close_view)

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
        self.retarget()
        for spring in self.springs:
            spring.step(seconds)
        fading = self.fade_content(seconds)
        self.layers.queue_allocate()
        self.canvas.queue_draw()
        self.update_input_region()
        resizing = self.media.widget.get_transition_running()
        if fading or resizing or not all(spring.at_rest() for spring in self.springs):
            return GLib.SOURCE_CONTINUE
        self.ticking = False
        return GLib.SOURCE_REMOVE

    def fade_content(self, seconds) -> bool:
        target = 0.0 if self.shape == HIDDEN else 1.0
        step = seconds / CONTENT_FADE_SECONDS
        opacity = self.content.get_opacity()
        opacity = min(opacity + step, target) if target > opacity else max(opacity - step, target)
        self.content.set_opacity(opacity)
        self.content.set_can_target(opacity > 0)
        return opacity != target

    def update_input_region(self):
        surface = self.get_surface()
        if surface is None:
            return
        outline = self.outline()
        if outline.height >= 1:
            area = (outline.left, 0, outline.full_width, outline.height)
        elif self.config.hover:
            line_width = SHAPES[LINE][0]
            area = (WINDOW_WIDTH / 2 - line_width / 2, 0, line_width, HOT_ZONE_HEIGHT)
        else:
            area = (0, 0, 0, 0)
        x, y, width, height = (round(value) for value in area)
        if not self.at_top:
            y = WINDOW_HEIGHT - height
        surface.set_input_region(cairo.Region(cairo.RectangleInt(x, y, width, height)))
