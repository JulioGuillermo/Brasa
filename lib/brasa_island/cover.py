"""Finds and loads a track's cover without blocking the UI: mpris:artUrl (a file or a web address) or,
when the player gives none, the thumbnail of a YouTube video. Always a centred square, so it never
changes the island's layout."""
import threading
import urllib.request

from gi.repository import Gdk, GdkPixbuf, GLib

from . import youtube

DOWNLOAD_TIMEOUT = 5
SIDE_PIXELS = 256


def cover_url(metadata: dict) -> str:
    return metadata.get("mpris:artUrl") or youtube.thumbnail(metadata.get("xesam:url", ""))


def fetch(url: str) -> bytes:
    if url.startswith("file://"):
        path = GLib.filename_from_uri(url)[0]
        with open(path, "rb") as image:
            return image.read()
    with urllib.request.urlopen(url, timeout=DOWNLOAD_TIMEOUT) as reply:
        return reply.read()


def without_letterbox(image: GdkPixbuf.Pixbuf) -> GdkPixbuf.Pixbuf:
    height = image.get_width() * 9 // 16
    return image.new_subpixbuf(0, (image.get_height() - height) // 2, image.get_width(), height)


def square(data: bytes, url: str) -> Gdk.Texture:
    loader = GdkPixbuf.PixbufLoader()
    loader.write(data)
    loader.close()
    image = loader.get_pixbuf()
    if url.startswith(youtube.THUMBNAILS):
        image = without_letterbox(image)
    side = min(image.get_width(), image.get_height())
    left = (image.get_width() - side) // 2
    top = (image.get_height() - side) // 2
    centre = image.new_subpixbuf(left, top, side, side)
    return Gdk.Texture.new_for_pixbuf(centre.scale_simple(SIDE_PIXELS, SIDE_PIXELS, GdkPixbuf.InterpType.BILINEAR))


def load(url: str, on_loaded):
    """Calls on_loaded(url, texture) in the UI thread; texture is None when the cover cannot be read."""

    def work():
        try:
            texture = square(fetch(url), url)
        except (OSError, ValueError, GLib.Error):
            texture = None
        GLib.idle_add(on_loaded, url, texture)

    threading.Thread(target=work, daemon=True).start()
