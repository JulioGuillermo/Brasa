"""The island's settings, from ~/.config/brasa/island.conf (KEY=value lines, # for comments)."""
import os
import re
from dataclasses import dataclass, field

PATH = os.path.expanduser("~/.config/brasa/island.conf")
EDGES = ("top", "bottom")
ALIGNS = ("left", "center", "right")
LAUNCHER_VIEWS = ("grid", "list")
LAUNCHER_SIZES = ("small", "medium", "large")


@dataclass(frozen=True)
class Place:
    edge: str = "top"
    align: str = "center"
    monitor: str = ""


@dataclass
class Config:
    place: Place = field(default_factory=Place)
    hover: bool = True
    media: bool = True
    notification_centre: bool = True
    peek_seconds: float = 2.5
    color: str = "#0c1838"
    launcher: bool = True
    launcher_place: Place = field(default_factory=lambda: Place(edge="bottom"))
    launcher_view: str = "grid"
    launcher_size: str = "medium"
    launcher_hover: bool = True


def read_pairs(path):
    pairs = {}
    with open(path) as conf:
        for line in conf:
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            value = re.split(r"\s+#", value)[0]
            pairs[key.strip().lower()] = value.strip().strip("\"'")
    return pairs


def switched_on(value: str) -> bool:
    return value.lower() not in ("off", "no", "false", "0")


def on_off(value: bool) -> str:
    return "on" if value else "off"


def one_of(value: str, choices, default: str) -> str:
    return value if value in choices else default


def place_from(pairs, prefix: str, default: Place) -> Place:
    return Place(
        edge=one_of(pairs.get(f"{prefix}edge", ""), EDGES, default.edge),
        align=one_of(pairs.get(f"{prefix}align", ""), ALIGNS, default.align),
        monitor=pairs.get(f"{prefix}monitor", default.monitor),
    )


def load() -> Config:
    config = Config()
    if not os.path.exists(PATH):
        return config
    pairs = read_pairs(PATH)
    config.place = place_from(pairs, "", config.place)
    config.launcher_place = place_from(pairs, "launcher_", config.launcher_place)
    config.hover = switched_on(pairs.get("hover", "on"))
    config.media = switched_on(pairs.get("media", "on"))
    config.notification_centre = switched_on(pairs.get("notification_centre", "on"))
    config.launcher = switched_on(pairs.get("launcher", "on"))
    config.launcher_view = one_of(pairs.get("launcher_view", ""), LAUNCHER_VIEWS, config.launcher_view)
    config.launcher_size = one_of(pairs.get("launcher_size", ""), LAUNCHER_SIZES, config.launcher_size)
    config.launcher_hover = switched_on(pairs.get("launcher_hover", on_off(config.hover)))
    try:
        config.peek_seconds = float(pairs.get("peek_seconds", config.peek_seconds))
    except ValueError:
        pass
    if pairs.get("color", "").startswith("#"):
        config.color = pairs["color"]
    return config


def save(config: Config):
    lines = [
        "# The island's settings. The Island window (Super+Shift+m) writes them; by hand, Super+Ctrl+r applies them.",
        "# Music and notifications",
        f"edge={config.place.edge}",
        f"align={config.place.align}",
        f"monitor={config.place.monitor}",
        f"media={on_off(config.media)}",
        f"notification_centre={on_off(config.notification_centre)}",
        f"peek_seconds={config.peek_seconds:g}",
        f"hover={on_off(config.hover)}",
        "# App launcher",
        f"launcher={on_off(config.launcher)}",
        f"launcher_edge={config.launcher_place.edge}",
        f"launcher_align={config.launcher_place.align}",
        f"launcher_monitor={config.launcher_place.monitor}",
        f"launcher_view={config.launcher_view}",
        f"launcher_size={config.launcher_size}",
        f"launcher_hover={on_off(config.launcher_hover)}",
        "# Both",
        f"color={config.color}",
    ]
    with open(PATH, "w") as conf:
        conf.write("\n".join(lines) + "\n")
