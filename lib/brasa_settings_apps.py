"""The Apps page of Brasa settings: which app each brasa-app key opens (apps.conf). A change applies on
the next key press."""
import shutil

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gtk

import brasa_apps_conf as apps_conf

AUTOMATIC = ""


def option_label(app):
    if app == apps_conf.NONE:
        return "None"
    if shutil.which(app):
        return app
    return f"{app} · not installed"


class RoleRow(Adw.ComboRow):
    def __init__(self, role, title, keys, current, on_change):
        super().__init__(title=title, subtitle=keys)
        self.role = role
        self.values = [AUTOMATIC] + apps_conf.known_apps(role)
        if role in apps_conf.OPTIONAL:
            self.values.append(apps_conf.NONE)
        if current not in self.values:
            self.values.append(current)
        labels = ["Automatic"] + [option_label(app) for app in self.values[1:]]
        self.set_model(Gtk.StringList.new(labels))
        self.set_selected(self.values.index(current))
        self.keys = keys
        self.show_now()
        self.connect("notify::selected", lambda *_: on_change())

    def show_now(self):
        """The keys, and what Automatic opens now."""
        if self.value() != AUTOMATIC:
            self.set_subtitle(self.keys)
            return
        now = apps_conf.which_app(self.role) or "none installed"
        self.set_subtitle(f"{self.keys} · now {now}")

    def value(self):
        return self.values[self.get_selected()]


class AppsPage(Adw.PreferencesPage):
    def __init__(self):
        super().__init__()
        chosen = apps_conf.load()
        group = Adw.PreferencesGroup(
            title="Apps the keys open",
            description="Automatic: the first installed of the list · one not installed: ./install.sh "
                        "installs it · applies on the next key press")
        self.rows = []
        for role, title, keys in apps_conf.ROLES:
            row = RoleRow(role, title, keys, chosen.get(role, AUTOMATIC), self.changed)
            self.rows.append(row)
            group.add(row)
        self.add(group)

    def changed(self):
        apps_conf.save({row.role: row.value() for row in self.rows})
        for row in self.rows:
            row.show_now()
