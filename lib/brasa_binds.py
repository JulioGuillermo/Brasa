"""The key bindings the running window manager really has (Hyprland or sway), each with what it does, what
it runs and what kind of thing it is."""
import glob
import json
import os
import re
import shlex
import subprocess
from dataclasses import dataclass, field

BRASA = os.path.realpath(os.path.expanduser("~/.config/brasa"))
HOME_BIN = os.path.expanduser("~/.local/bin")

CATEGORIES = ["Apps", "Brasa", "Commands", "Windows", "Workspaces", "Sound", "Brightness", "Media",
              "Mouse", "Window manager"]

HYPR_MODMASK = [(64, "Super"), (4, "Ctrl"), (8, "Alt"), (1, "Shift"), (2, "Caps"), (16, "Mod2"), (32, "Mod3"),
                (128, "Mod5")]
MOD_NAMES = {"SUPER": "Super", "WIN": "Super", "LOGO": "Super", "MOD4": "Super", "CTRL": "Ctrl",
             "CONTROL": "Ctrl", "ALT": "Alt", "MOD1": "Alt", "SHIFT": "Shift"}
MOD_ORDER = ["Super", "Ctrl", "Alt", "Shift"]
KEY_NAMES = {"mouse:272": "Left button", "mouse:273": "Right button", "mouse:274": "Middle button",
             "mouse_down": "Wheel down", "mouse_up": "Wheel up", "button1": "Left button",
             "button3": "Right button", "button4": "Wheel up", "button5": "Wheel down", "Return": "Enter",
             "period": ".", "comma": ",", "space": "Space", "Escape": "Esc"}
MODIFIER_KEYS = {"Super_L": "Super", "Super_R": "Super"}
MOUSE_KEY = re.compile(r"^(mouse|button\d)")

HYPR_ACTIONS = [
    (r"^window\.close", "Close window"), (r"^window\.kill", "Kill the app"),
    (r"^window\.fullscreen", "Fullscreen"), (r"^window\.float", "Floating / tiled"),
    (r"^window\.drag", "Move the window with the mouse"), (r"^window\.resize", "Resize the window with the mouse"),
    (r"^window\.move.*workspace", "Move window to a workspace"), (r"^window\.move", "Move window"),
    (r'workspace = "[er]\+1"', "Next workspace"), (r'workspace = "[er]-1"', "Previous workspace"),
    (r"^focus.*workspace", "Go to a workspace"), (r"^focus.*monitor", "Focus a monitor"),
    (r"^focus", "Focus a window"), (r"^workspace", "Workspace"), (r"^submap", "Key mode"),
    (r"^exit", "Quit the window manager"), (r"^dpms", "Screens on / off"), (r"^layout", "Layout"),
]
SWAY_ACTIONS = [
    (r"^kill", "Close window"), (r"^fullscreen", "Fullscreen"), (r"^floating", "Floating / tiled"),
    (r"^move .*workspace", "Move window to a workspace"), (r"^move workspace to output", "Move workspace to a monitor"),
    (r"^move", "Move window"), (r"^workspace", "Go to a workspace"), (r"^focus output", "Focus a monitor"),
    (r"^focus", "Focus a window"), (r"^layout", "Layout"), (r"^mode", "Key mode"), (r"^reload", "Reload sway"),
    (r"^exit", "Quit sway"), (r"^input", "Keyboard / mouse setting"), (r"^split", "Split"),
]


@dataclass
class Bind:
    mods: list
    key: str
    description: str = ""
    action: str = ""
    category: str = "Window manager"
    run: list = field(default_factory=list)
    mode: str = ""

    @property
    def keys(self):
        if self.key in MODIFIER_KEYS:
            return f"{MODIFIER_KEYS[self.key]} alone"
        return " + ".join(self.mods + [key_name(self.key)])


def key_name(key):
    """Enter for Return, Wheel up for mouse_up, "Audio Raise Volume key" for XF86AudioRaiseVolume..."""
    if key in KEY_NAMES:
        return KEY_NAMES[key]
    if key.startswith("XF86"):
        return re.sub(r"(?<=[a-z])(?=[A-Z0-9])", " ", key[4:]) + " key"
    return key


def normal_mods(names):
    mods = {MOD_NAMES.get(n.upper(), n.capitalize()) for n in names if n}
    return sorted(mods, key=lambda m: MOD_ORDER.index(m) if m in MOD_ORDER else len(MOD_ORDER))


def combo_id(mods, key):
    return frozenset(mods), key.lower()


# ---- descriptions of commands

def desktop_entries():
    """{program or desktop id: app name} from the installed .desktop files."""
    dirs = [os.path.expanduser("~/.local/share")] + os.environ.get(
        "XDG_DATA_DIRS", "/usr/local/share:/usr/share").split(":")
    names = {}
    for base in dirs:
        apps = os.path.join(base, "applications")
        if not os.path.isdir(apps):
            continue
        for entry in os.listdir(apps):
            if entry.endswith(".desktop"):
                name, program = read_desktop(os.path.join(apps, entry))
                if name:
                    names.setdefault(entry.removesuffix(".desktop"), name)
                    if program:
                        names.setdefault(program, name)
    return names


def read_desktop(path):
    name = program = None
    try:
        with open(path, encoding="utf-8", errors="replace") as desktop:
            for line in desktop:
                if line.startswith("[") and line.strip() != "[Desktop Entry]":
                    break
                if line.startswith("Name=") and not name:
                    name = line[5:].strip()
                elif line.startswith("Exec=") and not program:
                    words = shlex.split(line[5:], posix=True) if line[5:].strip() else []
                    words = [w for w in words if "=" not in w or w.startswith("/")]
                    program = os.path.basename(words[0]) if words else None
    except (OSError, ValueError):
        pass
    return name, program


def script_summary(path):
    """The first line of a script's header comment, up to its first parenthesis, colon or full stop."""
    try:
        with open(path, encoding="utf-8", errors="replace") as script:
            lines = script.readlines()[1:4]
    except OSError:
        return None
    for line in lines:
        text = line.strip().lstrip("#").strip().strip('"')
        if text:
            return re.split(r" \(|: |\. ", text)[0]
    return None


def program_of(command):
    try:
        words = shlex.split(command)
    except ValueError:
        words = command.split()
    words = [w for w in words if not re.match(r"^\w+=", w)]
    return (words[0] if words else ""), words[1:]


def describe_command(command, apps):
    program, args = program_of(command)
    name = os.path.basename(program)
    if name == "gtk-launch" and args:
        return f"Open {apps.get(args[0], args[0])}", "Apps"
    if name in apps:
        return f"Open {apps[name]}", "Apps"
    path = os.path.expanduser(program)
    if not os.path.isabs(path):
        path = os.path.join(HOME_BIN, name)
    if name.startswith("brasa-") and os.path.exists(path):
        return script_summary(path) or name, "Brasa"
    return f"Run {name}", "Commands"


def command_category(command, key, fallback):
    text = f"{command} {key}"
    if re.search(r"brightness|backlight|MonBrightness", text, re.I):
        return "Brightness"
    if re.search(r"playerctl|AudioPlay|AudioPause|AudioNext|AudioPrev|AudioStop", text):
        return "Media"
    if re.search(r"volume|wpctl|pamixer|pactl|AudioMute|AudioMicMute", text, re.I):
        return "Sound"
    if "brasa-wm" in command:
        return "Windows"
    return fallback


def action_category(action, key):
    if MOUSE_KEY.match(key):
        return "Mouse"
    if re.search(r"workspace|monitor|output", action):
        return "Workspaces"
    if re.search(r"^(window|focus|kill|fullscreen|floating|move|layout|split)", action):
        return "Windows"
    return "Window manager"


def matching_phrase(action, phrases):
    return next((phrase for pattern, phrase in phrases if re.search(pattern, action)), action)


def keys_tsv_descriptions():
    """{combo: description} from Brasa's keys.tsv (sway's config has no descriptions)."""
    found = {}
    try:
        lines = open(os.path.join(BRASA, "keys.tsv"), encoding="utf-8").read().splitlines()
    except OSError:
        return found
    for line in lines:
        if not line.strip() or line.startswith("#"):
            continue
        mods, key, _, description, *_ = line.split("\t")
        mods = normal_mods([m for m in mods.split("+") if m != "-"])
        keys = [(str(n), str(n)) for n in range(1, 10)] if key == "1..9" else [(key, None)]
        for k, n in keys:
            found[combo_id(mods, k)] = description.replace("{n}", n) if n else description
    return found


# ---- Hyprland

def lua_calls(text, name):
    """The argument lists (as source text, split at top-level commas) of every call to name(...)."""
    text = "\n".join(line for line in text.splitlines() if not line.strip().startswith("--"))
    for start in (m.end() for m in re.finditer(re.escape(name) + r"\s*\(", text)):
        args, depth, quote, current, i = [], 1, None, "", start
        while i < len(text) and depth:
            char = text[i]
            if quote:
                if char == "\\":
                    current += text[i:i + 2]
                    i += 2
                    continue
                if char == quote:
                    quote = None
            elif char in "\"'":
                quote = char
            elif char in "([{":
                depth += 1
            elif char in ")]}":
                depth -= 1
            elif char == "," and depth == 1:
                args.append(current.strip())
                current = ""
                i += 1
                continue
            if depth:
                current += char
            i += 1
        args.append(current.strip())
        yield args


def hypr_sources():
    """{combo: Lua expression} of every hl.bind in the Lua files of ~/.config/hypr."""
    folder = os.path.realpath(os.path.expanduser("~/.config/hypr"))
    sources = {}
    for name in sorted(os.listdir(folder)):
        if not name.endswith(".lua"):
            continue
        with open(os.path.join(folder, name), encoding="utf-8") as lua:
            text = lua.read()
        for args in lua_calls(text, "hl.bind"):
            combo = re.match(r"""^["'](.+)["']$""", args[0]) if len(args) > 1 else None
            if not combo:
                continue
            *mods, key = [part.strip() for part in combo.group(1).split("+")]
            sources.setdefault(combo_id(normal_mods(mods), key), args[1])
    return sources


def lua_string(text):
    found = re.match(r"""^\s*["'](.*)["']\s*$""", text, re.S)
    return found.group(1).replace('\\"', '"') if found else None


def hypr_binds():
    binds = json.loads(subprocess.run(["hyprctl", "binds", "-j"], capture_output=True, text=True).stdout)
    sources = hypr_sources()
    apps = desktop_entries()
    result = []
    for raw in binds:
        mods = [name for bit, name in HYPR_MODMASK if raw["modmask"] & bit]
        bind = Bind(mods=mods, key=raw["key"], description=raw.get("description", ""), mode=raw.get("submap", ""))
        if raw["dispatcher"] == "__lua":
            expression = sources.get(combo_id(mods, raw["key"]), "")
            exec_call = re.match(r"^hl\.dsp\.exec_cmd\((.*)\)$", expression, re.S)
            command = lua_string(exec_call.group(1)) if exec_call else None
            if command:
                describe_exec(bind, command, apps)
            elif expression:
                action = re.sub(r"^hl\.dsp\.", "", expression)
                bind.action = action
                bind.category = action_category(action, bind.key)
                bind.description = bind.description or matching_phrase(action, HYPR_ACTIONS)
                code = f"({expression})()" if expression.startswith("function") else f"hl.dispatch({expression})"
                bind.run = ["hyprctl", "eval", code]
            else:
                bind.category = action_category("", bind.key)
        elif raw["dispatcher"] == "exec":
            describe_exec(bind, raw["arg"], apps)
        else:
            action = f"{raw['dispatcher']} {raw['arg']}".strip()
            bind.action = action
            bind.category = action_category(action, bind.key)
            bind.description = bind.description or action
            bind.run = ["hyprctl", "dispatch", raw["dispatcher"], raw["arg"]]
        if MOUSE_KEY.match(bind.key):
            bind.run = []
        result.append(bind)
    return result


def describe_exec(bind, command, apps):
    description, category = describe_command(command, apps)
    bind.action = command
    bind.description = bind.description or description
    bind.category = command_category(command, bind.key, category)
    bind.run = ["sh", "-c", command]


# ---- sway

def sway_config_text():
    config = json.loads(subprocess.run(["swaymsg", "-t", "get_config", "-r"], capture_output=True,
                                       text=True).stdout)
    main = config.get("config", "")
    if "included_configs" in config:
        included = [c.get("raw_config", "") for c in config["included_configs"]]
    else:
        included = read_includes(main)
    return "\n".join([main] + included)


def read_includes(text, seen=None):
    """The text of the files an older sway's config includes (newer ones send them with the config)."""
    seen = seen if seen is not None else set()
    texts = []
    for pattern in re.findall(r"^\s*include\s+(.+?)\s*$", text, re.M):
        for path in sorted(glob.glob(os.path.expandvars(os.path.expanduser(pattern.strip("\"'"))))):
            if path in seen:
                continue
            seen.add(path)
            with open(path, encoding="utf-8", errors="replace") as included:
                content = included.read()
            texts += [content] + read_includes(content, seen)
    return texts


def sway_binds_from(text):
    text = re.sub(r"\\\n\s*", " ", text)
    variables, modes, binds = {}, [], []
    apps, described = desktop_entries(), keys_tsv_descriptions()
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        for name, value in sorted(variables.items(), key=lambda v: -len(v[0])):
            line = line.replace(name, value)
        if setting := re.match(r"^set\s+(\$\S+)\s+(.+)$", line):
            variables[setting.group(1)] = setting.group(2)
        elif mode := re.match(r"""^mode\s+(?:--\S+\s+)*["']?([^"'{]+?)["']?\s*\{$""", line):
            modes.append(mode.group(1))
        elif line == "}" and modes:
            modes.pop()
        elif bind := re.match(r"^bind(sym|code)\s+((?:--\S+\s+)*)(\S+)\s+(.+)$", line):
            binds.append(sway_bind(bind.group(3), bind.group(4), modes[-1] if modes else "", apps, described))
    return binds


def sway_bind(combo, command, mode, apps, described):
    *mods, key = combo.split("+")
    mods = normal_mods(mods)
    bind = Bind(mods=mods, key=key, mode=mode, description=described.get(combo_id(mods, key), ""))
    if command.startswith("exec "):
        describe_exec(bind, re.sub(r"^exec\s+(--no-startup-id\s+)?", "", command), apps)
    else:
        bind.action = command
        bind.category = action_category(command, key)
        bind.description = bind.description or matching_phrase(command, SWAY_ACTIONS)
        bind.run = ["swaymsg", command]
    if MOUSE_KEY.match(key):
        bind.run = []
        bind.category = "Mouse"
    return bind


def current_binds():
    if os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"):
        return hypr_binds()
    if os.environ.get("SWAYSOCK"):
        return sway_binds_from(sway_config_text())
    return []
