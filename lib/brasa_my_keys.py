"""my-keys.tsv as the settings window edits it: one entry per binding line, comments kept where they are."""
import os
from dataclasses import dataclass

from brasa_keys_tsv import MY_KEYS, REMOVED


@dataclass
class MyKey:
    mods: list
    key: str
    action: str
    description: str = ""
    options: str = ""

    def line(self):
        mods = "+".join(self.mods) or "-"
        fields = [mods, self.key, self.action, self.description]
        if self.options:
            fields.append(self.options)
        return "\t".join(fields).rstrip("\t")

    @property
    def removes(self):
        return self.action == REMOVED


def parse(line):
    fields = line.split("\t") + ["", ""]
    mods = [m.strip().upper() for m in fields[0].split("+") if m.strip() not in ("", "-")]
    return MyKey(mods, fields[1].strip(), fields[2].strip(), fields[3].strip(), fields[4].strip())


class MyKeysFile:
    def __init__(self, path=MY_KEYS):
        self.path = path
        try:
            with open(path, encoding="utf-8") as file:
                self.lines = file.read().splitlines()
        except FileNotFoundError:
            self.lines = ["# Your own key bindings, on top of keys.tsv (same format). Brasa settings edits it."]

    def entries(self):
        """[(line number, MyKey)] of the binding lines."""
        found = []
        for number, line in enumerate(self.lines):
            if line.strip() and not line.lstrip().startswith("#"):
                found.append((number, parse(line)))
        return found

    def put(self, number, key: MyKey):
        """Replaces the line at number, or adds one when number is None."""
        if number is None:
            self.lines.append(key.line())
        else:
            self.lines[number] = key.line()
        self.save()

    def remove(self, number):
        del self.lines[number]
        self.save()

    def save(self):
        temporary = self.path + ".new"
        with open(temporary, "w", encoding="utf-8") as file:
            file.write("\n".join(self.lines) + "\n")
        os.replace(temporary, self.path)
