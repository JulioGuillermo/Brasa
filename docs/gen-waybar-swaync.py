# Generates config/waybar/config.jsonc, config/waybar/segments.css and config/swaync/config.json for
# the Brasa theme, for the hardware of this machine: blocks for things it does not have (battery, fans,
# temperature sensors, backlight, bluetooth) are left out, and the alternating segment colors and the
# triangles between them are computed for the blocks that remain. The installer runs it.
# Usage: python3 docs/gen-waybar-swaync.py config   (from the brasa folder)
import glob, json, os, sys
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
  "modules-center": ["custom/c1", "sway/workspaces", "hyprland/workspaces", "custom/c2"],
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

  # Each bar shows its own monitor's workspaces: the number, inside a circle drawn in style.css
  "sway/workspaces": {"format": "{name}", "disable-scroll": False, "all-outputs": False,
                      "persistent-workspaces": {str(n): [INTERNAL] if INTERNAL else [] for n in range(1, 5)}},
  "hyprland/workspaces": {"format": "{name}", "all-outputs": False,
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

def tog(label, on, off, state):
    return {"label": label, "type": "toggle", "command": f"sh -c '[ \"$SWAYNC_TOGGLE_STATE\" = true ] && {on} || {off}'",
            "update-command": f"sh -c '{state}'"}

swaync = {
  "$schema": "/etc/xdg/swaync/configSchema.json",
  "positionX": "right", "positionY": "top", "layer": "overlay", "cssPriority": "user",
  "control-center-margin-top": 8, "control-center-margin-right": 8, "control-center-margin-bottom": 8,
  "control-center-width": 380, "notification-window-width": 380,
  "timeout": 6, "timeout-low": 4, "timeout-critical": 0, "fit-to-screen": True,
  "notification-icon-size": 48, "image-visibility": "when-available", "transition-time": 200,
  "scripts": {"sound": {"exec": "sh -c '$HOME/.config/swaync/sound.sh'", "app-name": ".*", "run-on": "receive"}},
  "widgets": ["title", "dnd", "buttons-grid", "volume", "backlight", "mpris", "notifications"],
  "widget-config": {
    "title": {"text": "Notifications", "clear-all-button": True, "button-text": "Clear"},
    "dnd": {"text": "Do not disturb"},
    "buttons-grid": {"actions": [
      tog(ICON["net"], "nmcli radio wifi on", "nmcli radio wifi off",
          "[ \"$(nmcli radio wifi)\" = enabled ] && echo true || echo false"),
      tog(ICON["bt"], "bluetoothctl power on", "bluetoothctl power off",
          "bluetoothctl show | grep -q \"Powered: yes\" && echo true || echo false"),
      {"label": ICON["mute"], "command": "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"},
      {"label": "\U000F0493", "command": "sh -c 'swaync-client -cp; nm-connection-editor'"},
    ]},
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
