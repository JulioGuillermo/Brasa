"""Keeps the notification centre across restarts of the island: the notices and Do Not Disturb in
~/.local/state/brasa/notifications.json, their pictures as PNG files beside it (an app sends a picture
as raw pixels, which are saved once and named after their content)."""
import dataclasses
import hashlib
import json
import os

from gi.repository import GdkPixbuf, GLib

from .notice import Notice

FOLDER = os.path.join(GLib.get_user_state_dir(), "brasa")
PATH = os.path.join(FOLDER, "notifications.json")
PICTURES = os.path.join(FOLDER, "notification-pictures")
KEPT_FIELDS = ("id", "app", "summary", "body", "app_icon", "image_path", "desktop_entry", "urgency")


@dataclasses.dataclass
class Saved:
    notices: list[Notice]
    closed_by_app: set[int]
    dnd: bool


def picture_file(data: GLib.Variant) -> str:
    """Writes raw pixels (the image-data hint) as a PNG, once; returns its path, or "" if it fails."""
    width, height, rowstride, has_alpha, bits, _channels, _pixels = data.unpack()
    pixels = data.get_child_value(6).get_data_as_bytes()
    path = os.path.join(PICTURES, hashlib.sha1(pixels.get_data()).hexdigest() + ".png")
    if os.path.exists(path):
        return path
    try:
        os.makedirs(PICTURES, exist_ok=True)
        picture = GdkPixbuf.Pixbuf.new_from_bytes(pixels, GdkPixbuf.Colorspace.RGB, has_alpha, bits,
                                                  width, height, rowstride)
        picture.savev(path, "png", [], [])
        return path
    except GLib.Error:
        return ""


def record(notice: Notice) -> dict:
    kept = {name: getattr(notice, name) for name in KEPT_FIELDS}
    if notice.image_data is not None:
        kept["image_path"] = picture_file(notice.image_data) or notice.image_path
    return kept


def save(notices: list[Notice], closed_by_app: set[int], dnd: bool):
    records = [record(notice) for notice in notices]
    state = {"dnd": dnd, "closed_by_app": sorted(closed_by_app), "notices": records}
    os.makedirs(FOLDER, exist_ok=True)
    with open(f"{PATH}.new", "w", encoding="utf-8") as file:
        json.dump(state, file, ensure_ascii=False)
    os.replace(f"{PATH}.new", PATH)
    forget_unused_pictures({kept["image_path"] for kept in records})


def forget_unused_pictures(in_use: set[str]):
    if not os.path.isdir(PICTURES):
        return
    for name in os.listdir(PICTURES):
        path = os.path.join(PICTURES, name)
        if path not in in_use:
            os.remove(path)


def load() -> Saved:
    try:
        with open(PATH, encoding="utf-8") as file:
            state = json.load(file)
        notices = [Notice(**{name: kept[name] for name in KEPT_FIELDS}) for kept in state["notices"]]
        return Saved(notices, set(state["closed_by_app"]), bool(state["dnd"]))
    except (OSError, ValueError, KeyError, TypeError):
        return Saved([], set(), False)
