"""Turn the colors of an SVG icon red, Brasa's red: every colored hex color takes the red hue; greys,
black and white stay as they are."""
import colorsys
import re

HEX_COLOR = re.compile(r"(?<![\w(])#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})(?![\w-])")
GREY_SATURATION = 0.12
RED_HUE = 0.0


def parse_hex(digits):
    if len(digits) == 3:
        digits = "".join(digit * 2 for digit in digits)
    return [int(digits[start:start + 2], 16) / 255 for start in (0, 2, 4)]


def to_hex(red, green, blue):
    return "#" + "".join(f"{round(channel * 255):02x}" for channel in (red, green, blue))


def redden_color(digits, darker, min_saturation):
    """darker: how much of its lightness the color keeps; min_saturation: how vivid it ends at least."""
    hue, lightness, saturation = colorsys.rgb_to_hls(*parse_hex(digits))
    if saturation < GREY_SATURATION or lightness < 0.03 or lightness > 0.97:
        return "#" + digits
    return to_hex(*colorsys.hls_to_rgb(RED_HUE, lightness * darker, max(saturation, min_saturation)))


def redden_svg(text, darker=1.0, min_saturation=0.0):
    return HEX_COLOR.sub(lambda match: redden_color(match.group(1), darker, min_saturation), text)


CYAN_TO_RED = "0 1 0 0 0  1 0 0 0 0  1 0 0 0 0  0 0 0 1 0"
FILTER_ID = "brasa-cyan-to-red"


def redden_cyan_pictures(text):
    """Pictures embedded in the SVG (PNG glows) get a filter that turns their cyan into red."""
    if "<image" not in text:
        return text
    definition = (f'<filter id="{FILTER_ID}" color-interpolation-filters="sRGB">'
                  f'<feColorMatrix type="matrix" values="{CYAN_TO_RED}"/></filter>')
    text = re.sub(r"(<svg\b[^>]*>)", lambda match: match.group(1) + f"<defs>{definition}</defs>", text, count=1)
    return text.replace("<image ", f'<image filter="url(#{FILTER_ID})" ')
