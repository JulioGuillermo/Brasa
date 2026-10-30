"""The Key bindings page of Brasa settings: the user's own bindings (my-keys.tsv), to add, edit and
remove. Each change is saved and the window manager reloads them at once."""
import os
import subprocess

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gtk

from brasa_key_capture import shortcut_widget
from brasa_key_editor import KeyEditor, brasa_descriptions
from brasa_my_keys import MyKeysFile

BRASA_WM = os.path.expanduser("~/.local/bin/brasa-wm")


def reload_bindings():
    subprocess.Popen([BRASA_WM, "reload"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                     start_new_session=True)


def icon_button(icon, tooltip, on_click):
    button = Gtk.Button(icon_name=icon, tooltip_text=tooltip, valign=Gtk.Align.CENTER, css_classes=["flat"])
    button.connect("clicked", lambda _: on_click())
    return button


class KeysPage(Adw.PreferencesPage):
    def __init__(self):
        super().__init__()
        self.file = MyKeysFile()
        self.group = None
        self.build()

    def build(self):
        if self.group:
            self.remove(self.group)
        self.group = Adw.PreferencesGroup(
            title="Your key bindings",
            description="On top of Brasa's: the same keys replace Brasa's binding · saved in my-keys.tsv, "
                        "applied at once · Super+F1 lists them under Your own")
        self.group.set_header_suffix(icon_button("list-add-symbolic", "Add a key binding", lambda: self.edit(None)))
        entries = self.file.entries()
        brasa = brasa_descriptions()
        for number, entry in entries:
            self.group.add(self.entry_row(number, entry, brasa))
        if not entries:
            self.group.add(Adw.ActionRow(title="None yet", subtitle="The + button adds one"))
        self.add(self.group)

    def entry_row(self, number, entry, brasa):
        combo = (frozenset(entry.mods), entry.key.lower())
        if entry.removes:
            title = f"Turned off: {brasa.get(combo, 'nothing of Brasa there')}"
        else:
            title = entry.description or entry.action.removeprefix("exec ")
        row = Adw.ActionRow(title=title, subtitle="" if entry.removes else entry.action.removeprefix("exec "),
                            activatable=True)
        row.set_title_lines(1)
        row.set_subtitle_lines(1)
        row.add_prefix(shortcut_widget(entry.mods, entry.key))
        row.add_suffix(icon_button("user-trash-symbolic", "Remove", lambda: self.remove_entry(number)))
        row.connect("activated", lambda _: self.edit(number))
        return row

    def edit(self, number):
        entries = dict(self.file.entries())
        others = {(frozenset(e.mods), e.key.lower()) for n, e in entries.items() if n != number}

        def save(key):
            self.file.put(self.replaced_line(number, key, entries), key)
            self.changed()
        KeyEditor(entries.get(number), others, save).present(self)

    def replaced_line(self, number, key, entries):
        """The line to overwrite: the edited one, else one of the user's already on those keys."""
        if number is not None:
            return number
        combo = (frozenset(key.mods), key.key.lower())
        for other_number, other in entries.items():
            if (frozenset(other.mods), other.key.lower()) == combo:
                return other_number
        return None

    def remove_entry(self, number):
        self.file.remove(number)
        self.changed()

    def changed(self):
        self.build()
        reload_bindings()
