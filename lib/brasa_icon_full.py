"""Brasa's full icon themes: a whole theme copied, every SVG icon in Brasa's colors."""
import os
import shutil
import subprocess
from dataclasses import dataclass
from typing import Callable

import brasa_ini
from brasa_icon_files import theme_path
from brasa_navy import beauty_brasa
from brasa_themes_found import USER_ICONS


@dataclass
class FullTheme:
    folder: str
    name: str
    source: str
    recolor: Callable[[str, bool], str]


FULL_THEMES = [
    FullTheme("Brasa-BeautySolar-All", "Brasa · BeautySolar all icons", "BeautySolar", beauty_brasa),
]


def is_symbolic(file_name):
    return "symbolic" in file_name


def svg_files(path):
    for folder, _, files in os.walk(path):
        for file_name in files:
            if file_name.endswith(".svg"):
                yield os.path.join(folder, file_name)


def unlink_symbolic_names(path):
    """A symbolic icon that is a link to a regular one becomes a copy of it, so it can be recolored as
    symbolic. Run before any recoloring: the copy is made from the theme's own picture."""
    for file in list(svg_files(path)):
        target = os.path.realpath(file)
        if not os.path.islink(file) or not os.path.isfile(target):
            continue
        if is_symbolic(os.path.basename(file)) and not is_symbolic(os.path.basename(target)):
            copy = file + ".brasa-copy"
            shutil.copyfile(target, copy)
            os.replace(copy, file)


def recolor_tree(path, recolor):
    unlink_symbolic_names(path)
    for file in svg_files(path):
        if os.path.islink(file):
            continue
        with open(file, encoding="utf-8", errors="replace") as svg:
            text = svg.read()
        with open(file, "w", encoding="utf-8") as svg:
            svg.write(recolor(text, is_symbolic(os.path.basename(file))))


def update_cache(path):
    if shutil.which("gtk-update-icon-cache"):
        subprocess.run(["gtk-update-icon-cache", "-q", "-f", "-t", path],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def build(theme):
    """Make the theme again from its source. False when the source theme is not installed."""
    source = theme_path(theme.source)
    if source is None:
        return False
    work = os.path.join(USER_ICONS, f".{theme.folder}.building")
    shutil.rmtree(work, ignore_errors=True)
    shutil.copytree(source, work, symlinks=True, ignore=shutil.ignore_patterns("icon-theme.cache"))
    recolor_tree(work, theme.recolor)
    index = os.path.join(work, "index.theme")
    brasa_ini.set_key(index, "Icon Theme", "Name", theme.name)
    brasa_ini.set_key(index, "Icon Theme", "Comment", f"{theme.source} in Brasa's colors (made by brasa-themes mix)")
    update_cache(work)
    final = os.path.join(USER_ICONS, theme.folder)
    shutil.rmtree(final, ignore_errors=True)
    os.rename(work, final)
    return True


def build_all(report):
    for theme in FULL_THEMES:
        if build(theme):
            report(f"Made {theme.folder}")
        else:
            report(f"Skipped {theme.folder}: {theme.source} is not installed")
