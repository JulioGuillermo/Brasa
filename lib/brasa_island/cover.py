"""Loads a track's cover (mpris:artUrl: a file or a web address) without blocking the UI."""
import threading
import urllib.request

from gi.repository import Gdk, GLib

DOWNLOAD_TIMEOUT = 5


def fetch(url: str) -> bytes:
    if url.startswith("file://"):
        path = GLib.filename_from_uri(url)[0]
        with open(path, "rb") as image:
            return image.read()
    with urllib.request.urlopen(url, timeout=DOWNLOAD_TIMEOUT) as reply:
        return reply.read()


def load(url: str, on_loaded):
    """Calls on_loaded(url, texture) in the UI thread; texture is None when the cover cannot be read."""

    def work():
        try:
            texture = Gdk.Texture.new_from_bytes(GLib.Bytes.new(fetch(url)))
        except (OSError, ValueError, GLib.Error):
            texture = None
        GLib.idle_add(on_loaded, url, texture)

    threading.Thread(target=work, daemon=True).start()
