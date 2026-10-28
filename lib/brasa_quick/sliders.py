"""Volume and, with a backlight, screen brightness. While the panel is open they follow changes made
elsewhere (the keys, another program), except right after you moved them."""
import glob
import subprocess
import time

from gi.repository import Gtk

from brasa_island import volume

from . import glyphs

KEY_STEP = 0.05
USER_HOLD_SECONDS = 1.0


def screen_brightness() -> float:
    """Read from the first backlight, the one brightnessctl changes."""
    device = next(iter(sorted(glob.glob("/sys/class/backlight/*"))), None)
    try:
        with open(f"{device}/brightness") as current, open(f"{device}/max_brightness") as top:
            return int(current.read()) / int(top.read())
    except (TypeError, OSError, ValueError, ZeroDivisionError):
        return 0.0


def set_screen_brightness(level: float):
    subprocess.Popen(["brightnessctl", "-q", "set", f"{max(1, round(level * 100))}%"],
                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


class Slider(Gtk.Box):
    def __init__(self, icon, read, write):
        super().__init__(spacing=10, css_classes=["quick-slider"])
        self.read = read
        self.write = write
        self.moved_at = 0.0
        self.scale = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 0, 1, 0.01)
        self.scale.set_hexpand(True)
        self.scale.set_draw_value(False)
        self.scale.get_adjustment().set_step_increment(KEY_STEP)
        self.scale.get_adjustment().set_page_increment(KEY_STEP * 2)
        self.scale.connect("change-value", self.on_change)
        self.append(Gtk.Label(label=icon, css_classes=["quick-slider-icon"]))
        self.append(self.scale)

    def on_change(self, _scale, _scroll, value):
        self.moved_at = time.monotonic()
        self.write(min(max(value, 0), 1))
        return False

    def refresh(self):
        if time.monotonic() - self.moved_at < USER_HOLD_SECONDS:
            return
        self.scale.set_value(self.read())


def sliders(with_backlight: bool) -> list[Slider]:
    found = [Slider(glyphs.VOLUME, volume.read, volume.write)]
    if with_backlight:
        found.append(Slider(glyphs.BRIGHTNESS, screen_brightness, set_screen_brightness))
    return found
