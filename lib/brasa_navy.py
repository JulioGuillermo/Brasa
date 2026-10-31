"""Brasa's night blue for the body of a folder icon, with its details red."""
import colorsys
import re

from brasa_redden import HEX_COLOR, parse_hex, redden_color, to_hex

NAVY_HUE = 225 / 360
NAVY_SATURATION = 0.62
BLUE_GREY_HUES = (190 / 360, 240 / 360)


def navy_color(digits):
    """The color as night blue, keeping how light or dark it was relative to the others."""
    _, lightness, _ = colorsys.rgb_to_hls(*parse_hex(digits))
    return to_hex(*colorsys.hls_to_rgb(NAVY_HUE, 0.06 + lightness * 0.3, NAVY_SATURATION))


def is_blue_grey(digits):
    hue, _, saturation = colorsys.rgb_to_hls(*parse_hex(digits))
    return BLUE_GREY_HUES[0] <= hue <= BLUE_GREY_HUES[1] and saturation < 0.45


def is_dark(digits):
    _, lightness, _ = colorsys.rgb_to_hls(*parse_hex(digits))
    return lightness < 0.55


def is_near_white(digits):
    _, lightness, saturation = colorsys.rgb_to_hls(*parse_hex(digits))
    return saturation < 0.12 and lightness > 0.8


def light_red(digits):
    _, lightness, _ = colorsys.rgb_to_hls(*parse_hex(digits))
    return to_hex(*colorsys.hls_to_rgb(0.0, 0.45 + lightness * 0.2, 1.0))


def slot_navy(text):
    """Slot's folders: the dark blue-grey front turns night blue, the colors behind and the light symbol red."""
    def recolor(match):
        digits = match.group(1)
        if is_blue_grey(digits) and is_dark(digits):
            return navy_color(digits)
        if is_blue_grey(digits) or is_near_white(digits):
            return light_red(digits)
        return redden_color(digits, darker=0.9, min_saturation=0.7)
    return HEX_COLOR.sub(recolor, text)


GRADIENT = re.compile(r'(<linearGradient\b[^>]*\bid="([^"]+)"[^>]*>)(.*?)(</linearGradient>)', re.S)
BODY_PATH = re.compile(r'<path\b[^>]*opacity="0\.5"[^>]*fill="url\(#([^)]+)\)"[^>]*/>')


def beauty_navy(text):
    """BeautySolar's folders: the half-transparent body turns solid night blue, the tab and symbol red."""
    body = BODY_PATH.search(text)
    body_gradient = body.group(1) if body else None
    if body:
        solid = body.group(0).replace('opacity="0.5" ', "", 1)
        text = text.replace(body.group(0), solid, 1)

    def recolor_gradient(match):
        opening, gradient_id, stops, closing = match.groups()
        if gradient_id == body_gradient:
            stops = HEX_COLOR.sub(lambda color: navy_color(color.group(1)), stops)
        else:
            stops = HEX_COLOR.sub(lambda color: redden_color(color.group(1), 0.95, 0.75), stops)
        return opening + stops + closing
    return GRADIENT.sub(recolor_gradient, text)
