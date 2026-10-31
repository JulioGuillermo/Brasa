"""Find the file of an icon in an icon theme by reading its index.theme, without GTK, so it can run
outside the main thread. Looks in the theme, then in the themes it inherits."""
import os

from brasa_themes_found import icon_folders, read_index

EXTENSIONS = (".svg", ".png")


def theme_path(folder):
    for icons_folder in icon_folders():
        path = os.path.join(icons_folder, folder)
        if os.path.isfile(os.path.join(path, "index.theme")):
            return path
    return None


def size_range(index_folder):
    """The smallest and largest size a folder's icons are drawn at, as the icon theme spec reads them."""
    size = int(index_folder.get("Size", "0"))
    kind = index_folder.get("Type", "Threshold").lower()
    if kind == "scalable":
        return int(index_folder.get("MinSize", size)), int(index_folder.get("MaxSize", size))
    if kind == "threshold":
        threshold = int(index_folder.get("Threshold", "2"))
        return size - threshold, size + threshold
    return size, size


def size_distance(index_folder, wanted_size):
    try:
        smallest, largest = size_range(index_folder)
    except ValueError:
        return 1000
    if wanted_size < smallest:
        return smallest - wanted_size
    if wanted_size > largest:
        return wanted_size - largest
    return 0


def fit(index_folder, wanted_size):
    """Sort key: closest size first, and between two as close, the larger one (it shrinks sharper)."""
    try:
        size = int(index_folder.get("Size", "0"))
    except ValueError:
        size = 0
    return size_distance(index_folder, wanted_size), -size


def folders_by_fit(index, wanted_size):
    """The theme's icon folders, best size first."""
    index_file = index.parser
    folders = [name.strip() for name in index.get("Directories", "").split(",") if name.strip()]
    fitting = [name for name in folders if index_file.has_section(name)]
    return sorted(fitting, key=lambda name: fit(index_file[name], wanted_size))


def find_in_theme(path, index, name, wanted_size):
    for folder in folders_by_fit(index, wanted_size):
        for extension in EXTENSIONS:
            file = os.path.join(path, folder, name + extension)
            if os.path.isfile(file):
                return file
    return None


def find_icon(folder, name, wanted_size):
    """The file of the icon, or None when neither the theme nor the ones it inherits have it."""
    pending = [folder]
    seen = set()
    while pending:
        current = pending.pop(0)
        if current in seen:
            continue
        seen.add(current)
        path = theme_path(current)
        index = read_index(path) if path else None
        if index is None:
            continue
        file = find_in_theme(path, index, name, wanted_size)
        if file:
            return file
        pending += [parent.strip() for parent in index.get("Inherits", "").split(",") if parent.strip()]
    return None
