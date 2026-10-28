"""Every tile of the panel, in order, for this machine."""
from . import custom, glyphs, hardware
from .items import Action, Choice, Menu, Switch

B = "~/.local/bin/"
HYPRLAND_LAYOUTS = [("dwindle", "Dwindle · split the focused window"), ("master", "Master · one big + a column"),
                    ("scrolling", "Scroll → · half-screen columns"), ("scrolling-down", "Scroll ↓ · half-screen rows"),
                    ("focus", "Focus → · the focused column big"), ("focus-down", "Focus ↓ · the focused row big")]
SWAY_LAYOUTS = [("splith", "Side by side"), ("splitv", "One above the other"), ("tabbed", "Tabs"),
                ("stacking", "Stacked")]
POWER_PROFILES = [("power-saver", "Power saver"), ("balanced", "Balanced"), ("performance", "Performance")]


def radios():
    tiles = [Switch(glyphs.WIFI, "Wi-Fi", "nmcli radio wifi on", "nmcli radio wifi off",
                    "nmcli radio wifi | grep -qx enabled")]
    if hardware.has_bluetooth():
        tiles.append(Switch(glyphs.BLUETOOTH, "Bluetooth", "bluetoothctl power on",
                            "bluetoothctl power off", "bluetoothctl show | grep -q 'Powered: yes'"))
    tiles.append(Switch(glyphs.AIRPLANE, "Airplane", B + "brasa-radio", B + "brasa-radio",
                        "rfkill list | grep -q 'Soft blocked: yes' && ! rfkill list | grep -q 'Soft blocked: no'"))
    return tiles


def layout_menu():
    layouts = HYPRLAND_LAYOUTS if hardware.in_hyprland() else SWAY_LAYOUTS
    return Menu(glyphs.LAYOUT, "Layout", [
        Choice(label, f"{B}brasa-wm layout-set {layout}", f"{B}brasa-wm layout-current | grep -qx {layout}")
        for layout, label in layouts])


def system_menus():
    menus = []
    if hardware.has_power_profiles():
        menus.append(Menu(glyphs.POWER_PROFILE, "Power", [
            Choice(label, f"{B}brasa-power {profile}", f"powerprofilesctl get | grep -qx {profile}")
            for profile, label in POWER_PROFILES]))
    if hardware.has_charge_limit():
        menus.append(Menu(glyphs.CHARGE_LIMIT, "Charge limit", [
            Choice(f"Charge up to {limit} %", f"{B}brasa-battery {limit}",
                   f"grep -qx {limit} /sys/class/power_supply/BAT*/charge_control_end_threshold")
            for limit in (60, 80, 90, 100)]))
    levels = hardware.keyboard_light_levels()
    if levels:
        menus.append(Menu(glyphs.KEY_LIGHT, "Key light", [
            Choice(name, f"{B}brasa-kbd set {value}", f"{B}brasa-kbd get | grep -qx {value}")
            for value, name in levels]))
    menus.append(Menu(glyphs.CLEAN, "Clean", [
        Choice("Clean /tmp…", f"{B}brasa-clean tmp", "false"),
        Choice("Free RAM (page cache)", f"{B}brasa-clean ram", "false")]))
    return menus


def tiles():
    return [
        *radios(),
        Switch(glyphs.MUTE, "Mute", "wpctl set-mute @DEFAULT_AUDIO_SINK@ 1",
               "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0", "wpctl get-volume @DEFAULT_AUDIO_SINK@ | grep -q MUTED"),
        Switch(glyphs.EFFECTS, "Effects", B + "brasa-fx all", B + "brasa-fx all",
               '! test -e "$XDG_RUNTIME_DIR/brasa-fx/blur-off"'),
        Action(glyphs.NETWORKS, "Networks", "nm-connection-editor"),
        layout_menu(),
        *system_menus(),
        Action(glyphs.KEYBOARD, "Keyboard", B + "brasa-keyboard"),
        Action(glyphs.SESSIONS, "Sessions", B + "brasa-sessions"),
        Action(glyphs.ISLAND, "Island", B + "brasa-island-settings"),
        Action(glyphs.KEYS, "Keys", B + "brasa-keys"),
        Action(glyphs.LOCK, "Lock", B + "brasa-lock"),
        *custom.tiles(),
    ]
