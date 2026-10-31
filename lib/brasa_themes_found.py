"""The icon and cursor themes installed on this machine, in every folder the desktop looks in."""
import configparser
import os
from dataclasses import dataclass

USER_ICONS = os.path.expanduser("~/.local/share/icons")
NOT_CHOOSABLE = {"default", "hicolor", "locolor"}


@dataclass
class Theme:
    folder: str
    name: str
    path: str


def icon_folders():
    data_dirs = os.environ.get("XDG_DATA_DIRS") or "/usr/local/share:/usr/share"
    system = [os.path.join(data_dir, "icons") for data_dir in data_dirs.split(":") if data_dir]
    return [USER_ICONS, os.path.expanduser("~/.icons")] + system


def read_index(path):
    index = configparser.ConfigParser(interpolation=None, strict=False)
    try:
        index.read(os.path.join(path, "index.theme"), encoding="utf-8")
    except (configparser.Error, UnicodeDecodeError):
        return None
    if not index.has_section("Icon Theme"):
        return None
    return index["Icon Theme"]


def has_icons(index):
    return bool(index.get("Directories", "").strip())


def has_cursors(path):
    return os.path.isdir(os.path.join(path, "cursors"))


def installed(kind):
    """Installed themes of a kind ("icons" or "cursors"), by name; a folder found twice counts once,
    the first place it is found wins, as for the desktop."""
    found = {}
    for icons_folder in icon_folders():
        if not os.path.isdir(icons_folder):
            continue
        for folder in os.listdir(icons_folder):
            path = os.path.join(icons_folder, folder)
            if folder in found or folder in NOT_CHOOSABLE or not os.path.isdir(path):
                continue
            index = read_index(path)
            if kind == "icons" and index is not None and has_icons(index):
                found[folder] = Theme(folder, index.get("Name", folder), path)
            if kind == "cursors" and has_cursors(path):
                name = index.get("Name", folder) if index is not None else folder
                found[folder] = Theme(folder, name, path)
    return sorted(found.values(), key=lambda theme: theme.name.lower())
