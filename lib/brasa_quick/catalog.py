"""Every tile of the panel, in order, for this machine. The tiles fill pages of six; the mouse wheel goes
from one page to the next."""
from . import custom, hardware
from .items import Action, Choice, Menu, Switch

B = "~/.local/bin/"
HYPRLAND_LAYOUTS = [("dwindle", "Dwindle · split the focused window"), ("master", "Master · one big + a column"),
                    ("scrolling", "Scroll → · half-screen columns"), ("scrolling-down", "Scroll ↓ · half-screen rows"),
                    ("focus", "Focus → · the focused column big"), ("focus-down", "Focus ↓ · the focused row big")]
SWAY_LAYOUTS = [("splith", "Side by side"), ("splitv", "One above the other"), ("tabbed", "Tabs"),
                ("stacking", "Stacked")]
POWER_PROFILES = [("power-saver", "Power saver"), ("balanced", "Balanced"), ("performance", "Performance")]


def radios():
    tiles = [Switch("network-wireless-symbolic", "Wi-Fi", "nmcli radio wifi on", "nmcli radio wifi off",
                    "nmcli radio wifi | grep -qx enabled")]
    if hardware.has_bluetooth():
        tiles.append(Switch("bluetooth-active-symbolic", "Bluetooth", "bluetoothctl power on",
                            "bluetoothctl power off", "bluetoothctl show | grep -q 'Powered: yes'"))
    tiles.append(Switch("airplane-mode-symbolic", "Airplane", B + "brasa-radio", B + "brasa-radio",
                        "rfkill list | grep -q 'Soft blocked: yes' && ! rfkill list | grep -q 'Soft blocked: no'"))
    return tiles


def layout_menu():
    layouts = HYPRLAND_LAYOUTS if hardware.in_hyprland() else SWAY_LAYOUTS
    return Menu("view-grid-symbolic", "Layout", [
        Choice(label, f"{B}brasa-wm layout-set {layout}", f"{B}brasa-wm layout-current | grep -qx {layout}")
        for layout, label in layouts])


def system_menus():
    menus = []
    if hardware.has_power_profiles():
        menus.append(Menu("power-profile-balanced-symbolic", "Power", [
            Choice(label, f"{B}brasa-power {profile}", f"powerprofilesctl get | grep -qx {profile}")
            for profile, label in POWER_PROFILES]))
    if hardware.has_charge_limit():
        menus.append(Menu("battery-level-80-symbolic", "Charge limit", [
            Choice(f"Charge up to {limit} %", f"{B}brasa-battery {limit}",
                   f"grep -qx {limit} /sys/class/power_supply/BAT*/charge_control_end_threshold")
            for limit in (60, 80, 90, 100)]))
    levels = hardware.keyboard_light_levels()
    if levels:
        menus.append(Menu("keyboard-brightness-symbolic", "Key light", [
            Choice(name, f"{B}brasa-kbd set {value}", f"{B}brasa-kbd get | grep -qx {value}")
            for value, name in levels]))
    menus.append(Menu("user-trash-symbolic", "Clean", [
        Choice("Clean /tmp…", f"{B}brasa-clean tmp", "false"),
        Choice("Free RAM (page cache)", f"{B}brasa-clean ram", "false")]))
    return menus


def tiles():
    return [
        *radios(),
        Switch("audio-volume-muted-symbolic", "Mute", "wpctl set-mute @DEFAULT_AUDIO_SINK@ 1",
               "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0", "wpctl get-volume @DEFAULT_AUDIO_SINK@ | grep -q MUTED"),
        Switch("preferences-desktop-appearance-symbolic", "Effects", B + "brasa-fx all", B + "brasa-fx all",
               '! test -e "$XDG_RUNTIME_DIR/brasa-fx/blur-off"'),
        Action("network-workgroup-symbolic", "Networks", "nm-connection-editor"),
        layout_menu(),
        *system_menus(),
        Action("input-keyboard-symbolic", "Keyboard", B + "brasa-keyboard"),
        Action("system-users-symbolic", "Sessions", B + "brasa-sessions"),
        Action("view-dual-symbolic", "Island", B + "brasa-island-settings"),
        Action("preferences-desktop-keyboard-shortcuts-symbolic", "Keys", B + "brasa-keys"),
        Action("system-lock-screen-symbolic", "Lock", B + "brasa-lock"),
        *custom.tiles(),
    ]
