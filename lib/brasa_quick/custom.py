"""Your own tiles, from ~/.config/brasa/panel.conf, one per line:
    icon | name | command                              a button
    icon | name | on command | off command | test      a switch, lit while the test succeeds"""
import os
import sys

from .items import Action, Switch

PATH = os.path.expanduser("~/.config/brasa/panel.conf")
RUN = "~/.local/bin/brasa-run"


def through_run(name: str, command: str) -> str:
    return f"{RUN} {name.replace(' ', '_')} {command}"


def tile(fields: list[str]):
    icon, name, *commands = fields
    if len(commands) == 1:
        return Action(icon, name, through_run(name, commands[0]))
    on, off, state = commands
    return Switch(icon, name, through_run(name, on), through_run(name, off), state)


def tiles():
    if not os.path.exists(PATH):
        return []
    found = []
    with open(PATH, encoding="utf-8") as conf:
        for line in conf:
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            fields = [part.strip() for part in line.split("|")]
            if len(fields) not in (3, 5):
                print(f"panel.conf: skipped (needs 3 or 5 fields): {line.strip()}", file=sys.stderr)
                continue
            found.append(tile(fields))
    return found
