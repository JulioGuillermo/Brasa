"""The Appearance page of Brasa settings: the icon theme and the cursor theme, out of every one installed,
each with a preview. A choice applies at once."""
import os
import threading

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("GdkPixbuf", "2.0")
from gi.repository import Adw, Gdk, GdkPixbuf, GLib, Gtk

import brasa_themes_apply as themes
from brasa_icon_files import find_icon
from brasa_themes_found import installed
from brasa_xcursor import read_image

PREVIEW_ICONS = ["user-home", "folder", "utilities-terminal", "internet-web-browser", "text-x-generic",
                 "preferences-system"]
PREVIEW_CURSORS = [("left_ptr", "default"), ("hand2", "pointer"), ("xterm", "text"), ("watch", "wait"),
                   ("fleur", "move")]
PREVIEW_SIZE = 28


def pixbuf_texture(pixbuf):
    memory_format = Gdk.MemoryFormat.R8G8B8A8 if pixbuf.get_has_alpha() else Gdk.MemoryFormat.R8G8B8
    return Gdk.MemoryTexture.new(pixbuf.get_width(), pixbuf.get_height(), memory_format,
                                 pixbuf.read_pixel_bytes(), pixbuf.get_rowstride())


def icon_textures(theme):
    textures = []
    for name in PREVIEW_ICONS:
        file = find_icon(theme.folder, name, PREVIEW_SIZE)
        if not file:
            continue
        try:
            pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(file, PREVIEW_SIZE, PREVIEW_SIZE, True)
        except GLib.Error:
            continue
        textures.append(pixbuf_texture(pixbuf))
    return textures


def cursor_file(theme, names):
    for name in names:
        path = os.path.join(theme.path, "cursors", name)
        if os.path.exists(path):
            return path
    return None


def cursor_textures(theme):
    textures = []
    for names in PREVIEW_CURSORS:
        file = cursor_file(theme, names)
        image = read_image(file, PREVIEW_SIZE) if file else None
        if image is None:
            continue
        textures.append(Gdk.MemoryTexture.new(image.width, image.height, Gdk.MemoryFormat.B8G8R8A8_PREMULTIPLIED,
                                              GLib.Bytes.new(image.pixels), image.width * 4))
    return textures


def show_textures(box, textures):
    for texture in textures:
        box.append(Gtk.Image(paintable=texture, pixel_size=PREVIEW_SIZE))
    return GLib.SOURCE_REMOVE


def load_previews(previews, make_textures):
    """Off the main thread: reading some themes takes half a second. Each row gets its pictures as soon
    as they are ready."""
    for theme, box in previews:
        textures = make_textures(theme)
        GLib.idle_add(show_textures, box, textures)


class ThemeGroup(Adw.PreferencesGroup):
    """One row per installed theme of a kind, the chosen one checked, previews filled in the background."""

    def __init__(self, title, description, kind, choice, make_textures):
        super().__init__(title=title, description=description)
        chosen = themes.load()[choice]
        first_button = None
        previews = []
        for theme in installed(kind):
            button = Gtk.CheckButton(group=first_button, active=theme.folder == chosen)
            first_button = first_button or button
            button.connect("toggled", self.toggled, choice, theme.folder)
            subtitle = theme.folder if theme.folder != theme.name else None
            row = Adw.ActionRow(title=theme.name, subtitle=subtitle, activatable_widget=button)
            row.add_prefix(button)
            preview = Gtk.Box(spacing=6, valign=Gtk.Align.CENTER)
            row.add_suffix(preview)
            previews.append((theme, preview))
            self.add(row)
        threading.Thread(target=load_previews, args=(previews, make_textures), daemon=True).start()

    @staticmethod
    def toggled(button, choice, folder):
        if button.get_active():
            themes.choose(choice, folder)


class LookPage(Adw.PreferencesPage):
    def __init__(self):
        super().__init__()
        self.add(ThemeGroup("Icons", "Every installed icon theme · applies at once, some apps only when "
                                     "they start again", "icons", "icons", icon_textures))
        self.add(ThemeGroup("Cursor", "Every installed cursor theme · applies at once", "cursors", "cursor",
                            cursor_textures))
