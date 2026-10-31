"""Brasa's night blue for the body of an icon, with its details red, gradients kept."""
import colorsys
import re

from brasa_gradients import LIGHT_RED, NAVY, RED, outside_gradients, recolor_gradients
from brasa_redden import parse_hex, redden_color, to_hex

NAVY_HUE = 225 / 360
BLUE_GREY_HUES = (190 / 360, 240 / 360)
GREY_SATURATION = 0.12


def hls(digits):
    return colorsys.rgb_to_hls(*parse_hex(digits))


def is_grey(digits):
    return hls(digits)[2] < GREY_SATURATION


def is_blue_grey(digits):
    hue, _, saturation = hls(digits)
    return BLUE_GREY_HUES[0] <= hue <= BLUE_GREY_HUES[1] and saturation < 0.45


def is_dark(digits):
    return hls(digits)[1] < 0.55


def is_near_white(digits):
    return is_grey(digits) and hls(digits)[1] > 0.8


def navy_flat(digits):
    return to_hex(*colorsys.hls_to_rgb(NAVY_HUE, 0.06 + hls(digits)[1] * 0.3, 0.62))


def light_red_flat(digits):
    return to_hex(*colorsys.hls_to_rgb(0.0, 0.45 + hls(digits)[1] * 0.2, 1.0))


def slot_ramp(_gradient_id, colors):
    if all(is_grey(digits) for digits in colors):
        return None
    if all(is_blue_grey(digits) or is_grey(digits) for digits in colors):
        return NAVY if all(is_dark(digits) for digits in colors) else LIGHT_RED
    return RED


def slot_flat(match):
    digits = match.group(1)
    if is_blue_grey(digits) and is_dark(digits):
        return navy_flat(digits)
    if is_blue_grey(digits) or is_near_white(digits):
        return light_red_flat(digits)
    return redden_color(digits, darker=0.9, min_saturation=0.7)


def slot_navy(text):
    """Slot's folders: the blue-grey front night blue, the colors behind and the light symbol red."""
    return recolor_gradients(outside_gradients(text, slot_flat), slot_ramp)


TRANSLUCENT_PATH = re.compile(r'<path\b[^>]*(?<![\w-])opacity="0?\.\d+"[^>]*\bfill="url\(#([^)]+)\)"[^>]*>')
OPACITY = re.compile(r'\s(?<![\w-])opacity="0?\.\d+"')


def flat_red(match):
    return redden_color(match.group(1), darker=0.95, min_saturation=0.75)


def beauty_brasa(text, symbolic=False):
    """BeautySolar: the translucent part of an icon turns solid night blue, everything on it red. A
    symbolic icon keeps its transparency, in red: GTK paints it in the text color, and only the
    transparency tells its two tones apart."""
    body_gradients = set()
    for path in TRANSLUCENT_PATH.finditer(text):
        if symbolic:
            continue
        body_gradients.add(path.group(1))
        text = text.replace(path.group(0), OPACITY.sub("", path.group(0), count=1), 1)

    def ramp(gradient_id, colors):
        if all(is_grey(digits) for digits in colors):
            return None
        return NAVY if gradient_id in body_gradients else RED
    return recolor_gradients(outside_gradients(text, flat_red), ramp)
