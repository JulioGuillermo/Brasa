"""The tiles in pages: three by two, or four by four when the panel is expanded. One turn of the mouse
wheel, or one swipe on the touchpad, moves exactly one page; the dots show where you are."""
import time

from gi.repository import Gdk, Gtk

from .tile import Tile

COMPACT = (3, 2)
EXPANDED = (4, 4)
SLIDE_MS = 220
WHEEL_PAUSE_SECONDS = 0.35
SWIPE_PIXELS = 30


def chunks(items, size):
    return [items[start:start + size] for start in range(0, len(items), size)]


class Pager(Gtk.Box):
    def __init__(self, items, expanded: bool, on_action, on_menu):
        super().__init__(spacing=10, css_classes=["quick-pager"])
        self.columns, self.rows = EXPANDED if expanded else COMPACT
        self.tiles = [Tile(item, on_action, on_menu) for item in items]
        self.stack = Gtk.Stack(transition_type=Gtk.StackTransitionType.SLIDE_UP_DOWN,
                               transition_duration=SLIDE_MS, hexpand=True)
        self.dots = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6, valign=Gtk.Align.CENTER,
                            css_classes=["quick-dots"])
        self.dot_list = []
        for number, tiles in enumerate(chunks(self.tiles, self.per_page)):
            self.stack.add_named(self.page(tiles), str(number))
            dot = Gtk.Button(css_classes=["quick-dot"], tooltip_text=f"Page {number + 1}", focusable=False)
            dot.connect("clicked", lambda _dot, number=number: self.show_page(number))
            self.dots.append(dot)
            self.dot_list.append(dot)
        self.dots.set_visible(self.pages > 1)
        self.append(self.stack)
        self.append(self.dots)
        self.listen_to_wheel()
        self.current = 0
        self.last_turn = 0.0
        self.swipe = 0.0
        self.swipe_used = False
        self.show_page(0)

    @property
    def per_page(self) -> int:
        return self.columns * self.rows

    @property
    def pages(self) -> int:
        return len(self.dot_list)

    def page(self, tiles):
        grid = Gtk.Grid(column_homogeneous=True, row_homogeneous=True, column_spacing=10, row_spacing=12)
        for index in range(self.per_page):
            cell = tiles[index] if index < len(tiles) else Gtk.Box(css_classes=["quick-tile"])
            grid.attach(cell, index % self.columns, index // self.columns, 1, 1)
        return grid

    def show_page(self, number):
        self.current = max(0, min(number, self.pages - 1))
        self.stack.set_visible_child_name(str(self.current))
        for index, dot in enumerate(self.dot_list):
            if index == self.current:
                dot.add_css_class("current")
            else:
                dot.remove_css_class("current")

    def refresh(self):
        for tile in self.tiles:
            tile.refresh()

    # ---- Wheel and touchpad

    def listen_to_wheel(self):
        wheel = Gtk.EventControllerScroll(flags=Gtk.EventControllerScrollFlags.VERTICAL)
        wheel.connect("scroll-begin", self.on_swipe_begin)
        wheel.connect("scroll", self.on_scroll)
        self.add_controller(wheel)

    def on_swipe_begin(self, _controller):
        self.swipe = 0.0
        self.swipe_used = False

    def on_scroll(self, controller, _dx, dy):
        """A wheel sends whole steps; a touchpad sends small ones between scroll-begin and scroll-end,
        and moves one page per swipe."""
        if controller.get_unit() == Gdk.ScrollUnit.WHEEL:
            self.on_wheel_step(dy)
        else:
            self.on_swipe_step(dy)
        return True

    def on_wheel_step(self, dy):
        now = time.monotonic()
        if dy == 0 or now - self.last_turn < WHEEL_PAUSE_SECONDS:
            return
        self.last_turn = now
        self.turn(1 if dy > 0 else -1)

    def on_swipe_step(self, dy):
        if self.swipe_used:
            return
        self.swipe += dy
        if abs(self.swipe) >= SWIPE_PIXELS:
            self.swipe_used = True
            self.turn(1 if self.swipe > 0 else -1)

    def turn(self, step):
        self.show_page(self.current + step)
        tile = self.first_tile_of(self.current)
        if self.get_root().get_focus() in [t.button for t in self.tiles]:
            tile.button.grab_focus()

    # ---- Keyboard

    def first_tile_of(self, page) -> Tile:
        return self.tiles[page * self.per_page]

    def focused_index(self, focus) -> int | None:
        for index, tile in enumerate(self.tiles):
            if tile.button == focus:
                return index
        return None

    def focus_tile(self, index):
        index = max(0, min(index, len(self.tiles) - 1))
        self.show_page(index // self.per_page)
        self.tiles[index].button.grab_focus()

    def neighbour(self, index, key) -> int | None:
        """The tile an arrow key leads to; None past the last row of the last page."""
        page, place = divmod(index, self.per_page)
        row, column = divmod(place, self.columns)
        last = len(self.tiles) - 1
        if key == "left":
            return max(index - 1, 0)
        if key == "right":
            return min(index + 1, last)
        if key == "up":
            if row > 0:
                return index - self.columns
            if page > 0:
                return (page - 1) * self.per_page + (self.rows - 1) * self.columns + column
            return index
        below = index + self.columns
        if row < self.rows - 1 and below <= last:
            return below
        if page < self.pages - 1:
            return min((page + 1) * self.per_page + column, last)
        return None
