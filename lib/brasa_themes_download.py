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


@dataclass
class CatalogueTheme:
    kind: str
    product: str
    file_pattern: re.Pattern
    folders: list[str]


def catalogue():
    themes = []
    with open(CATALOGUE, encoding="utf-8") as file:
        for line in file:
            if line.startswith("#") or not line.strip():
                continue
            kind, product, file_pattern, folders = line.rstrip("\n").split("\t")
            themes.append(CatalogueTheme(kind, product, re.compile(file_pattern), folders.split(",")))
    return themes


def is_installed(folder):
    return any(os.path.isdir(os.path.join(icons, folder)) for icons in icon_folders())


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "brasa-themes"})
    return urllib.request.urlopen(request, timeout=120)


def archive_url(theme):
    """The download link of the newest archive whose name matches (the links expire: ask each time)."""
    with fetch(FILES_URL.format(theme.product)) as response:
        files = json.load(response)["files"]
    matching = [entry for entry in files if entry["active"] and theme.file_pattern.fullmatch(entry["name"])]
    if not matching:
        raise LookupError(f"gnome-look product {theme.product} has no file like {theme.file_pattern.pattern}")
    newest = max(matching, key=lambda entry: entry.get("updated_timestamp") or "")
    return urllib.parse.unquote(newest["url"])


def safe_member(member, destination):
    """Links to absolute paths are left out (some themes point into /usr/share/icons/breeze-dark, which
    they inherit anyway); everything else as tarfile's data filter allows."""
    if member.issym() and member.linkname.startswith("/"):
        return None
    return tarfile.data_filter(member, destination)


def unpack(archive, into):
    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as packed:
            for member in packed.namelist():
                target = os.path.realpath(os.path.join(into, member))
                if not target.startswith(os.path.realpath(into) + os.sep):
                    raise ValueError(f"the archive holds a file outside its folder: {member}")
            packed.extractall(into)
        return
    with tarfile.open(archive) as packed:
        packed.extractall(into, filter=safe_member)


def download(theme):
    """Download and unpack one archive; its theme folders end up in ~/.local/share/icons."""
    url = archive_url(theme)
    os.makedirs(USER_ICONS, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=USER_ICONS, prefix=".brasa-download-") as work:
        archive = os.path.join(work, "archive")
        with fetch(url) as response, open(archive, "wb") as file:
            shutil.copyfileobj(response, file)
        unpacked = os.path.join(work, "unpacked")
        unpack(archive, unpacked)
        for folder in theme.folders:
            if not os.path.isfile(os.path.join(unpacked, folder, "index.theme")):
                raise LookupError(f"the archive has no {folder}/index.theme")
        for folder in theme.folders:
            if not is_installed(folder):
                os.rename(os.path.join(unpacked, folder), os.path.join(USER_ICONS, folder))


def download_missing(report):
    """Download every theme of the catalogue not installed yet; report(text) says how each goes.
    Returns the number that failed."""
    failed = 0
    for theme in catalogue():
        if all(is_installed(folder) for folder in theme.folders):
            continue
        report(f"Downloading {', '.join(theme.folders)} ({theme.kind})")
        try:
            download(theme)
        except (OSError, ValueError, LookupError, KeyError, tarfile.TarError, zipfile.BadZipFile) as error:
            report(f"Could not download {', '.join(theme.folders)}: {error}")
            failed += 1
    return failed
