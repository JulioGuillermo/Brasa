"""The chosen icon and cursor themes (appearance.conf), and putting them to use: GTK, Qt, the window
manager's cursor and the apps it starts. Also the base the Brasa · icon themes take their other icons from."""
import json
import os
import shutil
import subprocess

import brasa_ini
import brasa_icon_full
import brasa_icon_mix
from brasa_themes_found import installed

BRASA = os.path.realpath(os.path.expanduser("~/.config/brasa"))
APPEARANCE_CONF = os.path.join(BRASA, "appearance.conf")
HEADER = "# Icon and cursor themes: Brasa settings, Appearance page (or brasa-themes icons|cursor|base NAME)"
DEFAULTS = {"icons": "Brasa-Slot-Navy", "cursor": "Fuchsia-Red", "icon_base": "Brasa-BeautySolar-All"}
CURSOR_SIZE = 24
STATE = os.path.join(os.environ.get("XDG_STATE_HOME") or os.path.expanduser("~/.local/state"), "brasa")
HYPRLAND_CURSOR = os.path.join(STATE, "cursor.lua")
GTK_SETTINGS = [os.path.expanduser(f"~/.config/gtk-{version}/settings.ini") for version in ("3.0", "4.0")]
QT_SETTINGS = [os.path.expanduser(f"~/.config/{tool}/{tool}.conf") for tool in ("qt5ct", "qt6ct")]
XCURSOR_DEFAULT = os.path.expanduser("~/.icons/default/index.theme")


def load():
    """{"icons": theme, "cursor": theme, "icon_base": theme}; a missing one is the default."""
    chosen = dict(DEFAULTS)
    try:
        with open(APPEARANCE_CONF, encoding="utf-8") as file:
            lines = file.read().splitlines()
    except FileNotFoundError:
        return chosen
    for line in lines:
        key, equals, value = line.partition("=")
        if equals and key.strip() in DEFAULTS:
            chosen[key.strip()] = value.strip()
    return chosen


def save(chosen):
    lines = [HEADER] + [f"{key}={chosen[key]}" for key in DEFAULTS]
    with open(APPEARANCE_CONF, "w", encoding="utf-8") as file:
        file.write("\n".join(lines) + "\n")


def is_installed(kind, folder):
    return any(theme.folder == folder for theme in installed(kind))


def quiet(command):
    subprocess.run(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def gsettings(key, value):
    if shutil.which("gsettings"):
        quiet(["gsettings", "set", "org.gnome.desktop.interface", key, value])


def lua_string(text):
    return json.dumps(text, ensure_ascii=False)


def use_icons(theme):
    gsettings("icon-theme", theme)
    for path in GTK_SETTINGS:
        brasa_ini.set_key(path, "Settings", "gtk-icon-theme-name", theme)
    for path in QT_SETTINGS:
        brasa_ini.set_key(path, "Appearance", "icon_theme", theme)


def use_cursor(theme):
    gsettings("cursor-theme", theme)
    gsettings("cursor-size", str(CURSOR_SIZE))
    for path in GTK_SETTINGS:
        brasa_ini.set_key(path, "Settings", "gtk-cursor-theme-name", theme)
    brasa_ini.set_key(XCURSOR_DEFAULT, "Icon Theme", "Inherits", theme)
    os.makedirs(STATE, exist_ok=True)
    with open(HYPRLAND_CURSOR, "w", encoding="utf-8") as file:
        file.write("-- Written by brasa-themes: your cursor theme\n"
                   f'hl.env("XCURSOR_THEME", {lua_string(theme)})\n')
    if os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"):
        quiet(["hyprctl", "setcursor", theme, str(CURSOR_SIZE)])
        quiet(["hyprctl", "eval", f'hl.env("XCURSOR_THEME", {lua_string(theme)})'])
    elif os.environ.get("SWAYSOCK"):
        quiet(["swaymsg", f'seat seat0 xcursor_theme "{theme}" {CURSOR_SIZE}'])


def choose(kind, theme):
    """Save the theme as the one of its kind ("icons" or "cursor") and use it now."""
    chosen = load()
    chosen[kind] = theme
    save(chosen)
    if kind == "icons":
        use_icons(theme)
    else:
        use_cursor(theme)


def choose_icon_base(theme):
    """Save the base of the Brasa · icon themes, point them to it and show the icons again."""
    chosen = load()
    chosen["icon_base"] = theme
    save(chosen)
    brasa_icon_mix.rebase(theme)
    gsettings("icon-theme", "hicolor")
    use_icons(chosen["icons"])


def make_brasa_icons(report):
    """Brasa's own icon themes, from the themes installed."""
    brasa_icon_mix.build_all(report, load()["icon_base"])
    brasa_icon_full.build_all(report)


def apply_saved():
    """Use the chosen themes again, each only if installed, and write appearance.conf if missing."""
    chosen = load()
    save(chosen)
    if is_installed("icons", chosen["icons"]):
        use_icons(chosen["icons"])
    if is_installed("cursors", chosen["cursor"]):
        use_cursor(chosen["cursor"])
