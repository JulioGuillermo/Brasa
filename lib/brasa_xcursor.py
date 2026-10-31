"""Read one picture of an X cursor file (the format of cursor themes' cursors/ folder)."""
import struct
from dataclasses import dataclass

MAGIC = b"Xcur"
IMAGE = 0xFFFD0002


@dataclass
class CursorImage:
    width: int
    height: int
    pixels: bytes


def read_image(path, wanted_size):
    """The picture whose nominal size is closest to wanted_size; pixels are premultiplied BGRA bytes.
    None when the file is not a cursor."""
    try:
        with open(path, "rb") as file:
            data = file.read()
    except OSError:
        return None
    if data[:4] != MAGIC:
        return None
    header_size, _version, entries = struct.unpack_from("<III", data, 4)
    images = []
    for index in range(entries):
        kind, size, position = struct.unpack_from("<III", data, header_size + index * 12)
        if kind == IMAGE:
            images.append((abs(size - wanted_size), position))
    if not images:
        return None
    _, position = min(images)
    width, height = struct.unpack_from("<II", data, position + 16)
    start = position + 36
    return CursorImage(width, height, data[start:start + width * height * 4])
