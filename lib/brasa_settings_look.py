"""The Appearance page of Brasa settings: the icon theme and the cursor theme, out of every one installed,
each with a preview. A choice applies at once."""
import os

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gdk, GLib, Gtk

import brasa_themes_apply as themes
from brasa_themes_found import installed
from brasa_xcursor import read_image

PREVIEW_ICONS = ["user-home", "folder", "utilities-terminal", "internet-web-browser", "text-x-generic",
                 "preferences-system"]
PREVIEW_CURSORS = [("left_ptr", "default"), ("hand2", "pointer"), ("xterm", "text"), ("watch", "wait"),
                   ("fleur", "move")]
PREVIEW_SIZE = 28


def icon_preview(theme):
    icon_theme = Gtk.IconTheme(theme_name=theme.folder)
    box = Gtk.Box(spacing=6, valign=Gtk.Align.CENTER)
    for name in PREVIEW_ICONS:
        paintable = icon_theme.lookup_icon(name, None, PREVIEW_SIZE, 1, Gtk.TextDirection.NONE, 0)
        box.append(Gtk.Image(paintable=paintable, pixel_size=PREVIEW_SIZE))
    return box


def cursor_file(theme, names):
    for name in names:
        path = os.path.join(theme.path, "cursors", name)
        if os.path.exists(path):
            return path
    return None


def cursor_texture(path):
    image = read_image(path, PREVIEW_SIZE) if path else None
    if image is None:
        return None
    return Gdk.MemoryTexture.new(image.width, image.height, Gdk.MemoryFormat.B8G8R8A8_PREMULTIPLIED,
                                 GLib.Bytes.new(image.pixels), image.width * 4)


def cursor_preview(theme):
    box = Gtk.Box(spacing=6, valign=Gtk.Align.CENTER)
    for names in PREVIEW_CURSORS:
        texture = cursor_texture(cursor_file(theme, names))
        if texture:
            box.append(Gtk.Image(paintable=texture, pixel_size=PREVIEW_SIZE))
    return box


class ThemeGroup(Adw.PreferencesGroup):
    """One row per installed theme of a kind, the chosen one checked. The previews come one by one once
    the window is up: some themes take a quarter of a second to read."""

    def __init__(self, title, description, kind, choice, preview):
        super().__init__(title=title, description=description)
        chosen = themes.load()[choice]
        first_button = None
        for theme in installed(kind):
            button = Gtk.CheckButton(group=first_button, active=theme.folder == chosen)
            first_button = first_button or button
            button.connect("toggled", self.toggled, choice, theme.folder)
            subtitle = theme.folder if theme.folder != theme.name else None
            row = Adw.ActionRow(title=theme.name, subtitle=subtitle, activatable_widget=button)
            row.add_prefix(button)
            GLib.idle_add(self.add_preview, row, preview, theme)
            self.add(row)

    @staticmethod
    def add_preview(row, preview, theme):
        row.add_suffix(preview(theme))
        return GLib.SOURCE_REMOVE

    @staticmethod
    def toggled(button, choice, folder):
        if button.get_active():
            themes.choose(choice, folder)


class LookPage(Adw.PreferencesPage):
    def __init__(self):
        super().__init__()
        self.add(ThemeGroup("Icons", "Every installed icon theme · applies at once, some apps only when "
                                     "they start again", "icons", "icons", icon_preview))
        self.add(ThemeGroup("Cursor", "Every installed cursor theme · applies at once", "cursors", "cursor",
                            cursor_preview))
