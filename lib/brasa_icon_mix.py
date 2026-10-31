"""Brasa's own icon themes: Papirus-Dark for everything, with the folders of another theme, all of them
red. Each is a small theme in ~/.local/share/icons holding only folder icons; the rest it inherits."""
import configparser
import os
import re
import shutil
from dataclasses import dataclass
from typing import Callable

from brasa_icon_files import theme_path
from brasa_redden import redden_cyan_pictures, redden_svg
from brasa_themes_found import USER_ICONS, read_index

DIRECTORY_KEYS = ("Size", "Scale", "Context", "Type", "MinSize", "MaxSize", "Threshold")
FOLDER_ICON = re.compile(r"^(folder|user-home|user-desktop)")
PAPIRUS_BLUE = re.compile(r"^(folder|user)-blue(?=[-.])")


def any_folder(file_name):
    if "symbolic" in file_name or not FOLDER_ICON.match(file_name):
        return None
    return file_name


def papirus_blue_folder(file_name):
    """Papirus keeps each folder color apart (folder-blue-music.svg): the blue one, named as the plain icon."""
    if not PAPIRUS_BLUE.match(file_name):
        return None
    return PAPIRUS_BLUE.sub(r"\1", file_name)


def papirus_red(text):
    return redden_svg(text, darker=0.72, min_saturation=0.8)


def slot_red(text):
    return redden_svg(text, darker=0.9, min_saturation=0.7)


def beauty_red(text):
    return redden_svg(text, darker=0.95, min_saturation=0.75)


def lyra_red(text):
    return redden_cyan_pictures(redden_svg(text))


@dataclass
class Mix:
    folder: str
    name: str
    source: str
    pick: Callable[[str], str | None]
    recolor: Callable[[str], str]
    only_sizes: tuple[int, ...] = ()


BASE = "Brasa-Papirus"
MIXES = [
    Mix(BASE, "Brasa · Papirus", "Papirus-Dark", papirus_blue_folder, papirus_red),
    Mix("Brasa-Kanada", "Brasa · Kanada", "KanadaIcons", any_folder, redden_svg),
    Mix("Brasa-Baphomet", "Brasa · Baphomet", "Baphomet", any_folder, redden_svg),
    Mix("Brasa-Lyra", "Brasa · Lyra", "Lyra-dark", any_folder, lyra_red, only_sizes=(48,)),
    Mix("Brasa-Slot", "Brasa · Slot", "Slot-Symbolic-Dark-Icons", any_folder, slot_red),
    Mix("Brasa-BeautySolar", "Brasa · BeautySolar", "BeautySolar", any_folder, beauty_red),
]


def inherits(mix):
    if mix.folder == BASE:
        return "Papirus-Dark,breeze-dark,hicolor"
    return f"{BASE},Papirus-Dark,breeze-dark,hicolor"


def folder_directories(path, index, only_sizes):
    """The source theme's directories of colored folder icons."""
    parser = index.parser
    for directory in index.get("Directories", "").split(","):
        directory = directory.strip()
        if not parser.has_section(directory) or "symbolic" in directory:
            continue
        if not os.path.isdir(os.path.join(path, directory)):
            continue
        section = parser[directory]
        if section.get("Context", "").lower() != "places":
            continue
        if only_sizes and int(section.get("Size", "0")) not in only_sizes:
            continue
        yield directory, section


def copy_icons(mix, source_dir, target_dir):
    """The picked icons of one directory, recolored; how many."""
    copied = 0
    for file_name in sorted(os.listdir(source_dir)):
        new_name = mix.pick(file_name)
        source = os.path.join(source_dir, file_name)
        if not new_name or not os.path.isfile(source):
            continue
        os.makedirs(target_dir, exist_ok=True)
        target = os.path.join(target_dir, new_name)
        if file_name.endswith(".svg"):
            with open(source, encoding="utf-8", errors="replace") as file:
                text = mix.recolor(file.read())
            with open(target, "w", encoding="utf-8") as file:
                file.write(text)
        else:
            shutil.copyfile(source, target)
        copied += 1
    return copied


def write_index(mix, path, directories):
    index = configparser.ConfigParser(interpolation=None)
    index.optionxform = str
    index["Icon Theme"] = {
        "Name": mix.name,
        "Comment": f"Papirus-Dark with the folders of {mix.source}, in Brasa's red (made by brasa-themes mix)",
        "Inherits": inherits(mix),
        "Directories": ",".join(name for name, _ in directories),
    }
    for name, section in directories:
        index[name] = {key: section[key] for key in DIRECTORY_KEYS if key in section}
    with open(os.path.join(path, "index.theme"), "w", encoding="utf-8") as file:
        index.write(file, space_around_delimiters=False)


def build(mix):
    """Make the theme again from its source. False when the source theme is not installed."""
    source = theme_path(mix.source)
    index = read_index(source) if source else None
    if index is None:
        return False
    work = os.path.join(USER_ICONS, f".{mix.folder}.building")
    shutil.rmtree(work, ignore_errors=True)
    os.makedirs(work)
    written = []
    for directory, section in folder_directories(source, index, mix.only_sizes):
        if copy_icons(mix, os.path.join(source, directory), os.path.join(work, directory)):
            written.append((directory, section))
    write_index(mix, work, written)
    final = os.path.join(USER_ICONS, mix.folder)
    shutil.rmtree(final, ignore_errors=True)
    os.rename(work, final)
    return True


def build_all(report):
    """Every mix whose source theme is installed; report(text) says which were made or skipped."""
    for mix in MIXES:
        if build(mix):
            report(f"Made {mix.folder}")
        else:
            report(f"Skipped {mix.folder}: {mix.source} is not installed")
