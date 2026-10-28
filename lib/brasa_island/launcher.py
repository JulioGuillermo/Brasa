"""The island's app launcher: a search field over every installed app, as a grid of icons or a list.
Typing filters; the arrows move the selection, Enter launches it, Escape closes."""
from dataclasses import dataclass

from gi.repository import Gdk, Gio, Gtk, Pango

from . import apps

FALLBACK_ICON = "application-x-executable"


@dataclass(frozen=True)
class Size:
    list_height: int
    columns: int
    grid_icon: int
    list_icon: int


SIZES = {
    "small": Size(list_height=220, columns=4, grid_icon=40, list_icon=24),
    "medium": Size(list_height=340, columns=5, grid_icon=48, list_icon=28),
    "large": Size(list_height=500, columns=7, grid_icon=56, list_icon=32),
}


def app_icon(app, pixel_size) -> Gtk.Image:
    icon = app.get_icon()
    image = Gtk.Image.new_from_gicon(icon) if icon else Gtk.Image.new_from_icon_name(FALLBACK_ICON)
    image.set_pixel_size(pixel_size)
    return image


def grid_tile(app, size: Size) -> Gtk.Widget:
    name = Gtk.Label(label=app.get_display_name(), ellipsize=Pango.EllipsizeMode.END, max_width_chars=10,
                     justify=Gtk.Justification.CENTER, css_classes=["island-app-name"])
    tile = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6, css_classes=["island-app"])
    tile.append(app_icon(app, size.grid_icon))
    tile.append(name)
    return tile


def list_row(app, size: Size) -> Gtk.Widget:
    name = Gtk.Label(label=app.get_display_name(), xalign=0, ellipsize=Pango.EllipsizeMode.END,
                     css_classes=["island-app-name"])
    about = Gtk.Label(label=app.get_description() or app.get_generic_name() or "", xalign=0, hexpand=True,
                      ellipsize=Pango.EllipsizeMode.END, css_classes=["island-app-about"])
    texts = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, valign=Gtk.Align.CENTER, hexpand=True)
    texts.append(name)
    texts.append(about)
    row = Gtk.Box(spacing=12, css_classes=["island-app", "island-app-row"])
    row.append(app_icon(app, size.list_icon))
    row.append(texts)
    return row


class Launcher:
    """on_done() after launching an app or on Escape."""

    def __init__(self, on_done):
        self.on_done = on_done
        self.view = "grid"
        self.size = SIZES["medium"]
        self.all_apps = apps.installed()
        self.shown = []
        self.tiles = []
        self.selected = 0
        self.search = Gtk.SearchEntry(placeholder_text="Search apps", hexpand=True, css_classes=["island-search"])
        self.search.connect("search-changed", lambda _entry: self.fill())
        self.search.connect("activate", lambda _entry: self.launch_selected())
        self.search.connect("stop-search", lambda _entry: self.on_done())
        self.items = Gtk.FlowBox(selection_mode=Gtk.SelectionMode.NONE, homogeneous=True, valign=Gtk.Align.START,
                                 row_spacing=4, column_spacing=4)
        self.items.connect("child-activated", lambda _box, child: self.launch(child.get_index()))
        self.scroller = Gtk.ScrolledWindow(child=self.items, hscrollbar_policy=Gtk.PolicyType.NEVER,
                                           css_classes=["island-apps"])
        self.nothing = Gtk.Label(label="No app matches", css_classes=["island-time"], vexpand=True)
        self.widget = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        for child in (self.search, self.scroller, self.nothing):
            self.widget.append(child)
        keys = Gtk.EventControllerKey(propagation_phase=Gtk.PropagationPhase.CAPTURE)
        keys.connect("key-pressed", self.on_key)
        self.widget.add_controller(keys)
        Gio.AppInfoMonitor.get().connect("changed", self.on_apps_changed)
        self.fill()

    def set_style(self, view: str, size: str):
        self.view = view
        self.size = SIZES[size]
        self.scroller.set_size_request(-1, self.size.list_height)
        self.nothing.set_size_request(-1, self.size.list_height)
        columns = self.size.columns if view == "grid" else 1
        self.items.set_min_children_per_line(columns)
        self.items.set_max_children_per_line(columns)
        self.fill()

    @property
    def columns(self) -> int:
        return self.items.get_max_children_per_line()

    def opened(self):
        self.search.set_text("")
        self.fill()
        self.search.grab_focus()

    def on_apps_changed(self, _monitor):
        self.all_apps = apps.installed()
        self.fill()

    def fill(self):
        self.shown = apps.matching(self.all_apps, self.search.get_text())
        self.items.remove_all()
        make = grid_tile if self.view == "grid" else list_row
        for app in self.shown:
            self.items.append(make(app, self.size))
        self.scroller.set_visible(bool(self.shown))
        self.nothing.set_visible(not self.shown)
        self.select(0)

    def select(self, index: int):
        if not self.shown:
            return
        old = self.items.get_child_at_index(self.selected)
        if old is not None:
            old.remove_css_class("chosen")
        self.selected = max(0, min(index, len(self.shown) - 1))
        new = self.items.get_child_at_index(self.selected)
        new.add_css_class("chosen")
        self.scroll_to(new)

    def scroll_to(self, child):
        found, bounds = child.compute_bounds(self.items)
        if not found:
            return
        scroll = self.scroller.get_vadjustment()
        top, bottom = bounds.get_y(), bounds.get_y() + bounds.get_height()
        if top < scroll.get_value():
            scroll.set_value(top)
        elif bottom > scroll.get_value() + scroll.get_page_size():
            scroll.set_value(bottom - scroll.get_page_size())

    def on_key(self, _controller, keyval, _keycode, _state):
        moves = {
            Gdk.KEY_Down: self.columns,
            Gdk.KEY_Up: -self.columns,
            Gdk.KEY_Page_Down: self.columns * 3,
            Gdk.KEY_Page_Up: -self.columns * 3,
        }
        if self.view == "grid":
            moves.update({Gdk.KEY_Right: 1, Gdk.KEY_Left: -1, Gdk.KEY_Tab: 1, Gdk.KEY_ISO_Left_Tab: -1})
        if keyval in moves:
            self.select(self.selected + moves[keyval])
            return True
        if keyval == Gdk.KEY_Escape:
            self.on_done()
            return True
        return False

    def launch_selected(self):
        if self.shown:
            self.launch(self.selected)

    def launch(self, index: int):
        apps.launch(self.shown[index])
        self.on_done()
