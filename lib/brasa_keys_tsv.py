"""Brasa's key bindings (keys.tsv) with the user's own (my-keys.tsv) on top: a row of the user's with the
same keys replaces Brasa's, and one whose action is "none" removes it."""
import os
from dataclasses import dataclass, field

BRASA = os.path.realpath(os.path.expanduser("~/.config/brasa"))
BRASA_KEYS = os.path.join(BRASA, "keys.tsv")
MY_KEYS = os.path.join(BRASA, "my-keys.tsv")
REMOVED = "none"
MODIFIERS = {"SUPER", "SHIFT", "CTRL", "ALT"}


@dataclass
class KeyRow:
    mods: list
    key: str
    action: str
    description: str
    options: list = field(default_factory=list)
    origin: str = ""

    @property
    def combo(self):
        return frozenset(self.mods), self.key.lower()


def expanded_keys(key):
    """[(key, n)]: "1..9" gives one per digit, with n to put in place of {n}."""
    if key == "1..9":
        return [(str(n), str(n)) for n in range(1, 10)]
    return [(key, None)]


def default_description(action):
    return action.removeprefix("exec ").strip()


def parse_line(line, origin):
    fields = line.split("\t")
    if len(fields) < 3:
        raise ValueError("needs modifiers, key and action, separated by tabs")
    mods_text, key, action = (text.strip() for text in fields[:3])
    description = fields[3].strip() if len(fields) > 3 else ""
    options = [o for o in fields[4].strip().split(",") if o] if len(fields) > 4 else []
    mods = [m.upper() for m in mods_text.split("+") if m != "-"]
    unknown = [m for m in mods if m not in MODIFIERS]
    if unknown:
        raise ValueError(f"unknown modifier {unknown[0]} (SUPER, SHIFT, CTRL, ALT, or - for none)")
    if not key:
        raise ValueError("no key")
    rows = []
    for each_key, n in expanded_keys(key):
        def put_n(text):
            return text.replace("{n}", n) if n else text
        each_description = put_n(description) or default_description(put_n(action))
        rows.append(KeyRow(mods, each_key, put_n(action), each_description, options, origin))
    return rows


def read_rows(path):
    """(rows, problems) of one file; a missing file has neither."""
    rows, problems = [], []
    try:
        with open(path, encoding="utf-8") as file:
            lines = file.read().splitlines()
    except FileNotFoundError:
        return rows, problems
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        origin = f"{os.path.basename(path)} line {number}"
        try:
            rows += parse_line(line, origin)
        except ValueError as error:
            problems.append(f"{origin}: {error}")
    return rows, problems


def merged_rows():
    """(rows, problems): Brasa's rows, each replaced by the user's with the same keys, then the user's new
    ones; rows whose action is "none" are left out."""
    brasa, problems = read_rows(BRASA_KEYS)
    mine, my_problems = read_rows(MY_KEYS)
    mine_by_combo = {row.combo: row for row in mine}
    rows = [mine_by_combo.pop(row.combo, row) for row in brasa]
    rows += mine_by_combo.values()
    return [row for row in rows if row.action != REMOVED], problems + my_problems


def my_combos():
    """(modifiers, key) of every row in my-keys.tsv."""
    rows, _ = read_rows(MY_KEYS)
    return [(row.mods, row.key) for row in rows if row.action != REMOVED]
