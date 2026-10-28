"""How one notification looks in the island: icon, app, title, text and its buttons."""
import re

from gi.repository import Gdk, GdkPixbuf, Gio, GLib, Gtk, Pango

from .notice import DEFAULT_ACTION, Notice

ICON_SIZE = 36
BODY_LINES = 4
FALLBACK_ICON = "dialog-information-symbolic"
IMAGE_TAG = re.compile(r"<img\b[^>]*>", re.IGNORECASE)
STALE_HINT = "Its app no longer answers this one"


def texture_from_data(data: GLib.Variant) -> Gdk.Texture:
    width, height, rowstride, has_alpha, bits, _channels, _pixels = data.unpack()
    pixels = data.get_child_value(6).get_data_as_bytes()
    image = GdkPixbuf.Pixbuf.new_from_bytes(pixels, GdkPixbuf.Colorspace.RGB, has_alpha, bits,
                                            width, height, rowstride)
    return Gdk.Texture.new_for_pixbuf(image)


def show_source(image: Gtk.Image, source: str) -> bool:
    """source: a file, a file:// address or an icon name."""
    if source.startswith("file://"):
        source = GLib.filename_from_uri(source)[0]
    if source.startswith("/"):
        try:
            image.set_from_paintable(Gdk.Texture.new_from_filename(source))
            return True
        except GLib.Error:
            return False
    if source and Gtk.IconTheme.get_for_display(Gdk.Display.get_default()).has_icon(source):
        image.set_from_icon_name(source)
        return True
    return False


def show_desktop_icon(image: Gtk.Image, entry: str) -> bool:
    try:
        app = Gio.DesktopAppInfo.new(f"{entry.removesuffix('.desktop')}.desktop")
    except TypeError:
        app = None
    if app is None or app.get_icon() is None:
        return False
    image.set_from_gicon(app.get_icon())
    return True


def show_icon(image: Gtk.Image, notice: Notice):
    if notice.image_data is not None:
        image.set_from_paintable(texture_from_data(notice.image_data))
        return
    for source in (notice.image_path, notice.app_icon):
        if show_source(image, source):
            return
    if notice.desktop_entry and show_desktop_icon(image, notice.desktop_entry):
        return
    image.set_from_icon_name(FALLBACK_ICON)


def safe_markup(text: str) -> str:
    text = IMAGE_TAG.sub("", text)
    try:
        Pango.parse_markup(text, -1, "\0")
        return text
    except GLib.Error:
        return GLib.markup_escape_text(text)


class NoticeRow(Gtk.Box):
    """on_close(notice_id) for the close button (deletes it); on_press(notice_id, action_key) for its buttons
    and for a click on it. A stale row is one its app no longer listens to (it closed it, or the island
    restarted): greyed buttons, and a click opens the app."""

    def __init__(self, notice: Notice, on_close, on_press):
        super().__init__(spacing=12, css_classes=["island-notice"])
        self.on_close = on_close
        self.on_press = on_press
        self.notice = notice
        self.stale = False
        self.icon = Gtk.Image(pixel_size=ICON_SIZE, valign=Gtk.Align.START, css_classes=["island-notice-icon"])
        self.app = Gtk.Label(xalign=0, hexpand=True, ellipsize=Pango.EllipsizeMode.END,
                             css_classes=["island-notice-app"])
        close = Gtk.Button(icon_name="window-close-symbolic", css_classes=["island-button", "island-close"],
                           valign=Gtk.Align.START, tooltip_text="Delete")
        close.connect("clicked", lambda _button: self.on_close(self.notice.id))
        header = Gtk.Box(spacing=6)
        header.append(self.app)
        header.append(close)
        self.summary = Gtk.Label(xalign=0, wrap=True, wrap_mode=Pango.WrapMode.WORD_CHAR, lines=2,
                                 ellipsize=Pango.EllipsizeMode.END, css_classes=["island-title"])
        self.body = Gtk.Label(xalign=0, wrap=True, wrap_mode=Pango.WrapMode.WORD_CHAR, lines=BODY_LINES,
                              ellipsize=Pango.EllipsizeMode.END, css_classes=["island-notice-body"])
        self.buttons = Gtk.Box(spacing=6, homogeneous=True, css_classes=["island-notice-buttons"])
        texts = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2, hexpand=True)
        for child in (header, self.summary, self.body, self.buttons):
            texts.append(child)
        self.append(self.icon)
        self.append(texts)
        click = Gtk.GestureClick()
        click.connect("released", self.on_click)
        self.add_controller(click)
        self.show_notice(notice)

    def show_notice(self, notice: Notice):
        self.notice = notice
        show_icon(self.icon, notice)
        self.set_css_classes(["island-notice", "critical"] if notice.critical else ["island-notice"])
        if self.stale:
            self.add_css_class("stale")
        self.app.set_text(notice.app)
        self.summary.set_text(notice.summary)
        self.summary.set_visible(bool(notice.summary))
        self.body.set_markup(safe_markup(notice.body))
        self.body.set_visible(bool(notice.body))
        while child := self.buttons.get_first_child():
            self.buttons.remove(child)
        for key, label in notice.buttons:
            button = Gtk.Button(label=label, css_classes=["island-action"])
            button.connect("clicked", lambda _button, key=key: self.on_press(self.notice.id, key))
            self.buttons.append(button)
        self.buttons.set_visible(bool(notice.buttons))
        self.set_stale(self.stale)

    def set_stale(self, stale: bool):
        self.stale = stale
        if stale:
            self.add_css_class("stale")
        else:
            self.remove_css_class("stale")
        button = self.buttons.get_first_child()
        while button is not None:
            button.set_sensitive(not stale)
            button.set_tooltip_text(STALE_HINT if stale else None)
            button = button.get_next_sibling()

    @property
    def clickable(self) -> bool:
        return bool(self.notice.desktop_entry) if self.stale else self.notice.has_default

    def on_click(self, gesture, _presses, _x, _y):
        if self.clickable:
            gesture.set_state(Gtk.EventSequenceState.CLAIMED)
            self.on_press(self.notice.id, DEFAULT_ACTION)
