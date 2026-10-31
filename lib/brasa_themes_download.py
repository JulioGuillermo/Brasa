"""Download the icon and cursor themes of data/themes.tsv from gnome-look.org into ~/.local/share/icons."""
import json
import os
import re
import shutil
import tarfile
import tempfile
import urllib.parse
import urllib.request
import zipfile
from dataclasses import dataclass

from brasa_themes_found import USER_ICONS, icon_folders

BRASA = os.path.realpath(os.path.expanduser("~/.config/brasa"))
CATALOGUE = os.path.join(BRASA, "data", "themes.tsv")
FILES_URL = "https://www.gnome-look.org/p/{}/loadFiles"
ARCHIVE = re.compile(r"\.(tar\.gz|tgz|tar\.xz|tar\.bz2|zip)$")


@dataclass
class CatalogueTheme:
    kind: str
    product: str
    folder: str


def catalogue():
    themes = []
    with open(CATALOGUE, encoding="utf-8") as file:
        for line in file:
            if line.startswith("#") or not line.strip():
                continue
            kind, product, folder = line.rstrip("\n").split("\t")
            themes.append(CatalogueTheme(kind, product, folder))
    return themes


def is_installed(folder):
    return any(os.path.isdir(os.path.join(icons, folder)) for icons in icon_folders())


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "brasa-themes"})
    return urllib.request.urlopen(request, timeout=60)


def archive_url(product):
    """The download link of the product's Linux archive (the links expire: ask for them each time)."""
    with fetch(FILES_URL.format(product)) as response:
        files = json.load(response)["files"]
    for entry in files:
        name = entry["name"]
        if entry["active"] and ARCHIVE.search(name) and "windows" not in name.lower():
            return urllib.parse.unquote(entry["url"]), name
    raise LookupError(f"gnome-look product {product} has no Linux archive")


def unpack(archive, name, into):
    if name.endswith(".zip"):
        with zipfile.ZipFile(archive) as packed:
            for member in packed.namelist():
                target = os.path.realpath(os.path.join(into, member))
                if not target.startswith(os.path.realpath(into) + os.sep):
                    raise ValueError(f"{name} holds a file outside its folder: {member}")
            packed.extractall(into)
    else:
        with tarfile.open(archive) as packed:
            packed.extractall(into, filter="data")


def download(theme):
    """Download and unpack one theme; its folder ends up in ~/.local/share/icons."""
    url, name = archive_url(theme.product)
    os.makedirs(USER_ICONS, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=USER_ICONS, prefix=".brasa-download-") as work:
        archive = os.path.join(work, "archive")
        with fetch(url) as response, open(archive, "wb") as file:
            shutil.copyfileobj(response, file)
        unpacked = os.path.join(work, "unpacked")
        unpack(archive, name, unpacked)
        source = os.path.join(unpacked, theme.folder)
        if not os.path.isfile(os.path.join(source, "index.theme")):
            raise LookupError(f"{name} has no {theme.folder}/index.theme")
        os.rename(source, os.path.join(USER_ICONS, theme.folder))


def download_missing(report):
    """Download every theme of the catalogue not installed yet; report(text) says how each goes.
    Returns the number that failed."""
    failed = 0
    for theme in catalogue():
        if is_installed(theme.folder):
            continue
        report(f"Downloading {theme.folder} ({theme.kind})")
        try:
            download(theme)
        except (OSError, ValueError, LookupError, KeyError, tarfile.TarError, zipfile.BadZipFile) as error:
            report(f"Could not download {theme.folder}: {error}")
            failed += 1
    return failed
