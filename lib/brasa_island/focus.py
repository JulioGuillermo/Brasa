"""Which monitor has the focus, asked to the window manager (Hyprland or sway)."""
import json
import os
import subprocess

ASK_TIMEOUT = 1


def ask(command) -> list:
    try:
        reply = subprocess.run(command, capture_output=True, text=True, timeout=ASK_TIMEOUT).stdout
        return json.loads(reply)
    except (OSError, subprocess.SubprocessError, ValueError):
        return []


def focused_connector() -> str:
    if os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"):
        monitors = ask(["hyprctl", "monitors", "-j"])
    elif os.environ.get("SWAYSOCK"):
        monitors = ask(["swaymsg", "-r", "-t", "get_outputs"])
    else:
        return ""
    return next((monitor["name"] for monitor in monitors if monitor.get("focused")), "")
