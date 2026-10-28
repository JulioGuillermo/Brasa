"""One notification as an app sent it (org.freedesktop.Notifications.Notify), read from the D-Bus message."""
from dataclasses import dataclass, field

from gi.repository import GLib

LOW, NORMAL, CRITICAL = 0, 1, 2
DEFAULT_ACTION = "default"
SYNCHRONOUS_HINTS = ("synchronous", "private-synchronous", "x-canonical-private-synchronous")
IMAGE_DATA_HINTS = ("image-data", "image_data", "icon_data")
IMAGE_PATH_HINTS = ("image-path", "image_path")


@dataclass
class Notice:
    id: int
    app: str
    summary: str
    body: str
    app_icon: str = ""
    image_path: str = ""
    image_data: GLib.Variant | None = None
    desktop_entry: str = ""
    buttons: list[tuple[str, str]] = field(default_factory=list)
    urgency: int = NORMAL
    expire_ms: int = -1
    synchronous: str = ""
    replaces_id: int = 0

    @property
    def critical(self) -> bool:
        return self.urgency == CRITICAL


def hint(hints: GLib.Variant, names, kind: str):
    for name in names:
        value = hints.lookup_value(name, GLib.VariantType.new(kind))
        if value is not None:
            return value
    return None


def unpacked_hint(hints, names, kind, default):
    value = hint(hints, names, kind)
    return value.unpack() if value is not None else default


def buttons_of(actions: list[str]) -> list[tuple[str, str]]:
    pairs = zip(actions[0::2], actions[1::2])
    return [(key, label) for key, label in pairs if key != DEFAULT_ACTION]


def from_notify(body: GLib.Variant, notice_id: int) -> Notice:
    """body: the Notify call's arguments (susssasa{sv}i)."""
    hints = body.get_child_value(6)
    return Notice(
        id=notice_id,
        app=body.get_child_value(0).get_string(),
        replaces_id=body.get_child_value(1).get_uint32(),
        app_icon=body.get_child_value(2).get_string(),
        summary=body.get_child_value(3).get_string(),
        body=body.get_child_value(4).get_string(),
        buttons=buttons_of(body.get_child_value(5).unpack()),
        image_data=hint(hints, IMAGE_DATA_HINTS, "(iiibiiay)"),
        image_path=unpacked_hint(hints, IMAGE_PATH_HINTS, "s", ""),
        desktop_entry=unpacked_hint(hints, ("desktop-entry",), "s", ""),
        urgency=unpacked_hint(hints, ("urgency",), "y", NORMAL),
        synchronous=unpacked_hint(hints, SYNCHRONOUS_HINTS, "s", ""),
        expire_ms=body.get_child_value(7).get_int32(),
    )
