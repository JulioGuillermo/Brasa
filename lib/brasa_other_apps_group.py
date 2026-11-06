"""The Sessions window's list of the windows open in the user's other sessions, each with Close and Kill."""
import json

from gi.repository import Adw, Gtk


class OtherAppsGroup(Adw.PreferencesGroup):
    """run(argv, on_output) runs a command in the background; command is brasa-other-apps."""

    def __init__(self, run, command):
        super().__init__(title="Apps in other sessions",
                         description="Close lets an app ask about unsaved work. "
                                     "Kill stops it at once, with all its windows")
        self.run = run
        self.command = command
        self.rows = {}
        self.empty_row = Adw.ActionRow(title="None")
        self.add(self.empty_row)

    def refresh(self):
        self.run([self.command, "json"], self.show)

    def show(self, output):
        try:
            windows = json.loads(output)
        except json.JSONDecodeError:
            return
        shown = {window["id"] for window in windows}
        for window_id in list(self.rows):
            if window_id not in shown:
                self.remove(self.rows.pop(window_id))
        for window in windows:
            if window["id"] not in self.rows:
                self.rows[window["id"]] = self.window_row(window["id"])
                self.add(self.rows[window["id"]])
            self.rows[window["id"]].set_title(window["title"] or window["app"])
            self.rows[window["id"]].set_subtitle(
                f"{window['app']} · {window['session']} · workspace {window['workspace']}")
        self.empty_row.set_visible(not windows)

    def window_row(self, window_id):
        row = Adw.ActionRow(title_lines=1, subtitle_lines=1)
        close = Gtk.Button(label="Close", valign=Gtk.Align.CENTER)
        close.connect("clicked", lambda _: self.act(row, "close", window_id))
        kill = Gtk.Button(label="Kill", valign=Gtk.Align.CENTER, css_classes=["destructive-action"])
        kill.connect("clicked", lambda _: self.act(row, "kill", window_id))
        row.add_suffix(close)
        row.add_suffix(kill)
        return row

    def act(self, row, action, window_id):
        row.set_sensitive(False)

        def done(_output):
            row.set_sensitive(True)
            self.refresh()

        self.run([self.command, action, window_id], done)
