"""A picture an app sends as raw pixels (the image-data hint of a notification)."""
from gi.repository import GdkPixbuf, GLib


def pixbuf_from_data(data: GLib.Variant) -> GdkPixbuf.Pixbuf:
    """data: (iiibiiay) width, height, rowstride, has alpha, bits per sample, channels, pixels."""
    width = data.get_child_value(0).get_int32()
    height = data.get_child_value(1).get_int32()
    rowstride = data.get_child_value(2).get_int32()
    has_alpha = data.get_child_value(3).get_boolean()
    bits = data.get_child_value(4).get_int32()
    pixels = data.get_child_value(6).get_data_as_bytes()
    return GdkPixbuf.Pixbuf.new_from_bytes(pixels, GdkPixbuf.Colorspace.RGB, has_alpha, bits,
                                           width, height, rowstride)
