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
                    "tray", "custom/notif"],

  **{f"custom/{n}": {"format": g, "tooltip": False} for n, g in
     [("l1", "\ue0b0"), ("l2", "\ue0b0"), ("l3", "\ue0b0"), ("l4", "\ue0b0"), ("l5", "\ue0b0"), ("l6", "\ue0b0"),
      ("c1", "\ue0b2"), ("c2", "\ue0b0"), ("r0", "\ue0b2"), ("r1", "\ue0b2"), ("r2", "\ue0b2"), ("r3", "\ue0b2")]},
  "group/g-cpu": {"orientation": "horizontal", "modules": ["cpu", "cpu#freq"]},
  "group/g-thermal": {"orientation": "horizontal",
                      "modules": (["custom/temp"] if HAS["temp"] else []) + (["custom/fan"] if HAS["fans"] else [])},
  "group/g-mem": {"orientation": "horizontal", "modules": ["memory", "memory#swap"]},
  "group/g-net": {"orientation": "horizontal", "modules": ["network#rate"]},
  "group/g-power": {"orientation": "horizontal", "modules": ["battery", "custom/volt"]},
  "group/g-screen": {"orientation": "horizontal", "modules": ["custom/hz"]},
  "group/g-ctl": {"orientation": "horizontal",
                  "modules": ["sway/language", "hyprland/language", "network#wifi"] + (["bluetooth"] if HAS["bluetooth"] else [])
                             + ["wireplumber"] + (["backlight"] if HAS["backlight"] else [])},

  "cpu": {"interval": 1, "format": I(ICON["cpu"]) + " {usage:>2}%", "states": load, "tooltip": False},
  "cpu#freq": {"interval": 1, "format": "{avg_frequency:.1f}<small>GHz</small>", "tooltip": False},
  "custom/temp": {"exec": S + " temp", "return-type": "json", "interval": 3},
  "custom/fan": {"exec": S + " fan", "return-type": "json", "interval": 3},
  "memory": {"interval": 1, "states": {"warm": 70, "hot": 85, "crit": 93},
             "format": I(ICON["ram"]) + " {used:.1f}<small>G</small> <span color='#5a6a90'>·</span> {avail:.1f}<small>G</small> <span color='#5a6a90'>·</span> {percentage}%",
             "tooltip-format": "RAM: {used:.1f} of {total:.1f} GiB used, {avail:.1f} GiB available"},
  "memory#swap": {"interval": 5, "format": I(ICON["swap"]) + " {swapPercentage}%",
                  "tooltip-format": "Swap: {swapUsed:.1f} of {swapTotal:.1f} GiB used"},
  "network#rate": {"interval": 2,
                   "format": I(ICON["up"]) + " {bandwidthUpBytes}  " + I(ICON["down"]) + " {bandwidthDownBytes}",
                   "tooltip-format": "{ifname}: {ipaddr}"},
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
                "on-click": "~/.local/bin/brasa-refresh menu", "on-click-right": "~/.local/bin/brasa-refresh cycle",
                "on-click-middle": "~/.local/bin/brasa-hz-real"},

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
  "sway/language": {"format": "{short}", "tooltip-format": "{long}", "on-click": "~/.local/bin/brasa-wm layout-next"},
  "hyprland/language": {"format": "{short}", "on-click": "~/.local/bin/brasa-wm layout-next"},
  "tray": {"icon-size": 16, "spacing": 8},
  "custom/notif": {"exec": "swaync-client -swb", "return-type": "json", "format": "{icon}",
                   "format-icons": {"notification": ICON["bell_new"], "none": ICON["bell"],
                                    "dnd-notification": ICON["bell_off"], "dnd-none": ICON["bell_off"],
                                    "inhibited-notification": ICON["bell_new"], "inhibited-none": ICON["bell"],
                                    "dnd-inhibited-notification": ICON["bell_off"], "dnd-inhibited-none": ICON["bell_off"]},
                   "tooltip": False, "on-click": "swaync-client -t -sw", "on-click-right": "swaync-client -d -sw",
                   "escape": True},
}

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
      choice(f"{name:<10} {desc}", B + f"brasa-wm layout-set {name.lower()}",
             f"{B}brasa-wm layout-current | grep -qx {name.lower()}")
      for name, desc in (("Dwindle", "split the focused window"), ("Master", "one big + a column"),
                         ("Scrolling", "columns you scroll"))]},
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
# What starts with the machine: lit entries are on, a click switches them (the panel stays open)
MENUS["menu#boot"] = {"label": tile("\U000F14DE", "Startup"), "position": "left", "actions": [
    {**tog(f"{'Remote session at boot':<38}", B + "brasa-remote session boot", B + "brasa-remote session noboot",
           "systemctl --user -q is-enabled brasa-remote-session && echo true || echo false")},
    {**tog(f"{'SSH server at boot':<38}", B + "brasa-remote ssh boot", B + "brasa-remote ssh noboot",
           "{ systemctl -q is-enabled sshd || systemctl -q is-enabled ssh; } 2>/dev/null && echo true || echo false")}]}
# Sharing this machine's own desktop (port 5901): one mode at a time
MODE = "$XDG_RUNTIME_DIR/brasa-remote/mode"
MENUS["menu#remote"] = {"label": tile("\U000F1483", "Remote"), "position": "left", "actions": [
    choice("Off       not shared", B + "brasa-remote off", "! systemctl --user -q is-active brasa-wayvnc"),
    *[choice(f"{name:<9} {desc}", f"{B}brasa-remote off; {B}brasa-remote {name.lower()}",
             f"systemctl --user -q is-active brasa-wayvnc && grep -qx {name.lower()} {MODE}")
      for name, desc in (("Mirror", "share this screen"), ("Screen", "add a virtual monitor"),
                         ("Take", "use it remotely, screens off"))]]}
MENUS = {k: MENUS[k] for k in ("menu#layout", "menu#battery", "menu#power", "menu#remote", "menu#boot") if k in MENUS}


swaync = {
  "$schema": "/etc/xdg/swaync/configSchema.json",
  "positionX": "right", "positionY": "top", "layer": "overlay", "cssPriority": "user",
  "control-center-margin-top": 8, "control-center-margin-right": 8, "control-center-margin-bottom": 8,
  "control-center-width": 380, "notification-window-width": 380,
  "timeout": 6, "timeout-low": 4, "timeout-critical": 0, "fit-to-screen": True,
  "notification-icon-size": 48, "image-visibility": "when-available", "transition-time": 200,
  "scripts": {"sound": {"exec": "sh -c '$HOME/.config/swaync/sound.sh'", "app-name": ".*", "run-on": "receive"}},
  "widgets": ["title", "dnd", "buttons-grid", "menubar", "volume", "backlight", "mpris", "notifications"],
  "widget-config": {
    "title": {"text": "Notifications", "clear-all-button": True, "button-text": "Clear"},
    "dnd": {"text": "Do not disturb"},
    # One grid: lit = on, a click turns it on or off
    "buttons-grid": {"buttons-per-row": 3, "actions": [
      tog(tile("\U000F05A9", "Wi-Fi"), "nmcli radio wifi on", "nmcli radio wifi off",
          "nmcli radio wifi | grep -qx enabled && echo true || echo false"),
      tog(tile("\U000F00AF", "Bluetooth"), "bluetoothctl power on", "bluetoothctl power off",
          "bluetoothctl show | grep -q Powered:.yes && echo true || echo false"),
      tog(tile("\U000F0581", "Mute"), "wpctl set-mute @DEFAULT_AUDIO_SINK@ 1", "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0",
          "wpctl get-volume @DEFAULT_AUDIO_SINK@ | grep -q MUTED && echo true || echo false"),
      {"label": tile("\U000F16BE", "Networks"), "command": "sh -c 'swaync-client -cp; nm-connection-editor'"},
      tog(tile("\U000F08B9", "Session"), B + "brasa-remote session", B + "brasa-remote session stop",
          "systemctl --user -q is-active brasa-remote-session && echo true || echo false"),
      tog(tile("\U000F08C0", "SSH"), B + "brasa-remote ssh on", B + "brasa-remote ssh off",
          "{ systemctl -q is-active sshd || systemctl -q is-active ssh; } 2>/dev/null && echo true || echo false"),
      tog(tile("\U000F0E51", "Effects"), B + "brasa-fx all", B + "brasa-fx all",
          "test -e $XDG_RUNTIME_DIR/brasa-fx/blur-off && echo false || echo true"),
      {"label": tile("\U000F033E", "Lock"), "command": f"sh -c '{CLOSE}; {B}brasa-lock'"},
      {"label": tile("\U000F030C", "Keys"), "command": f"sh -c '{CLOSE}; {B}brasa-keys'"},
    ]},
    "menubar": MENUS,
    "volume": {"label": ICON["vol"][2]},
    "backlight": {"label": ICON["bri"], "device": "intel_backlight"},
    "mpris": {"image-size": 64, "blur": True},
  },
}

os.makedirs(f"{out}/waybar", exist_ok=True); os.makedirs(f"{out}/swaync", exist_ok=True)
json.dump(waybar, open(f"{out}/waybar/config.jsonc", "w"), ensure_ascii=False, indent=2)

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
