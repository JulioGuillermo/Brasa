"""The island's settings, from ~/.config/brasa/island.conf (KEY=value lines, # for comments)."""
import os
import re
from dataclasses import dataclass

PATH = os.path.expanduser("~/.config/brasa/island.conf")


@dataclass
class Config:
    edge: str = "top"
    align: str = "center"
    monitor: str = ""
    hover: bool = True
    media: bool = True
    notifications: bool = True
    notification_centre: bool = True
    peek_seconds: float = 2.5
    color: str = "#0c1838"


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


def load() -> Config:
    config = Config()
    if not os.path.exists(PATH):
        return config
    pairs = read_pairs(PATH)
    if pairs.get("edge") in ("top", "bottom"):
        config.edge = pairs["edge"]
    if pairs.get("align") in ("left", "center", "right"):
        config.align = pairs["align"]
    config.monitor = pairs.get("monitor", config.monitor)
    config.hover = switched_on(pairs.get("hover", "on"))
    config.media = switched_on(pairs.get("media", "on"))
    config.notifications = switched_on(pairs.get("notifications", "on"))
    config.notification_centre = switched_on(pairs.get("notification_centre", "on"))
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
        f"edge={config.edge}",
        f"align={config.align}",
        f"monitor={config.monitor}",
        f"hover={on_off(config.hover)}",
        f"media={on_off(config.media)}",
        f"notifications={on_off(config.notifications)}",
        f"notification_centre={on_off(config.notification_centre)}",
        f"peek_seconds={config.peek_seconds:g}",
        f"color={config.color}",
    ]
    with open(PATH, "w") as conf:
        conf.write("\n".join(lines) + "\n")
