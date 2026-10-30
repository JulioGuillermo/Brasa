"""A dialog listing the installed apps, with their icons, to search and pick one."""
import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gio, Gtk


def installed_apps():
    apps = [app for app in Gio.AppInfo.get_all() if app.should_show()]
    return sorted(apps, key=lambda app: app.get_display_name().lower())


def desktop_id(app):
    return app.get_id().removesuffix(".desktop")


class AppRow(Gtk.ListBoxRow):
    def __init__(self, app):
        super().__init__()
        self.app = app
        self.search_text = f"{app.get_display_name()} {desktop_id(app)} {app.get_description() or ''}".lower()
        icon = Gtk.Image(gicon=app.get_icon() or Gio.ThemedIcon.new("application-x-executable"), pixel_size=32)
        name = Gtk.Label(label=app.get_display_name(), xalign=0)
        detail = Gtk.Label(label=desktop_id(app), xalign=0, css_classes=["dim-label", "caption"])
        texts = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, valign=Gtk.Align.CENTER)
        texts.append(name)
        texts.append(detail)
        box = Gtk.Box(spacing=12, margin_top=6, margin_bottom=6, margin_start=6, margin_end=6)
        box.append(icon)
        box.append(texts)
        self.set_child(box)


class AppPicker(Adw.Dialog):
    """on_pick(app: Gio.AppInfo) runs when one is chosen."""

    def __init__(self, on_pick):
        super().__init__(title="Choose an app", content_width=460, content_height=600)
        self.on_pick = on_pick
        self.search = Gtk.SearchEntry(placeholder_text="Search apps", hexpand=True)
        self.search.connect("search-changed", lambda _: self.list.invalidate_filter())
        self.search.connect("activate", lambda _: self.pick_first())
        self.list = Gtk.ListBox(css_classes=["navigation-sidebar"])
        self.list.set_filter_func(self.matches)
        self.list.connect("row-activated", lambda _, row: self.pick(row.app))
        for app in installed_apps():
            self.list.append(AppRow(app))
        header = Adw.HeaderBar(title_widget=self.search)
        view = Adw.ToolbarView(content=Gtk.ScrolledWindow(child=self.list, vexpand=True))
        view.add_top_bar(header)
        self.set_child(view)
        self.set_focus(self.search)

    def matches(self, row):
        words = self.search.get_text().lower().split()
        return all(word in row.search_text for word in words)

    def pick_first(self):
        row = self.list.get_first_child()
        while row and not (row.get_mapped() and row.get_child_visible()):
            row = row.get_next_sibling()
        if isinstance(row, AppRow):
            self.pick(row.app)

    def pick(self, app):
        self.close()
        self.on_pick(app)
