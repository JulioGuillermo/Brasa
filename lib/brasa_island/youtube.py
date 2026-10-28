"""What the island can learn from a YouTube address on its own: the video's thumbnail and its length."""
import re
import threading
import urllib.parse
import urllib.request

from gi.repository import GLib

THUMBNAILS = "https://i.ytimg.com/vi/"
HOSTS = ("youtube.com", "www.youtube.com", "music.youtube.com", "m.youtube.com")
PAGE_TIMEOUT = 8
LENGTH_IN_PAGE = re.compile(rb'"lengthSeconds":"(\d+)"')
BROWSER = "Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0"

lengths = {}


def video_id(page_url: str) -> str:
    page = urllib.parse.urlparse(page_url)
    if page.hostname == "youtu.be":
        return page.path.strip("/")
    if page.hostname in HOSTS:
        return urllib.parse.parse_qs(page.query).get("v", [""])[0]
    return ""


def thumbnail(page_url: str) -> str:
    video = video_id(page_url)
    return f"{THUMBNAILS}{video}/hqdefault.jpg" if video else ""


def fetch_length(video: str) -> float:
    request = urllib.request.Request(f"https://www.youtube.com/watch?v={video}",
                                     headers={"User-Agent": BROWSER, "Accept-Language": "en"})
    with urllib.request.urlopen(request, timeout=PAGE_TIMEOUT) as reply:
        found = LENGTH_IN_PAGE.search(reply.read())
    return float(found.group(1)) if found else 0.0


def look_up_length(page_url: str, on_found):
    """Calls on_found(page_url, seconds) in the UI thread when the video's page gives its length.
    Each video is asked once; a failed lookup is not retried."""
    video = video_id(page_url)
    if not video:
        return
    if video in lengths:
        if lengths[video]:
            GLib.idle_add(on_found, page_url, lengths[video])
        return
    lengths[video] = 0.0

    def work():
        try:
            lengths[video] = fetch_length(video)
        except (OSError, ValueError):
            return
        if lengths[video]:
            GLib.idle_add(on_found, page_url, lengths[video])

    threading.Thread(target=work, daemon=True).start()
