"""Volume and, with a backlight, screen brightness."""
import subprocess

from gi.repository import Gtk

from brasa_island import volume

from . import glyphs

KEY_STEP = 0.05


def screen_brightness() -> float:
    """brightnessctl -m: device,class,current,percent%,max"""
    try:
        reply = subprocess.run(["brightnessctl", "-m"], capture_output=True, text=True, timeout=1).stdout
        return int(reply.split(",")[3].rstrip("%")) / 100
    except (OSError, subprocess.SubprocessError, IndexError, ValueError):
        return 0.0


def set_screen_brightness(level: float):
    subprocess.Popen(["brightnessctl", "-q", "set", f"{max(1, round(level * 100))}%"],
                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


class Slider(Gtk.Box):
    def __init__(self, icon, read, write):
        super().__init__(spacing=10, css_classes=["quick-slider"])
        self.read = read
        self.write = write
        self.scale = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 0, 1, 0.01)
        self.scale.set_hexpand(True)
        self.scale.set_draw_value(False)
        self.scale.get_adjustment().set_step_increment(KEY_STEP)
        self.scale.get_adjustment().set_page_increment(KEY_STEP * 2)
        self.scale.connect("change-value", self.on_change)
        self.append(Gtk.Label(label=icon, css_classes=["quick-slider-icon"]))
        self.append(self.scale)

    def on_change(self, _scale, _scroll, value):
        self.write(min(max(value, 0), 1))
        return False

    def refresh(self):
        self.scale.set_value(self.read())


def sliders(with_backlight: bool) -> list[Slider]:
    found = [Slider(glyphs.VOLUME, volume.read, volume.write)]
    if with_backlight:
        found.append(Slider(glyphs.BRIGHTNESS, screen_brightness, set_screen_brightness))
    return found
