"""The island's media module: what is playing, with cover, progress, controls and volume.
Gives two views: `peek` (one line, shown for a moment when the track or play/pause changes) and `full`."""
import time

from gi.repository import GLib, Gtk, Pango

from . import cover, volume
from .players import PLAYING, Players, metadata

POLL_MS = 500
VOLUME_WRITE_MS = 40
USER_HOLD_SECONDS = 0.8
FULL_COVER = 84
PEEK_COVER = 28
PLACEHOLDER_ICON = "audio-x-generic-symbolic"


def clock(seconds: float) -> str:
    seconds = max(int(seconds), 0)
    if seconds >= 3600:
        return f"{seconds // 3600}:{seconds // 60 % 60:02}:{seconds % 60:02}"
    return f"{seconds // 60}:{seconds % 60:02}"


def label(css, xalign=0.0, chars=None):
    text = Gtk.Label(xalign=xalign, css_classes=css, ellipsize=Pango.EllipsizeMode.END)
    if chars:
        text.set_max_width_chars(chars)
    return text


def round_button(icon, css, on_click):
    button = Gtk.Button(icon_name=icon, css_classes=["island-button", *css], valign=Gtk.Align.CENTER)
    button.connect("clicked", lambda _button: on_click())
    return button


class Cover(Gtk.Box):
    def __init__(self, size):
        super().__init__(css_classes=["island-cover"], overflow=Gtk.Overflow.HIDDEN,
                         width_request=size, height_request=size,
                         halign=Gtk.Align.START, valign=Gtk.Align.CENTER, hexpand=False, vexpand=False)
        self.size = size
        self.image = Gtk.Image(hexpand=True, halign=Gtk.Align.CENTER, valign=Gtk.Align.CENTER)
        self.append(self.image)
        self.show(None)

    def show(self, texture):
        if texture is None:
            self.image.set_from_icon_name(PLACEHOLDER_ICON)
            self.image.set_pixel_size(self.size // 2)
            return
        self.image.set_from_paintable(texture)
        self.image.set_pixel_size(self.size)


class Media:
    def __init__(self, on_peek):
        self.on_peek = on_peek
        self.shown = False
        self.poll_source = 0
        self.volume_source = 0
        self.seek_hold_until = 0.0
        self.volume_hold_until = 0.0
        self.cover_url = None
        self.covers = []
        self.peek = self.build_peek()
        self.full = self.build_full()
        self.players = Players(self.on_players_change)
        self.players.start()
        self.refresh()

    def build_peek(self):
        self.peek_cover = Cover(PEEK_COVER)
        self.covers.append(self.peek_cover)
        self.peek_title = label(["island-title"], chars=40)
        self.peek_title.set_hexpand(True)
        self.peek_artist = label(["island-artist"], chars=12)
        self.peek_state = Gtk.Image(css_classes=["island-state"])
        texts = Gtk.Box(spacing=8, hexpand=True)
        texts.append(self.peek_title)
        texts.append(self.peek_artist)
        peek = Gtk.Box(spacing=10, css_classes=["island-peek"])
        for child in (self.peek_cover, texts, self.peek_state):
            peek.append(child)
        return peek

    def build_full(self):
        self.full_cover = Cover(FULL_COVER)
        self.covers.append(self.full_cover)
        self.title = label(["island-title", "island-big"], chars=28)
        self.artist = label(["island-artist"], chars=32)
        self.album = label(["island-album"], chars=32)
        texts = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2, valign=Gtk.Align.CENTER, hexpand=True)
        for child in (self.title, self.artist, self.album):
            texts.append(child)
        top = Gtk.Box(spacing=14)
        top.append(self.full_cover)
        top.append(texts)

        self.position = label(["island-time"], xalign=1.0)
        self.length = label(["island-time"])
        self.progress = Gtk.Scale(hexpand=True, css_classes=["island-progress"], draw_value=False)
        self.progress.connect("change-value", self.on_seek)
        progress = Gtk.Box(spacing=8)
        for child in (self.position, self.progress, self.length):
            progress.append(child)

        self.play = round_button("media-playback-start-symbolic", ["island-play"], self.play_pause)
        buttons = Gtk.Box(spacing=6)
        buttons.append(round_button("media-skip-backward-symbolic", [], self.previous))
        buttons.append(self.play)
        buttons.append(round_button("media-skip-forward-symbolic", [], self.next))
        self.volume = Gtk.Scale(width_request=120, css_classes=["island-volume"], draw_value=False,
                                adjustment=Gtk.Adjustment(upper=1.0, step_increment=0.05, page_increment=0.1))
        self.volume.connect("change-value", self.on_volume)
        sound = Gtk.Box(spacing=4, halign=Gtk.Align.END, hexpand=True)
        sound.append(Gtk.Image(icon_name="audio-volume-high-symbolic", css_classes=["island-volume-icon"]))
        sound.append(self.volume)
        controls = Gtk.Box()
        controls.append(buttons)
        controls.append(sound)

        full = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6, css_classes=["island-full"])
        for child in (top, progress, controls):
            full.append(child)
        return full

    def set_shown(self, shown: bool):
        if shown == self.shown:
            return
        self.shown = shown
        if self.poll_source:
            GLib.source_remove(self.poll_source)
            self.poll_source = 0
        if shown:
            self.poll()
            self.poll_source = GLib.timeout_add(POLL_MS, self.poll)

    def on_players_change(self, reason):
        self.refresh()
        if reason in ("track", "status"):
            self.on_peek()

    def refresh(self):
        player = self.players.current
        if player is None:
            self.show_nothing()
            return
        data = metadata(player)
        title = data.get("xesam:title") or player.props.player_name
        artists = ", ".join(data.get("xesam:artist", []))
        self.title.set_text(title)
        self.peek_title.set_text(title)
        self.artist.set_text(artists)
        self.peek_artist.set_text(artists)
        self.album.set_text(data.get("xesam:album", ""))
        self.show_length(data.get("mpris:length", 0) / 1e6)
        state = "media-playback-pause-symbolic" if player.props.playback_status == PLAYING else "media-playback-start-symbolic"
        self.play.set_icon_name(state)
        self.peek_state.set_from_icon_name(state)
        self.load_cover(cover.cover_url(data))

    def show_length(self, seconds):
        known = seconds > 0
        self.progress.set_sensitive(known)
        self.progress.set_range(0, seconds if known else 1)
        self.length.set_text(clock(seconds) if known else "")
        if not known:
            self.progress.set_value(0)

    def show_nothing(self):
        for text in (self.title, self.peek_title):
            text.set_text("Nothing playing")
        for text in (self.artist, self.peek_artist, self.album, self.position):
            text.set_text("")
        self.show_length(0)
        self.load_cover("")

    def load_cover(self, url):
        if url == self.cover_url:
            return
        self.cover_url = url
        if not url:
            self.on_cover(url, None)
            return
        cover.load(url, self.on_cover)

    def on_cover(self, url, texture):
        if url != self.cover_url:
            return
        for view in self.covers:
            view.show(texture)

    def poll(self):
        now = time.monotonic()
        player = self.players.current
        if player is not None and self.progress.get_sensitive() and now > self.seek_hold_until:
            try:
                seconds = player.get_position() / 1e6
                self.progress.set_value(seconds)
                self.position.set_text(clock(seconds))
            except GLib.Error:
                self.position.set_text("")
        if now > self.volume_hold_until:
            self.volume.set_value(volume.read())
        return GLib.SOURCE_CONTINUE

    def on_seek(self, _scale, _scroll, seconds):
        player = self.players.current
        if player is None:
            return True
        self.seek_hold_until = time.monotonic() + USER_HOLD_SECONDS
        self.position.set_text(clock(seconds))
        try:
            player.set_position(int(seconds * 1e6))
        except GLib.Error:
            pass
        return False

    def on_volume(self, _scale, _scroll, _level):
        self.volume_hold_until = time.monotonic() + USER_HOLD_SECONDS
        if not self.volume_source:
            self.volume_source = GLib.timeout_add(VOLUME_WRITE_MS, self.write_volume)
        return False

    def write_volume(self):
        self.volume_source = 0
        volume.write(self.volume.get_value())
        return GLib.SOURCE_REMOVE

    def control(self, action):
        player = self.players.current
        if player is None:
            return
        try:
            action(player)
        except GLib.Error:
            pass

    def play_pause(self):
        self.control(lambda player: player.play_pause())

    def previous(self):
        self.control(lambda player: player.previous())

    def next(self):
        self.control(lambda player: player.next())
