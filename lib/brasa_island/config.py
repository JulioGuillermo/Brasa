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
    config.hover = pairs.get("hover", "on").lower() not in ("off", "no", "false", "0")
    try:
        config.peek_seconds = float(pairs.get("peek_seconds", config.peek_seconds))
    except ValueError:
        pass
    if pairs.get("color", "").startswith("#"):
        config.color = pairs["color"]
    return config
