"""One island window: a layer-shell surface on one screen edge whose outline grows out of the edge to fit
what it shows. It can host the music and notifications, the app launcher, or both when they share a
place. Springs drive every size, so the outline flows between states; the content is clipped to it."""
import cairo
from gi.repository import Gdk, GLib, Gtk
from gi.repository import Gtk4LayerShell as LayerShell

from .clip import Clip
from .config import Config, Place
from .focus import focused_connector
from .shape import Outline, paint
from .spring import Spring

HIDDEN, LINE, CARD = "hidden", "line", "card"
SHAPES = {  # body width, max body height, shoulder reach, corner radius
    HIDDEN: (150, 0, 0, 0),
    LINE: (352, 44, 64, 22),
    CARD: (470, 620, 120, 38),
    "small": (400, 320, 90, 32),
    "medium": (560, 460, 110, 38),
    "large": (780, 640, 130, 42),
}
PADDING = {LINE: (8, 18), CARD: (16, 24), "small": (14, 18), "medium": (16, 22), "large": (18, 26)}
SECTION_SPACING = 12
SLACK = 40
OVERHANG = 2
HOT_ZONE_HEIGHT = 3
HOVER_DWELL_MS = 120
LEAVE_DELAY_MS = 400
SHORTCUT_IDLE_MS = 5000
CONTENT_FADE_SECONDS = 0.15
MAX_FRAME_SECONDS = 1 / 30
MAIN, CENTRE, LAUNCHER = "main", "centre", "launcher"


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
    """media and notifications are both given or both None; launcher may be None."""

    def __init__(self, app, place: Place, config: Config, media=None, notifications=None, launcher=None):
        self.place = place
        self.config = config
        self.media = media
        self.notifications = notifications
        self.launcher = launcher
        self.shapes = [HIDDEN, LINE, CARD] if media else [HIDDEN]
        if launcher:
            self.shapes.append(config.launcher_size)
        self.window_width = max(SHAPES[s][0] + 2 * SHAPES[s][2] for s in self.shapes) + 2 * SLACK
        self.window_height = max(SHAPES[s][1] for s in self.shapes) + SLACK + OVERHANG
        super().__init__(application=app, css_classes=["brasa-island"],
                         default_width=self.window_width, default_height=self.window_height)
        self.color = rgba(config.color)
        self.shape = HIDDEN
        self.content_shape = self.shapes[1] if len(self.shapes) > 1 else HIDDEN
        self.mode = None
        self.peeking = False
        self.pointer_inside = False
        self.timers = {}
        self.last_frame = None
        self.ticking = False
        width, height, shoulder, radius = SHAPES[HIDDEN]
        self.width, self.height = Spring(width), Spring(height)
        self.shoulder, self.radius = Spring(shoulder), Spring(radius)
        self.inset = Spring(16)
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
        return self.place.edge == "top"

    @property
    def edge_y(self) -> float:
        """Where the screen edge is in the window. The window reaches OVERHANG past it, so its own border,
        which a fractional scale can leave half drawn, stays off screen."""
        return OVERHANG if self.at_top else self.window_height - OVERHANG

    @property
    def is_open(self) -> bool:
        return self.mode is not None

    # ---- Window

    def place_on_screen(self):
        LayerShell.init_for_window(self)
        LayerShell.set_namespace(self, "brasa-island")
        LayerShell.set_layer(self, LayerShell.Layer.OVERLAY)
        LayerShell.set_exclusive_zone(self, -1)
        LayerShell.set_keyboard_mode(self, LayerShell.KeyboardMode.NONE)
        anchored = {
            LayerShell.Edge.TOP: self.at_top,
            LayerShell.Edge.BOTTOM: not self.at_top,
            LayerShell.Edge.LEFT: self.place.align == "left",
            LayerShell.Edge.RIGHT: self.place.align == "right",
        }
        for edge, on in anchored.items():
            LayerShell.set_anchor(self, edge, on)
        screen_edge = LayerShell.Edge.TOP if self.at_top else LayerShell.Edge.BOTTOM
        LayerShell.set_margin(self, screen_edge, -OVERHANG)
        self.monitor = self.place.monitor
        if self.monitor:
            LayerShell.set_monitor(self, find_monitor(self.monitor))

    def follow_focus(self):
        """Without a monitor of its own, the island shows up on the focused one."""
        if self.place.monitor:
            return
        connector = focused_connector()
        monitor = find_monitor(connector)
        if monitor is None or connector == self.monitor:
            return
        self.monitor = connector
        LayerShell.set_monitor(self, monitor)

    def build(self):
        self.canvas = Gtk.DrawingArea(content_width=self.window_width, content_height=self.window_height)
        self.canvas.set_draw_func(self.draw)
        self.content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=SECTION_SPACING, opacity=0)
        for section in self.sections():
            self.content.append(section)
        self.clip = Clip(self.content)
        self.layers = Gtk.Overlay(child=self.canvas)
        self.layers.add_overlay(self.clip)
        self.layers.connect("get-child-position", self.place_content)
        return self.layers

    def sections(self):
        hosted = [self.media, self.notifications, self.launcher]
        return [section.widget for section in hosted if section is not None]

    def release_sections(self):
        """Gives the sections back before the window goes, for the next island to take."""
        for section in self.sections():
            self.content.remove(section)
        if self.media:
            self.media.set_shown(False)

    def order_sections(self):
        """Nearest the edge the music, then the notifications; the launcher stands alone."""
        sections = self.sections()
        if not self.at_top:
            sections.reverse()
        for before, section in zip([None, *sections], sections):
            self.content.reorder_child_after(section, before)
        if self.notifications:
            self.notifications.set_edge(self.at_top)

    def content_width(self) -> int:
        return SHAPES[self.content_shape][0] - 2 * PADDING[self.content_shape][1]

    def content_height(self) -> int:
        return self.content.measure(Gtk.Orientation.VERTICAL, self.content_width())[1]

    def place_content(self, _overlay, _clip, place):
        width = self.content_width()
        height = self.content_height()
        place.x = round((self.window_width - width) / 2)
        place.y = round(self.edge_y + self.inset.value if self.at_top else self.edge_y - self.inset.value - height)
        place.width, place.height = width, height
        body = self.outline()
        top = self.edge_y if self.at_top else self.edge_y - body.height
        self.clip.show_only(body.center - body.width / 2 - place.x, top - place.y, body.width, body.height)
        return True

    def outline(self) -> Outline:
        return Outline(self.window_width / 2, self.width.value, self.height.value, self.shoulder.value,
                       self.radius.value)

    def draw(self, _area, cr, _width, _height):
        cr.translate(0, self.edge_y)
        if not self.at_top:
            cr.scale(1, -1)
        paint(cr, self.outline(), self.color)

    # ---- What it shows

    def relayout(self):
        if self.mode == LAUNCHER:
            self.show_launcher()
            return
        if self.launcher:
            self.launcher.widget.set_visible(False)
        if not self.media:
            self.show_shape(HIDDEN)
            return
        open_main = self.mode in (MAIN, CENTRE)
        notices_on = self.showing_centre() or self.notifications.has_toasts
        media_on = self.config.media and (open_main or self.peeking or (notices_on and self.media.playing()))
        self.media.set_shown(media_on)
        self.media.widget.set_visible(media_on)
        self.media.set_view("full" if open_main else "line")
        self.notifications.widget.set_visible(notices_on)
        if not (media_on or notices_on):
            self.show_shape(HIDDEN)
        elif media_on and not open_main and not notices_on:
            self.show_shape(LINE)
        else:
            self.show_shape(CARD)

    def show_launcher(self):
        if self.media:
            self.media.set_shown(False)
            self.media.widget.set_visible(False)
            self.notifications.widget.set_visible(False)
        self.launcher.widget.set_visible(True)
        self.show_shape(self.config.launcher_size)

    def show_shape(self, shape):
        if self.shape == HIDDEN and shape != HIDDEN:
            self.follow_focus()
        self.shape = shape
        if shape != HIDDEN:
            self.content_shape = shape
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

    # ---- Opening and closing

    def toggle(self):
        """Super+m: music and notifications."""
        if not self.media:
            return
        if self.mode == MAIN:
            self.close_view()
            return
        self.open_view(MAIN)
        self.close_when_idle()

    def toggle_centre(self):
        """Super+n and the bar's bell: music and notifications, with the centre even when it is off."""
        if not self.media:
            return
        if self.mode == CENTRE:
            self.close_view()
            return
        self.open_view(CENTRE)
        self.close_when_idle()

    def showing_centre(self) -> bool:
        return self.mode == CENTRE or (self.mode == MAIN and self.config.notification_centre)

    def toggle_launcher(self, grab_keyboard=True):
        if not self.launcher:
            return
        if self.mode == LAUNCHER:
            self.close_view()
            return
        self.open_view(LAUNCHER, grab_keyboard)
        if not grab_keyboard:
            self.close_when_idle()

    def close_when_idle(self):
        if not self.pointer_inside:
            self.later("idle", SHORTCUT_IDLE_MS, self.close_view)

    def open_view(self, mode, grab_keyboard=True):
        self.cancel("idle")
        self.mode = mode
        if self.notifications:
            self.notifications.set_centre(self.showing_centre())
        if mode == LAUNCHER:
            self.launcher.opened()
        keyboard = LayerShell.KeyboardMode.EXCLUSIVE if mode == LAUNCHER and grab_keyboard else LayerShell.KeyboardMode.NONE
        LayerShell.set_keyboard_mode(self, keyboard)
        self.relayout()

    def close_view(self):
        self.cancel("idle")
        self.mode = None
        LayerShell.set_keyboard_mode(self, LayerShell.KeyboardMode.NONE)
        if self.notifications:
            self.notifications.set_centre(False)
        self.relayout()

    def peek(self):
        if not self.media or not self.config.media or self.config.peek_seconds <= 0:
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
        self.peeking = False
        self.close_view()

    def show_off(self):
        """Opens for a moment, to show where it now is."""
        if self.media:
            self.toggle()
        else:
            self.toggle_launcher(grab_keyboard=False)

    # ---- Pointer

    def hover_opens(self):
        if self.media and self.config.hover:
            return MAIN
        if self.launcher and self.config.launcher_hover:
            return LAUNCHER
        return None

    def on_enter(self, _controller, _x, _y):
        self.pointer_inside = True
        if self.notifications:
            self.notifications.set_hovered(True)
        self.cancel("leave")
        self.cancel("idle")
        showing_notifications = self.shape == CARD and not self.is_open
        mode = self.hover_opens()
        if mode and not self.is_open and not showing_notifications:
            self.later("dwell", HOVER_DWELL_MS, lambda: self.open_view(mode, grab_keyboard=False))

    def on_leave(self, _controller):
        self.pointer_inside = False
        if self.notifications:
            self.notifications.set_hovered(False)
        self.cancel("dwell")
        if self.is_open:
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
        resizing = self.media is not None and self.media.widget.get_transition_running()
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
        elif self.hover_opens():
            line_width = SHAPES[LINE][0]
            area = (self.window_width / 2 - line_width / 2, 0, line_width, HOT_ZONE_HEIGHT)
        else:
            area = (0, 0, 0, 0)
        x, y, width, height = (round(value) for value in area)
        height += OVERHANG
        if not self.at_top:
            y = self.window_height - height
        surface.set_input_region(cairo.Region(cairo.RectangleInt(x, y, width, height)))
