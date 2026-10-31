"""Recolor the gradients of an SVG icon into one of Brasa's color ramps and keep them gradients: the
darkest stop of each takes the dark end of the ramp, the lightest the light end, the others in between."""
import re
from dataclasses import dataclass

from brasa_redden import HEX_COLOR, parse_hex, to_hex

GRADIENT = re.compile(r"(<(linear|radial)Gradient\b[^>]*\bid=\"([^\"]+)\"[^>]*(?<!/)>)(.*?)(</\2Gradient>)", re.S)
STOP = re.compile(r"<stop\b[^>]*>")
STOP_COLOR = re.compile(r"(stop-color\s*[:=]\s*\"?)(#[0-9a-fA-F]{3,6}|white|black)\b")
NAMED = {"white": "ffffff", "black": "000000"}


@dataclass(frozen=True)
class Ramp:
    dark: str
    light: str

    def at(self, position):
        dark = parse_hex(self.dark.lstrip("#"))
        light = parse_hex(self.light.lstrip("#"))
        return to_hex(*(start + (end - start) * position for start, end in zip(dark, light)))


RED = Ramp("#9a0018", "#ff5a3a")
NAVY = Ramp("#0a1a45", "#2c4fa8")
LIGHT_RED = Ramp("#ff4a4a", "#ffb0a8")


def stop_digits(stop):
    match = STOP_COLOR.search(stop)
    if not match:
        return None
    value = match.group(2).lower()
    return NAMED.get(value, value.lstrip("#"))


def lightness(digits):
    return sum(parse_hex(digits)) / 3


def positions(colors):
    """Where on the ramp each stop goes: by how light it is among the gradient's stops."""
    lights = [lightness(digits) for digits in colors]
    darkest, lightest = min(lights), max(lights)
    if lightest - darkest < 0.02:
        count = len(colors)
        return [index / (count - 1) if count > 1 else 0.5 for index in range(count)]
    return [(light - darkest) / (lightest - darkest) for light in lights]


def recolor_stops(block, ramp):
    stops = STOP.findall(block)
    colors = [stop_digits(stop) for stop in stops]
    known = [digits for digits in colors if digits]
    if not known:
        return block
    places = iter(positions(known))
    for stop, digits in zip(stops, colors):
        if not digits:
            continue
        new_color = ramp.at(next(places))
        block = block.replace(stop, STOP_COLOR.sub(lambda match: match.group(1) + new_color, stop), 1)
    return block


def gradient_colors(block):
    return [digits for digits in (stop_digits(stop) for stop in STOP.findall(block)) if digits]


def recolor_gradients(text, choose_ramp):
    """choose_ramp(gradient id, its stop colors as hex digits) gives the Ramp, or None to leave it."""
    def recolor(match):
        opening, _, gradient_id, stops, closing = match.groups()
        ramp = choose_ramp(gradient_id, gradient_colors(stops))
        if ramp is None:
            return match.group(0)
        return opening + recolor_stops(stops, ramp) + closing
    return GRADIENT.sub(recolor, text)


def outside_gradients(text, recolor_flat):
    """Apply recolor_flat to every hex color that is not inside a gradient."""
    pieces = []
    last = 0
    for match in GRADIENT.finditer(text):
        pieces.append(HEX_COLOR.sub(recolor_flat, text[last:match.start()]))
        pieces.append(match.group(0))
        last = match.end()
    pieces.append(HEX_COLOR.sub(recolor_flat, text[last:]))
    return "".join(pieces)
