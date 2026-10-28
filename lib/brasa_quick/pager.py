"""The tiles in pages of six (three by two). The mouse wheel, or a dot, moves between pages."""
from gi.repository import Gtk

from .tile import Tile

COLUMNS = 3
ROWS = 2
PER_PAGE = COLUMNS * ROWS
SLIDE_MS = 220


def chunks(items, size):
    return [items[start:start + size] for start in range(0, len(items), size)]


class Pager(Gtk.Box):
    def __init__(self, items, on_action, on_menu):
        super().__init__(spacing=10, css_classes=["quick-pager"])
        self.tiles = [Tile(item, on_action, on_menu) for item in items]
        self.stack = Gtk.Stack(transition_type=Gtk.StackTransitionType.SLIDE_UP_DOWN,
                               transition_duration=SLIDE_MS, hexpand=True)
        self.dots = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6, valign=Gtk.Align.CENTER,
                            css_classes=["quick-dots"])
        for number, tiles in enumerate(chunks(self.tiles, PER_PAGE)):
            self.stack.add_named(self.page(tiles), str(number))
            dot = Gtk.Button(css_classes=["quick-dot"], tooltip_text=f"Page {number + 1}")
            dot.connect("clicked", lambda _dot, number=number: self.show_page(number))
            self.dots.append(dot)
        self.pages = len(self.dots.observe_children())
        self.dots.set_visible(self.pages > 1)
        self.append(self.stack)
        self.append(self.dots)
        wheel = Gtk.EventControllerScroll(flags=Gtk.EventControllerScrollFlags.VERTICAL
                                          | Gtk.EventControllerScrollFlags.DISCRETE)
        wheel.connect("scroll", self.on_wheel)
        self.add_controller(wheel)
        self.current = 0
        self.show_page(0)

    def page(self, tiles):
        grid = Gtk.Grid(column_homogeneous=True, row_homogeneous=True, column_spacing=10, row_spacing=12)
        for index, tile in enumerate(tiles):
            grid.attach(tile, index % COLUMNS, index // COLUMNS, 1, 1)
        for index in range(len(tiles), PER_PAGE):
            grid.attach(Gtk.Box(css_classes=["quick-tile"]), index % COLUMNS, index // COLUMNS, 1, 1)
        return grid

    def on_wheel(self, _controller, _dx, dy):
        if dy > 0:
            self.show_page(self.current + 1)
        elif dy < 0:
            self.show_page(self.current - 1)
        return True

    def show_page(self, number):
        number = max(0, min(number, self.pages - 1))
        self.current = number
        self.stack.set_visible_child_name(str(number))
        for index, dot in enumerate(self.dots.observe_children()):
            if index == number:
                dot.add_css_class("current")
            else:
                dot.remove_css_class("current")

    def refresh(self):
        for tile in self.tiles:
            tile.refresh()
