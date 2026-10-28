"""The island's outline: a body hanging from the screen edge (y = 0), joined to it by long, smooth
shoulders so it looks like the edge itself bending into the screen. Black at the edge, Brasa blue at the far end."""
from dataclasses import dataclass

import cairo
from gi.repository import Gdk

SHOULDER_FLATNESS = 0.5
SHOULDER_DROP = 0.5
CIRCLE_HANDLE = 0.552
RIM_RED = (1.0, 0.0, 0.0, 0.55)
RIM_WIDTH = 1.2
RIM_FADE = 10


@dataclass
class Outline:
    center: float
    width: float
    height: float
    shoulder: float
    radius: float

    @property
    def left(self) -> float:
        return self.center - self.width / 2 - self.shoulder

    @property
    def full_width(self) -> float:
        return self.width + 2 * self.shoulder


def trace(cr, outline: Outline):
    height = max(outline.height, 0.0)
    radius = min(outline.radius, height / 2, outline.width / 2)
    shoulder = max(outline.shoulder, 0.0)
    drop = min(shoulder * 0.75, height - radius)
    left = outline.center - outline.width / 2
    right = outline.center + outline.width / 2
    handle = radius * CIRCLE_HANDLE

    cr.new_path()
    cr.move_to(left - shoulder, -1)
    cr.line_to(left - shoulder, 0)
    cr.curve_to(left - shoulder * (1 - SHOULDER_FLATNESS), 0, left, drop * SHOULDER_DROP, left, drop)
    cr.line_to(left, height - radius)
    cr.curve_to(left, height - radius + handle, left + radius - handle, height, left + radius, height)
    cr.line_to(right - radius, height)
    cr.curve_to(right - radius + handle, height, right, height - radius + handle, right, height - radius)
    cr.line_to(right, drop)
    cr.curve_to(right, drop * SHOULDER_DROP, right + shoulder * (1 - SHOULDER_FLATNESS), 0, right + shoulder, 0)
    cr.line_to(right + shoulder, -1)
    cr.close_path()


def paint(cr, outline: Outline, color: Gdk.RGBA):
    if outline.height < 0.5:
        return
    trace(cr, outline)

    fill = cairo.LinearGradient(0, 0, 0, outline.height)
    fill.add_color_stop_rgb(0, 0, 0, 0)
    fill.add_color_stop_rgb(1, color.red, color.green, color.blue)
    cr.set_source(fill)
    cr.fill_preserve()

    rim = cairo.LinearGradient(0, 0, 0, RIM_FADE)
    rim.add_color_stop_rgba(0, *RIM_RED[:3], 0)
    rim.add_color_stop_rgba(1, *RIM_RED)
    cr.set_source(rim)
    cr.set_line_width(RIM_WIDTH)
    cr.stroke()
