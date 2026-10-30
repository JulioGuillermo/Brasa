"""The dialog that adds or edits one of the user's key bindings: the keys, then what they do: open an
app, run a file, run a command, or nothing (turns off Brasa's binding on those keys)."""
import os
import shlex

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gio, GLib, Gtk

from brasa_app_picker import AppPicker, desktop_id
from brasa_key_capture import KeyCaptureRow
from brasa_keys_tsv import BRASA_KEYS, REMOVED, read_rows
from brasa_my_keys import MyKey

APP, FILE, COMMAND, NOTHING = range(4)
KINDS = ["Open an app", "Run a file", "Run a command", "Nothing (turns off Brasa's binding)"]
APP_LAUNCH = "exec gtk-launch "


def brasa_descriptions():
    rows, _ = read_rows(BRASA_KEYS)
    return {row.combo: row.description for row in rows}


def app_by_id(app_id):
    try:
        return Gio.DesktopAppInfo.new(app_id + ".desktop")
    except TypeError:
        return None


def kind_of(action):
    """(kind, value): the app id, the file path or the command an action holds."""
    if action == REMOVED:
        return NOTHING, ""
    if action.startswith(APP_LAUNCH):
        return APP, action.removeprefix(APP_LAUNCH).strip()
    command = action.removeprefix("exec ").strip()
    words = shlex.split(command) if command else []
    if len(words) == 1 and os.path.isfile(os.path.expanduser(words[0])):
        return FILE, words[0]
    if action.startswith("wm "):
        return COMMAND, action
    return COMMAND, command


class KeyEditor(Adw.Dialog):
    """on_save(MyKey) runs on Save. others: combos of the user's other bindings."""

    def __init__(self, entry: MyKey | None, others, on_save):
        super().__init__(title="Edit key binding" if entry else "New key binding", content_width=520,
                         content_height=560)
        entry = entry or MyKey([], "", "")
        self.on_save, self.others = on_save, others
        self.brasa = brasa_descriptions()
        self.mods, self.key = entry.mods, entry.key
        kind, value = kind_of(entry.action) if entry.action else (APP, "")
        self.app_id = value if kind == APP else ""
        self.file = value if kind == FILE else ""

        keys_group = Adw.PreferencesGroup()
        self.capture = KeyCaptureRow(self.mods, self.key, self.keys_changed)
        self.clash = Adw.ActionRow(title="", css_classes=["property"])
        keys_group.add(self.capture)
        keys_group.add(self.clash)

        does = Adw.PreferencesGroup(title="What they do")
        self.kind = Adw.ComboRow(title="Kind", model=Gtk.StringList.new(KINDS), selected=kind)
        self.kind.connect("notify::selected", lambda *_: self.refresh())
        self.app_row = self.chooser_row("App", self.choose_app)
        self.file_row = self.chooser_row("File", self.choose_file)
        self.command = Adw.EntryRow(title="Command", text=value if kind == COMMAND else "")
        self.command.connect("changed", lambda *_: self.refresh())
        self.description = Adw.EntryRow(title="Description, shown in the key bindings window (optional)",
                                        text=entry.description)
        for row in (self.kind, self.app_row, self.file_row, self.command, self.description):
            does.add(row)

        self.save_button = Gtk.Button(label="Save", css_classes=["suggested-action"])
        self.save_button.connect("clicked", lambda _: self.save())
        header = Adw.HeaderBar()
        header.pack_end(self.save_button)
        page = Adw.PreferencesPage()
        page.add(keys_group)
        page.add(does)
        view = Adw.ToolbarView(content=page)
        view.add_top_bar(header)
        self.set_child(view)
        self.refresh()

    def chooser_row(self, title, on_choose):
        row = Adw.ActionRow(title=title)
        button = Gtk.Button(label="Choose…", valign=Gtk.Align.CENTER)
        button.connect("clicked", lambda _: on_choose())
        row.add_suffix(button)
        row.set_activatable_widget(button)
        return row

    def keys_changed(self, mods, key):
        self.mods, self.key = mods, key
        self.refresh()

    def combo(self):
        return frozenset(self.mods), self.key.lower()

    def refresh(self):
        kind = self.kind.get_selected()
        self.app_row.set_visible(kind == APP)
        self.file_row.set_visible(kind == FILE)
        self.command.set_visible(kind == COMMAND)
        self.description.set_visible(kind != NOTHING)
        app = app_by_id(self.app_id) if self.app_id else None
        self.app_row.set_subtitle(app.get_display_name() if app else "None chosen")
        self.file_row.set_subtitle(self.file_text())
        self.clash.set_title(self.clash_text())
        self.clash.set_visible(bool(self.key))
        self.save_button.set_sensitive(bool(self.key) and bool(self.action()))

    def file_text(self):
        if not self.file:
            return "None chosen"
        if not os.access(os.path.expanduser(self.file), os.X_OK):
            return f"{self.file} · not executable: chmod +x it first"
        return self.file

    def clash_text(self):
        if not self.key:
            return ""
        brasa = self.brasa.get(self.combo())
        if self.combo() in self.others:
            return "You already have a binding on these keys: this one replaces it"
        if brasa and self.kind.get_selected() == NOTHING:
            return f"Turns off Brasa's: {brasa}"
        if brasa:
            return f"Replaces Brasa's: {brasa}"
        if self.kind.get_selected() == NOTHING:
            return "Brasa has nothing on these keys"
        return "Free: Brasa has nothing on these keys"

    def action(self):
        kind = self.kind.get_selected()
        if kind == APP:
            return APP_LAUNCH + self.app_id if self.app_id else ""
        if kind == FILE:
            return "exec " + shlex.quote(self.file) if self.file else ""
        if kind == NOTHING:
            return REMOVED
        command = self.command.get_text().strip()
        if command.startswith("wm ") or not command:
            return command
        return "exec " + command

    def choose_app(self):
        def picked(app):
            self.app_id = desktop_id(app)
            if not self.description.get_text():
                self.description.set_text(app.get_display_name())
            self.refresh()
        AppPicker(picked).present(self)

    def choose_file(self):
        def chosen(dialog, result):
            try:
                file = dialog.open_finish(result)
            except GLib.Error:
                return
            self.file = file.get_path()
            self.refresh()
        Gtk.FileDialog(title="File to run").open(self.get_root(), None, chosen)

    def save(self):
        kind = self.kind.get_selected()
        description = "" if kind == NOTHING else self.description.get_text().strip()
        self.on_save(MyKey(self.mods, self.key, self.action(), description))
        self.close()
