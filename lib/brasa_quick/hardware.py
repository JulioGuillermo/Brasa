"""What this machine has, so the panel leaves out what it cannot use."""
import glob
import os
import shutil


def read(path: str) -> str:
    try:
        with open(path) as file:
            return file.read().strip()
    except OSError:
        return ""


def has_bluetooth() -> bool:
    return bool(glob.glob("/sys/class/bluetooth/*"))


def has_backlight() -> bool:
    return bool(glob.glob("/sys/class/backlight/*"))


def has_battery() -> bool:
    return bool(glob.glob("/sys/class/power_supply/BAT*"))


def has_charge_limit() -> bool:
    return bool(glob.glob("/sys/class/power_supply/BAT*/charge_control_end_threshold"))


def has_power_profiles() -> bool:
    return bool(shutil.which("powerprofilesctl"))


def has_hyprmod() -> bool:
    return bool(shutil.which("hyprmod"))


def in_hyprland() -> bool:
    return bool(os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"))


def keyboard_light_levels() -> list[tuple[int, str]]:
    """The keyboard backlight's levels as (value, name); none without one."""
    led = next(iter(glob.glob("/sys/class/leds/*kbd_backlight*")), None)
    top = int(read(f"{led}/max_brightness") or 0) if led else 0
    if not top:
        return []
    names = {1: ["On"], 2: ["Low", "High"], 3: ["Low", "Medium", "High"], 4: ["Low", "Medium", "High", "Max"]}
    if top in names:
        return [(0, "Off"), *zip(range(1, top + 1), names[top])]
    return [(round(top * percent / 100), "Off" if percent == 0 else f"{percent} %") for percent in (0, 25, 50, 75, 100)]


def battery_percent() -> str:
    level = read(next(iter(glob.glob("/sys/class/power_supply/BAT*/capacity")), ""))
    return f"{level} %" if level else ""
