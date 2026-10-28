"""The panel's outline: a body sitting in a bottom corner of the screen, joined to the side edge and to
the bottom edge by long, smooth shoulders, like the island but on two edges. Drawn in the corner's own
frame: x grows away from the side edge, y grows up from the bottom edge. Black along both edges, the
island's colour towards the free corner, a thin red rim."""
from dataclasses import dataclass

import cairo
from gi.repository import Gdk

SHOULDER_FLATNESS = 0.5
SHOULDER_DROP = 0.5
CIRCLE_HANDLE = 0.552
EDGE_SHADE = 0.6
RIM_RED = (1.0, 0.0, 0.0, 0.55)
RIM_WIDTH = 1.2
RIM_FADE = 10


@dataclass
class Corner:
    width: float
    height: float
    shoulder: float
    radius: float

    def holds(self, x: float, y: float) -> bool:
        """Whether a point of the corner's frame is on the body."""
        return 0 <= x <= self.width and 0 <= y <= self.height


def to_corner_frame(cr, screen_width: float, screen_height: float, on_right: bool):
    if on_right:
        cr.translate(screen_width, screen_height)
        cr.scale(-1, -1)
    else:
        cr.translate(0, screen_height)
        cr.scale(1, -1)


def trace(cr, corner: Corner):
    width, height = max(corner.width, 0.0), max(corner.height, 0.0)
    radius = min(corner.radius, width / 2, height / 2)
    shoulder = max(corner.shoulder, 0.0)
    side_drop = min(shoulder * 0.75, width - radius)
    bottom_drop = min(shoulder * 0.75, height - radius)
    handle = radius * CIRCLE_HANDLE

    cr.new_path()
    cr.move_to(-1, height + shoulder)
    cr.line_to(0, height + shoulder)
    cr.curve_to(0, height + shoulder * (1 - SHOULDER_FLATNESS),
                side_drop * SHOULDER_DROP, height, side_drop, height)
    cr.line_to(width - radius, height)
    cr.curve_to(width - radius + handle, height, width, height - radius + handle, width, height - radius)
    cr.line_to(width, bottom_drop)
    cr.curve_to(width, bottom_drop * SHOULDER_DROP,
                width + shoulder * (1 - SHOULDER_FLATNESS), 0, width + shoulder, 0)
    cr.line_to(width + shoulder, -1)
    cr.line_to(-1, -1)
    cr.close_path()


def edge_fade(length: float, horizontal: bool, alpha_at_edge: float) -> cairo.LinearGradient:
    fade = cairo.LinearGradient(0, 0, length, 0) if horizontal else cairo.LinearGradient(0, 0, 0, length)
    fade.add_color_stop_rgba(0, 0, 0, 0, alpha_at_edge)
    fade.add_color_stop_rgba(1, 0, 0, 0, 0)
    return fade


def paint(cr, corner: Corner, color: Gdk.RGBA):
    if corner.width < 0.5 or corner.height < 0.5:
        return
    trace(cr, corner)
    cr.save()
    cr.clip_preserve()
    cr.set_source_rgb(color.red, color.green, color.blue)
    cr.fill_preserve()
    cr.set_source(edge_fade(corner.width * EDGE_SHADE, True, 1))
    cr.fill_preserve()
    cr.set_source(edge_fade(corner.height * EDGE_SHADE, False, 1))
    cr.fill()
    cr.restore()

    cr.push_group()
    trace(cr, corner)
    cr.set_source_rgba(*RIM_RED)
    cr.set_line_width(RIM_WIDTH)
    cr.stroke()
    cr.set_operator(cairo.OPERATOR_DEST_OUT)
    cr.set_source(edge_fade(RIM_FADE, True, 1))
    cr.paint()
    cr.set_source(edge_fade(RIM_FADE, False, 1))
    cr.paint()
    cr.pop_group_to_source()
    cr.paint()
