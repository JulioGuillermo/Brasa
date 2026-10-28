# Generates config/waybar/config.jsonc, config/waybar/segments.css and config/swaync/config.json for
# the Brasa theme, for the hardware of this machine: blocks for things it does not have (battery, fans,
# temperature sensors, backlight, bluetooth) are left out, and the alternating segment colors and the
# triangles between them are computed for the blocks that remain. The installer runs it.
# Usage: python3 docs/gen-waybar-swaync.py config   (from the brasa folder)
import shutil, glob, json, os, sys
out = sys.argv[1]


def read(path):
    try:
        return open(path).read().strip()
    except OSError:
        return ""


hwmon = {read(f"{h}/name"): h for h in glob.glob("/sys/class/hwmon/hwmon*")}
HAS = {
    "battery": bool(glob.glob("/sys/class/power_supply/BAT*")),
    "fans": bool(glob.glob("/sys/class/hwmon/hwmon*/fan*_input")),
    "temp": any(n in hwmon for n in ("coretemp", "k10temp", "zenpower", "cpu_thermal", "acpitz")),
    "backlight": bool(glob.glob("/sys/class/backlight/*")),
    "bluetooth": bool(glob.glob("/sys/class/bluetooth/*")),
}
# Internal panel (laptops): the persistent workspaces 1-4 live there; elsewhere they would show up on
# every monitor's bar even though each workspace belongs to one monitor.
INTERNAL = next((os.path.basename(d).split("-", 1)[1] for d in sorted(glob.glob("/sys/class/drm/card*-*"))
                 if os.path.basename(d).split("-", 1)[1].startswith(("eDP", "LVDS", "DSI"))
                 and read(f"{d}/status") == "connected"), None)
LEFT = [g for g, ok in [("g-cpu", True), ("g-thermal", HAS["temp"] or HAS["fans"]), ("g-mem", True),
                        ("g-power", HAS["battery"]), ("g-screen", True)] if ok]
I = lambda c: f"<span color='#ff0000'>{c}</span>"   # group icon, in Redline's alert red

ICON = dict(cpu="", temp="", fan="\U000F0210", ram="\U000F035B", swap="\U000F04E1",
            up="\U000F005D", down="\U000F0045", net="\U000F05A9", bolt="\U000F0241", hz="\U000F0379",
            clock="\U000F0150", cal="\U000F00ED", bt="\U000F00AF", bt_off="\U000F00B2",
            vol=["\U000F057F", "\U000F0580", "\U000F057E"], mute="\U000F075F", bri="\U000F00DF",
            bell="\U000F009A", bell_new="\U000F009E", bell_off="\U000F009B",
            ws_on="", ws_off="", ws_urg="", eth="\U000F0200", disc="\U000F05AA",
            bat=["\U000F007A", "\U000F007B", "\U000F007C", "\U000F007D", "\U000F007E",
                 "\U000F007F", "\U000F0080", "\U000F0081", "\U000F0082", "\U000F0079"], chg="\U000F0084", timer="\U000F051F")
S = "~/.config/waybar/sens"
load = {"warm": 50, "hot": 75, "crit": 90}

waybar = {
  "layer": "top", "position": "top", "height": 28,
  "margin-top": 6, "margin-left": 8, "margin-right": 8, "spacing": 0,
  # Left: resource blocks this machine has, each followed by a triangle (l1, l2...)
  "modules-left": [m for i, g in enumerate(LEFT) for m in (f"group/{g}", f"custom/l{i + 1}")],
  "modules-center": ["custom/c1", "sway/workspaces#sway", "hyprland/workspaces#hypr", "custom/c2"],
  # Network sits on the right, nearest the center: its width changes the most and nothing else moves
  "modules-right": ["custom/r0", "group/g-net", "custom/r1", "clock", "custom/r2", "group/g-ctl", "custom/r3",
                    "tray", "custom/quick", "custom/notif"],

  **{f"custom/{n}": {"format": g, "tooltip": False} for n, g in
     [("l1", "\ue0b0"), ("l2", "\ue0b0"), ("l3", "\ue0b0"), ("l4", "\ue0b0"), ("l5", "\ue0b0"), ("l6", "\ue0b0"),
      ("c1", "\ue0b2"), ("c2", "\ue0b0"), ("r0", "\ue0b2"), ("r1", "\ue0b2"), ("r2", "\ue0b2"), ("r3", "\ue0b2")]},
  "group/g-cpu": {"orientation": "horizontal", "modules": ["cpu"]},
  "group/g-thermal": {"orientation": "horizontal", "modules": ["custom/thermal"]},
  "group/g-mem": {"orientation": "horizontal", "modules": ["memory"]},
  "group/g-net": {"orientation": "horizontal", "modules": ["network#rate"]},
  "group/g-power": {"orientation": "horizontal", "modules": ["battery", "custom/volt"]},
  "group/g-screen": {"orientation": "horizontal", "modules": ["custom/hz"]},
  "group/g-ctl": {"orientation": "horizontal",
                  "modules": ["sway/language", "hyprland/language", "network#wifi"] + (["bluetooth"] if HAS["bluetooth"] else [])
                             + ["wireplumber"] + (["backlight"] if HAS["backlight"] else [])},

  "cpu": {"interval": 1, "states": load, "tooltip": False},
  "custom/thermal": {"exec": S + " thermal", "return-type": "json", "interval": 3},
  "memory": {"interval": 1, "states": {"warm": 70, "hot": 85, "crit": 93},
             "tooltip-format": "RAM: {used:.1f} of {total:.1f} GiB used, {avail:.1f} GiB available\n"
                               "Swap: {swapUsed:.1f} of {swapTotal:.1f} GiB used\n"
                               "Click: where it is (apps, graphics, kernel, cache, swap) · Right click: fold",
             # The breakdown in a small floating terminal, only while it is open (brasa-mem)
             "on-click": "foot --app-id=brasa-mem --title=Memory -W 76x32 ~/.local/bin/brasa-mem"},
  "network#rate": {"interval": 2, "tooltip-format": "{ifname}: {ipaddr}\nRight click: fold"},
  "battery": {"interval": 5, "states": {"warning": 30, "critical": 15},
              "on-click-right": "~/.local/bin/brasa-battery menu",   # charge limit: 60 / 80 / 90 / 100 %
              # Time left while discharging, time to full while charging ({time} is empty when unknown)
              "format": I("{icon}") + " {capacity}% " + I(ICON["bolt"]) + " {power:.1f}<small>W</small>",
              "format-discharging": I("{icon}") + " {capacity}% " + I(ICON["bolt"]) + " {power:.1f}<small>W</small> "
                                    + I(ICON["timer"]) + " {time}",
              "format-charging": I(ICON["chg"]) + " {capacity}% " + I(ICON["bolt"]) + " {power:.1f}<small>W</small> "
                                 + I(ICON["timer"]) + " {time}",
              "format-full": I(ICON["bat"][-1]) + " {capacity}%",
              "format-time": "{H}h{M:02}<small>m</small>",
              "format-icons": ICON["bat"], "tooltip-format": "{timeTo} · {power:.1f} W"},
  "custom/volt": {"exec": S + " volt", "return-type": "json", "interval": 3},
  "custom/hz": {"exec": "~/.config/waybar/hz", "return-type": "json", "restart-interval": 5,
                "on-click": "~/.local/bin/brasa-refresh menu", "on-click-middle": "~/.local/bin/brasa-hz-real",
                "on-scroll-up": "~/.local/bin/brasa-refresh cycle", "on-scroll-down": "~/.local/bin/brasa-refresh cycle"},

  # Each bar shows its own monitor's workspaces: the number, inside a circle drawn in style.css.
  # "#sway" / "#hypr" become CSS classes: the two modules mark empty workspaces differently.
  "sway/workspaces#sway": {"format": "{name}", "disable-scroll": False, "all-outputs": False,
                      "persistent-workspaces": {str(n): [INTERNAL] if INTERNAL else [] for n in range(1, 5)}},
  "hyprland/workspaces#hypr": {"format": "{name}", "all-outputs": False,
                          "persistent-workspaces": {INTERNAL: [1, 2, 3, 4]} if INTERNAL else {"*": 4}},

  "clock": {"interval": 1, "format": I(ICON["cal"]) + " {:%a %d %b  " + I(ICON["clock"]) + " %H:%M:%S}",  # one {} only
            "format-alt": I(ICON["clock"]) + " {:%H:%M}",
            "tooltip-format": "<tt>{calendar}</tt>",
            "calendar": {"mode": "month", "weeks-pos": "left", "on-scroll": 1,
                         "format": {"months": "<span color='#ff0000'><b>{}</b></span>",
                                    "weekdays": "<span color='#c3cbe0'>{}</span>",
                                    "weeks": "<span color='#2c3a5e'>{}</span>",
                                    "days": "<span color='#ffffff'>{}</span>",
                                    "today": "<span background='#ff0000' color='#00040f'><b>{}</b></span>"}},
            "actions": {"on-scroll-up": "shift_up", "on-scroll-down": "shift_down", "on-click-right": "mode"}},

  "network#wifi": {"format-wifi": "{icon}", "format-icons": ["\U000F092F", "\U000F091F", "\U000F0922", "\U000F0925", "\U000F0928"],
                   "format-ethernet": ICON["eth"], "format-disconnected": ICON["disc"],
                   "tooltip-format-wifi": "{essid} · {signalStrength}% · {frequency} GHz\n{ipaddr}",
                   "on-click": "nm-connection-editor"},
  "bluetooth": {"format": ICON["bt"], "format-off": ICON["bt_off"], "format-disabled": ICON["bt_off"],
                "format-connected": ICON["bt"] + " {num_connections}",
                "tooltip-format-connected": "{device_enumerate}", "tooltip-format-enumerate-connected": "{device_alias}",
                "on-click": "blueman-manager"},
  "wireplumber": {"format": "{icon}", "format-muted": ICON["mute"], "format-icons": ICON["vol"],
                  "tooltip-format": "Volume {volume}% · {node_name}", "scroll-step": 5,
                  "on-click": "pavucontrol", "on-click-right": "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"},
  "backlight": {"device": "intel_backlight", "format": ICON["bri"], "tooltip-format": "Brightness {percent}%",
                "on-scroll-up": "brightnessctl -q s +5%", "on-scroll-down": "brightnessctl -q s 5%-"},
  "sway/language": {"format": "{short}", "tooltip-format": "{long}", "on-click": "~/.local/bin/brasa-wm layout-next",
                    "on-click-right": "~/.local/bin/brasa-keyboard"},
  "hyprland/language": {"format": "{short}", "on-click": "~/.local/bin/brasa-wm layout-next",
                        "on-click-right": "~/.local/bin/brasa-keyboard"},
  "tray": {"icon-size": 16, "spacing": 8},
  "custom/quick": {"format": "\U000F062E", "tooltip": False, "on-click": "~/.local/bin/brasa-quick"},
  # The island writes the bell's state to brasa-bell.json and sends this signal (lib/brasa_island/bell.py)
  "custom/notif": {"exec": "cat \"$XDG_RUNTIME_DIR/brasa-bell.json\" 2>/dev/null || echo '{\"alt\": \"none\"}'",
                   "return-type": "json", "format": "{icon}", "interval": "once", "signal": 8,
                   "format-icons": {"notification": ICON["bell_new"], "none": ICON["bell"],
                                    "dnd-notification": ICON["bell_off"], "dnd-none": ICON["bell_off"]},
                   "tooltip": False, "on-click": "~/.local/bin/brasa-island centre",
                   "on-click-right": "~/.local/bin/brasa-island dnd"},
}

# Blocks that fold, (unfolded, folded): right click switches. The bar of a narrow monitor starts folded
# (compact.jsonc, which brasa-bar gives to the monitors narrower than its limit), the others unfolded.
# The custom blocks' scripts print the folded block as "alt".
SEP = " <span color='#5a6a90'>·</span> "
FOLDS = {
  "cpu": (I(ICON["cpu"]) + " {usage:>2}% {avg_frequency:.1f}<small>GHz</small>", I(ICON["cpu"]) + " {usage:>2}%"),
  "custom/thermal": ("{text}", "{alt}"),
  "memory": (I(ICON["ram"]) + " {used:.1f}<small>G</small>" + SEP + "{avail:.1f}<small>G</small>" + SEP
             + "{percentage}%  " + I(ICON["swap"]) + " {swapPercentage}%", I(ICON["ram"]) + " {percentage}%"),
  "custom/volt": ("{text}", "{alt}"),
  "custom/hz": ("{text}", "{alt}"),
  "network#rate": (I(ICON["up"]) + " {bandwidthUpBytes}  " + I(ICON["down"]) + " {bandwidthDownBytes}",
                   I(ICON["down"]) + " {bandwidthDownBytes}"),
}
for module, (unfolded, folded) in FOLDS.items():
    waybar[module].update({"format": unfolded, "format-alt": folded, "format-alt-click": "click-right"})


def folded_bar(bar):
    """The same bar starting folded: every fold and the clock show their short form first."""
    compact = json.loads(json.dumps(bar))
    for module in [*FOLDS, "clock"]:
        compact[module]["format"], compact[module]["format-alt"] = bar[module]["format-alt"], bar[module]["format"]
    return compact


B = "~/.local/bin/"
HAS["charge_limit"] = bool(glob.glob("/sys/class/power_supply/BAT*/charge_control_end_threshold"))
HAS["power_profiles"] = bool(shutil.which("powerprofilesctl"))
CLOSE = "swaync-client -cp >/dev/null"


def tog(label, on, off, state):
    return {"label": label, "type": "toggle", "command": f"sh -c 'test $SWAYNC_TOGGLE_STATE = true && {on} || {off}'",
            "update-command": f"sh -c '{state}'"}


def tile(icon, word):
    """A grid button: the icon above, one word below. GTK left-aligns the lines of a label, so the icon is
    pushed to the middle of the word with spaces (the tiles use a monospace font)."""
    return f"{' ' * ((len(word) - 1) // 2)}{icon}\n{word}"


# Note: swaync mangles double quotes inside a command, so none of these commands use them
def choice(text, run, current):
    """A menu entry that is highlighted when it is the current choice: a toggle whose state comes from
    'current' (a test); whatever its state, a click runs 'run' and closes the panel (so the highlight is
    right the next time it opens). Padded to one width: with the monospace font of these menus that
    keeps the text left-aligned."""
    return {"label": f"{text:<38}", "type": "toggle", "command": f"sh -c '{run}; {CLOSE}'",
            "update-command": f"sh -c '{current} && echo true || echo false'"}


MENUS = {
  "menu#layout": {"label": tile("\U000F056E", "Layout"), "position": "left", "actions": [
      choice(f"{name:<10} {desc}", B + f"brasa-wm layout-set {layout}",
             f"{B}brasa-wm layout-current | grep -qx {layout}")
      for layout, name, desc in (("dwindle", "Dwindle", "split the focused window"),
                                 ("master", "Master", "one big + a column"),
                                 ("scrolling", "Scroll →", "half-screen columns"),
                                 ("scrolling-down", "Scroll ↓", "half-screen rows"),
                                 ("focus", "Focus →", "columns, the focused one big"),
                                 ("focus-down", "Focus ↓", "rows, the focused one big"))]},
}
if HAS["charge_limit"]:
    MENUS["menu#battery"] = {"label": tile("\U000F1211", "Battery"), "position": "left", "actions": [
        choice(f"Charge up to {n} %", B + f"brasa-battery {n}",
               f"grep -qx {n} /sys/class/power_supply/BAT*/charge_control_end_threshold")
        for n in (60, 80, 90, 100)]}
if HAS["power_profiles"]:
    MENUS["menu#power"] = {"label": tile("\U000F04C5", "Power"), "position": "left", "actions": [
        choice(label, B + f"brasa-power {p}", f"powerprofilesctl get | grep -qx {p}")
        for p, label in (("power-saver", "Power saver"), ("balanced", "Balanced"), ("performance", "Performance"))]}
# Keyboard backlight levels, from what the driver offers (Off and 1..max; percentages when there are many)
KBD = next(iter(glob.glob("/sys/class/leds/*kbd_backlight*")), None)
if KBD:
    top = int(read(f"{KBD}/max_brightness") or 0)
    names = {1: ["On"], 2: ["Low", "High"], 3: ["Low", "Medium", "High"], 4: ["Low", "Medium", "High", "Max"]}
    levels = ([(0, "Off")] + list(zip(range(1, top + 1), names[top]))) if top in names else \
             [(round(top * p / 100), "Off" if p == 0 else f"{p} %") for p in (0, 25, 50, 75, 100)]
    if top:
        MENUS["menu#kbdlight"] = {"label": tile("\U000F06E9", "Light"), "position": "left", "actions": [
            choice(f"Keyboard light: {name}", B + f"brasa-kbd set {n}", f"{B}brasa-kbd get | grep -qx {n}")
            for n, name in levels]}
# Freeing space: /tmp opens a window listing what would go; RAM drops the page cache
MENUS["menu#clean"] = {"label": tile("\U000F00E2", "Clean"), "position": "left", "actions": [
    {"label": f"{text:<38}", "command": f"sh -c '{CLOSE}; {B}brasa-clean {what}'"}
    for what, text in (("tmp", "\U000F0A7A  Clean /tmp…"), ("ram", "\U000F035B  Free RAM (page cache)"))]}
# Your own buttons, from ~/.config/brasa/panel.conf (see the comments in that file / docs/brasa.md):
#   icon | name | command                              a button
#   icon | name | on command | off command | test      a switch, lit while the test succeeds
CUSTOM = os.path.expanduser("~/.config/brasa/panel.conf")
custom = []
for line in (open(CUSTOM, encoding="utf-8") if os.path.exists(CUSTOM) else []):
    f = [x.strip() for x in line.split("|")]
    if not line.strip() or line.lstrip().startswith("#"):
        continue
    if len(f) not in (3, 5) or any(q in line for q in "\"'"):
        print(f"panel.conf: skipped (needs 3 or 5 fields and no quotes): {line.strip()}", file=sys.stderr)
        continue
    icon, name, *cmds = f
    run = lambda c: f"{B}brasa-run {name.replace(' ', '_')} {c}"
    text = f"{icon}  {name}"
    if len(cmds) == 1:
        custom.append({"label": f"{text:<38}", "command": f"sh -c '{CLOSE}; {run(cmds[0])}'"})
    else:
        custom.append(tog(f"{text:<38}", run(cmds[0]), run(cmds[1]), f"{cmds[2]} && echo true || echo false"))
if custom:
    MENUS["menu#custom"] = {"label": tile("\U000F09AC", "Custom"), "position": "left", "actions": custom}
MENUS = {k: MENUS[k] for k in ("menu#layout", "menu#battery", "menu#power", "menu#kbdlight", "menu#clean",
                            "menu#custom")
         if k in MENUS}
# Hyprland's settings in a window: HyprMod, which writes them to config/hypr/hyprland-gui.lua (not in git).
# A gear in the title row, left of Clear: a second menubar that style.css lays over that row
SETTINGS = {"buttons#settings": {"position": "right", "actions": [
    {"label": "\U000F0493", "command": f"sh -c '{CLOSE}; setsid -f hyprmod >/dev/null 2>&1'"}]}} \
    if shutil.which("hyprmod") else None


# One grid: lit = on, a click turns it on or off. Only what this machine has (no Bluetooth tile without
# Bluetooth); as many columns as fill every row (10 tiles: 5x2, 9: 3x3, 8: 4x2)
TILES = [
      tog(tile("\U000F05A9", "Wi-Fi"), "nmcli radio wifi on", "nmcli radio wifi off",
          "nmcli radio wifi | grep -qx enabled && echo true || echo false"),
      *([tog(tile("\U000F00AF", "Bluetooth"), "bluetoothctl power on", "bluetoothctl power off",
             "bluetoothctl show | grep -q Powered:.yes && echo true || echo false")] if HAS["bluetooth"] else []),
      tog(tile("\U000F0581", "Mute"), "wpctl set-mute @DEFAULT_AUDIO_SINK@ 1", "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0",
          "wpctl get-volume @DEFAULT_AUDIO_SINK@ | grep -q MUTED && echo true || echo false"),
      {"label": tile("\U000F16BE", "Networks"), "command": "sh -c 'swaync-client -cp; nm-connection-editor'"},
      {"label": tile("\U000F09F9", "Keyboard"), "command": f"sh -c '{CLOSE}; {B}brasa-keyboard'"},
      {"label": tile("\U000F1483", "Sessions"), "command": f"sh -c '{CLOSE}; {B}brasa-sessions'"},
      {"label": tile("\U000F075A", "Island"), "command": f"sh -c '{CLOSE}; {B}brasa-island-settings'"},
      tog(tile("\U000F0E51", "Effects"), B + "brasa-fx all", B + "brasa-fx all",
          "test -e $XDG_RUNTIME_DIR/brasa-fx/blur-off && echo false || echo true"),
      {"label": tile("\U000F033E", "Lock"), "command": f"sh -c '{CLOSE}; {B}brasa-lock'"},
      {"label": tile("\U000F0625", "Keys"), "command": f"sh -c '{CLOSE}; {B}brasa-keys'"},
    
]
COLUMNS = next((c for c in (5, 3, 4) if len(TILES) % c == 0), 5)

swaync = {
  "$schema": "/etc/xdg/swaync/configSchema.json",
  "positionX": "right", "positionY": "top", "layer": "overlay", "cssPriority": "user",
  "control-center-margin-top": 8, "control-center-margin-right": 8, "control-center-margin-bottom": 8,
  "control-center-width": 460, "notification-window-width": 400,
  "timeout": 6, "timeout-low": 4, "timeout-critical": 0, "fit-to-screen": True,
  "notification-icon-size": 48, "image-visibility": "when-available", "transition-time": 200,
  "scripts": {"sound": {"exec": "sh -c '$HOME/.config/swaync/sound.sh'", "app-name": ".*", "run-on": "receive"}},
  "widgets": ["title", *(["menubar#corner"] if SETTINGS else []), "dnd", "buttons-grid", "menubar", "volume", "backlight",
              "mpris", "notifications"],
  "widget-config": {
    "title": {"text": "Notifications", "clear-all-button": True, "button-text": "Clear"},
    "dnd": {"text": "Do not disturb"},
    "buttons-grid": {"buttons-per-row": COLUMNS, "actions": TILES},
    "menubar": MENUS,
    **({"menubar#corner": SETTINGS} if SETTINGS else {}),
    "volume": {"label": ICON["vol"][2]},
    "backlight": {"label": ICON["bri"], "device": "intel_backlight"},
    "mpris": {"image-size": 64, "blur": True},
  },
}

os.makedirs(f"{out}/waybar", exist_ok=True); os.makedirs(f"{out}/swaync", exist_ok=True)
json.dump(waybar, open(f"{out}/waybar/config.jsonc", "w"), ensure_ascii=False, indent=2)
json.dump(folded_bar(waybar), open(f"{out}/waybar/compact.jsonc", "w"), ensure_ascii=False, indent=2)

# Left segments alternate two night blues; each triangle takes the color of the block that ends and the
# background of the one that starts (the last one fades into the bar)
SEG = ["@seg1", "@seg2"]
css = ["/* Generated by docs/gen-waybar-swaync.py for this machine's hardware: do not edit by hand. */",
       f"/* Left blocks: {', '.join(LEFT)} */"]
for i, g in enumerate(LEFT):
    nxt = SEG[(i + 1) % 2] if i + 1 < len(LEFT) else "transparent"
    css.append(f"#{g} {{ background: {SEG[i % 2]}; }}")
    css.append(f"#custom-l{i + 1} {{ color: {SEG[i % 2]}; background: {nxt}; }}")
open(f"{out}/waybar/segments.css", "w").write("\n".join(css) + "\n")
print("waybar: left blocks", ", ".join(LEFT), "| hardware", {k: v for k, v in HAS.items()})
json.dump(swaync, open(f"{out}/swaync/config.json", "w"), ensure_ascii=False, indent=2)
# sway has its own layouts: the same panel with sway's in the Layout menu (brasa-autostart starts
# swaync with this file on sway)
sway_panel = json.loads(json.dumps(swaync))
sway_panel["widgets"] = [w for w in sway_panel["widgets"] if w != "menubar#corner"]   # HyprMod: Hyprland only
sway_panel["widget-config"].pop("menubar#corner", None)
sway_panel["widget-config"]["menubar"]["menu#layout"]["actions"] = [
    choice(f"{name:<12} {desc}", B + f"brasa-wm layout-set {mode}", f"{B}brasa-wm layout-current | grep -qx {mode}")
    for name, mode, desc in (("Side by side", "splith", "windows in a row"), ("Above/below", "splitv", "one over another"),
                             ("Tabs", "tabbed", "one at a time, tab bar"), ("Stacked", "stacking", "one at a time, titles"))]
json.dump(sway_panel, open(f"{out}/swaync/config-sway.json", "w"), ensure_ascii=False, indent=2)
open(f"{out}/swaync/island.css", "w").write("/* Generated by docs/gen-waybar-swaync.py: swaync shows its own popups. */\n")
# The menu buttons share the row (480 px: six of 75 px with 6 px gaps, the grid's width), however many
# menus this machine has, so the row always spans the grid. Imported by style.css.
n = len(MENUS)
open(f"{out}/swaync/menus.css", "w").write(
    "/* Generated by docs/gen-waybar-swaync.py: the width of the panel's menu buttons for "
    f"{n} menus. */\n.widget-menubar:not(.corner) > .menu-button-bar > .widget-menubar-container button "
    f"{{ min-width: {(480 - 6 * (n - 1)) / n - 8:.1f}px; }}\n")
