"""A row that records a key combination: press its button, then the keys. While it listens, the window
manager's own bindings are paused (brasa-wm keys-capture), so Super+... reaches the window too."""
import os
import subprocess

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gdk, Gtk

BRASA_WM = os.path.expanduser("~/.local/bin/brasa-wm")
MASKS = [(Gdk.ModifierType.SUPER_MASK, "SUPER"), (Gdk.ModifierType.CONTROL_MASK, "CTRL"),
         (Gdk.ModifierType.ALT_MASK, "ALT"), (Gdk.ModifierType.SHIFT_MASK, "SHIFT")]
ACCELERATOR_MODS = {"SUPER": "<Super>", "CTRL": "<Control>", "ALT": "<Alt>", "SHIFT": "<Shift>"}
MODIFIER_KEYS = {"Super_L", "Super_R", "Control_L", "Control_R", "Alt_L", "Alt_R", "Shift_L", "Shift_R",
                 "Meta_L", "Meta_R", "ISO_Level3_Shift", "Caps_Lock"}


def combo_text(mods, key):
    names = {"SUPER": "Super", "CTRL": "Ctrl", "ALT": "Alt", "SHIFT": "Shift"}
    return "+".join([names[m] for m in mods] + [key])


def shortcut_widget(mods, key):
    """The keys drawn as key caps, or as text when GTK does not know the key's name."""
    accelerator = "".join(ACCELERATOR_MODS[m] for m in mods) + key
    parsed, _, _ = Gtk.accelerator_parse(accelerator)
    if key and parsed:
        return Gtk.ShortcutLabel(accelerator=accelerator, valign=Gtk.Align.CENTER)
    return Gtk.Label(label=combo_text(mods, key) if key else "", css_classes=["keys"])


def set_wm_capture(on):
    subprocess.run([BRASA_WM, "keys-capture", "on" if on else "off"],
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


class KeyCaptureRow(Adw.ActionRow):
    """on_change(mods, key) runs after each recorded combination."""

    def __init__(self, mods, key, on_change):
        super().__init__(title="Keys")
        self.mods, self.key, self.on_change = list(mods), key, on_change
        self.listening = False
        self.shown = None
        self.button = Gtk.Button(label="Record", valign=Gtk.Align.CENTER)
        self.button.connect("clicked", lambda _: self.stop() if self.listening else self.start())
        self.add_suffix(self.button)
        self.keys_box = Gtk.Box(valign=Gtk.Align.CENTER)
        self.add_suffix(self.keys_box)
        self.controller = Gtk.EventControllerKey(propagation_phase=Gtk.PropagationPhase.CAPTURE)
        self.controller.connect("key-pressed", self.key_pressed)
        self.show_keys()
        self.connect("unrealize", lambda _: self.stop())

    def show_keys(self):
        if self.shown:
            self.keys_box.remove(self.shown)
        self.shown = shortcut_widget(self.mods, self.key)
        self.keys_box.append(self.shown)
        self.set_subtitle("" if self.key else "None yet: press Record, then the keys")

    def start(self):
        self.listening = True
        self.button.set_label("Cancel")
        self.set_subtitle("Press the keys now…")
        self.listener().add_controller(self.controller)
        set_wm_capture(True)

    def stop(self):
        if not self.listening:
            return
        self.listening = False
        set_wm_capture(False)
        self.listener().remove_controller(self.controller)
        self.button.set_label("Record")
        self.show_keys()

    def listener(self):
        """The dialog the row is in, else its window: where the keys arrive."""
        return self.get_ancestor(Adw.Dialog) or self.get_root()

    def key_pressed(self, controller, keyval, keycode, state):
        name = self.unshifted_name(controller, keyval, keycode)
        if name in MODIFIER_KEYS:
            return True
        self.mods = [mod for mask, mod in MASKS if state & mask]
        self.key = name
        self.stop()
        self.on_change(self.mods, self.key)
        return True

    def unshifted_name(self, controller, keyval, keycode):
        """b for Shift+b, 1 for Shift+1: the key's name without what Shift does to it."""
        display = self.get_display()
        found, plain, *_ = display.translate_key(keycode, 0, controller.get_group())
        name = Gdk.keyval_name(plain if found and plain else keyval) or ""
        return name.lower() if len(name) == 1 else name
