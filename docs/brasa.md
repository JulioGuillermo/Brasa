# Brasa · documentation

## Contents
1. [What it is](#what-it-is)
2. [Folder layout](#folder-layout)
3. [Installer](#installer)
4. [Key bindings](#key-bindings)
5. [Special keys](#special-keys)
6. [Screen refresh rate](#screen-refresh-rate)
7. [Several monitors](#several-monitors)
8. [Bar (waybar)](#bar-waybar)
9. [Notifications (swaync)](#notifications-swaync)
10. [Menus and dialogs](#menus-and-dialogs)
11. [Window managers](#window-managers)
12. [Effects and transparency](#effects-and-transparency)
13. [On-screen indicators](#on-screen-indicators)
14. [Lock screen](#lock-screen)
15. [Login screen](#login-screen)
16. [Shell: zsh and starship](#shell-zsh-and-starship)
17. [Remote desktop (VNC)](#remote-desktop-vnc)
18. [Colors](#colors)
19. [Trying it inside GNOME](#trying-it-inside-gnome)
20. [Known issues and untested bits](#known-issues-and-untested-bits)

## What it is

A complete theme and configuration for a light tiling desktop, made for a laptop whose RAM and GPU
are shared with language models: bar, notifications, launchers, lock screen, exit dialog and key
bindings, identical in SwayFX and Hyprland. All of it together uses about 150-250 MB, against
~300 MB for gnome-shell alone.

## Folder layout

| Path | What it is | Linked at |
|---|---|---|
| `install.sh` | Installer | — |
| `keys.tsv` | **Every key binding**, single source | `~/.config/brasa/keys.tsv` |
| `wallpaper.png` | Default wallpaper and lock screen background (change it with `brasa-wallpaper`) | `~/.local/share/backgrounds/brasa.png` |
| `data/emoji.tsv` | 3963 Unicode emoji with English names and CLDR keywords | — |
| `bin/brasa-gen` | Generates the sway and Hyprland bindings from `keys.tsv` | `~/.local/bin/` |
| `bin/brasa-wm` | Window actions that work in both (close, kill, focus, workspaces, exit…) | `~/.local/bin/` |
| `bin/brasa-fx` | Turns blur, animations and shadows on/off | `~/.local/bin/` |
| `bin/brasa-refresh` | Screen refresh rate per monitor: dynamic (VRR) or fixed | `~/.local/bin/` |
| `bin/brasa-monitors` | Saves and applies each monitor's layout and refresh rate; re-applies on hotplug | `~/.local/bin/` |
| `bin/brasa-hz-real` | Switches the bar's Hz between the real rate and the compositor's mode | `~/.local/bin/` |
| `bin/brasa-keys` | Searchable cheat sheet (`Super+F1`) | `~/.local/bin/` |
| `bin/brasa-exit` | Centered exit dialog (`Super+Shift+e`) | `~/.local/bin/` |
| `bin/brasa-shot` | Screenshots | `~/.local/bin/` |
| `bin/brasa-reload` | Reloads bar, notifications and menus (`Super+Ctrl+r`) | `~/.local/bin/` |
| `bin/brasa-emoji` | Emoji picker (`Super+.`) | `~/.local/bin/` |
| `bin/brasa-kbd` | Keyboard backlight, for any `*kbd_backlight*` LED | `~/.local/bin/` |
| `bin/brasa-display` | Displays and monitors (wdisplays) | `~/.local/bin/` |
| `bin/brasa-here` | Is a program running in this graphical session? (the local and the remote session each run their own bar, lock...) | `~/.local/bin/` |
| `bin/brasa-battery` | Battery charge limit (e.g. 80 %), kept across reboots; right click on the bar's battery | `~/.local/bin/` |
| `bin/brasa-power` | Power profile (power-profiles-daemon) with an on-screen notice | `~/.local/bin/` |
| `bin/brasa-run` | Runs a custom panel button's command and shows how it went | `~/.local/bin/` |
| `bin/brasa-wallpaper` | Wallpapers of the desktop, lock and login screens, together or each on its own | `~/.local/bin/` |
| `bin/brasa-drawer` | Full-screen app grid with rofi (`Super+d`, again to close) | `~/.local/bin/` |
| `bin/brasa-radio` | Airplane mode (rfkill) | `~/.local/bin/` |
| `bin/brasa-cam` | Tells which app is using the camera | `~/.local/bin/` |
| `bin/brasa-lock` | Locks with hyprlock (swaylock if hyprlock is missing) (`Super+Escape`) | `~/.local/bin/` |
| `bin/brasa-session` | lock / logout / suspend / reboot / poweroff for the exit dialog (logged: `journalctl -t brasa`) | `~/.local/bin/` |
| `bin/brasa-greeter` | Installs, removes or previews the login screen | `~/.local/bin/` |
| `bin/brasa-osd` | On-screen indicator for Brasa's own toggles | `~/.local/bin/` |
| `bin/brasa-remote` | Remote desktop over VNC (wayvnc): own session, virtual monitor, mirror or take-over | `~/.local/bin/` |
| `bin/brasa-autostart` | What starts with the session | `~/.local/bin/` |
| `bin/brasa-test` | Try it inside GNOME | `~/.local/bin/` |
| `config/waybar/` | Bar: modules, style, sensors (`sens`) and real refresh rate (`hz`) | `~/.config/waybar` |
| `config/swaync/` | Notifications, panel and sound (`sound.sh`) | `~/.config/swaync` |
| `config/swayosd/` | On-screen indicators: settings and style | `~/.config/swayosd` |
| `config/fuzzel/` | Quick launcher | `~/.config/fuzzel` |
| `config/rofi/drawer.rasi` | App grid: rofi config and theme (any `~/.config/rofi` is left alone) | read from `~/.config/brasa` |
| `config/nwg-bar/` | Exit dialog buttons and style | `~/.config/nwg-bar` |
| `config/sway/` | sway/SwayFX: `config` (Super), `brasa.conf` (shared), `fx.*.conf` (effects), `test.conf` (Alt), `binds.conf` (generated) | `~/.config/sway` |
| `config/hypr/` | Hyprland: `hyprland.lua`, `binds.lua` (generated) and `hyprlock.conf` | `~/.config/hypr` |
| `config/swaylock/` | swaylock fallback | `~/.config/swaylock` |
| `config/foot/foot.ini` | Terminal: colors, transparency with blur, bell | `~/.config/foot` |
| `greeter/` | Login screen: greetd, nwg-hello and the sway that runs it | `/etc/greetd`, `/etc/nwg-hello` (copied with sudo) |
| `config/kvantum/Brasa/` | Flat Kvantum theme for Qt apps (made by `docs/gen-kvantum.py`) | `~/.config/Kvantum/Brasa` |
| `config/qt/brasa-colors.conf` | Brasa colors for Qt apps without Kvantum (qt5ct / qt6ct, Fusion style) | read from `~/.config/brasa` |
| `config/gtk-3.0/gtk.css`, `config/gtk-4.0/gtk.css` | Brasa colors for GTK3 and GTK4/libadwaita apps | `~/.config/gtk-3.0/`, `~/.config/gtk-4.0/` |
| `docs/gen-waybar-swaync.py` | Generator for waybar's `config.jsonc` and swaync's `config.json` | — |

Everything is a symlink, so editing a file in `~/brasa` changes the live configuration.
Generated files (`binds.conf`, `binds.lua`, `fx.conf`) are not edited by hand.

## Installer

```
./install.sh [--wm swayfx|hyprland|both] [--greeter|--no-greeter] [--zsh|--no-zsh] [--no-gtk] [--no-prompt]
             [--root-prompt] [--no-deps] [--dry-run]
./install.sh --uninstall
```

- `--wm`: which window manager to install (both by default).
- `--greeter` / `--no-greeter`: install greetd + nwg-hello as the login screen instead of the current display manager, or skip it (see [Login screen](#login-screen)). **Without either option the installer asks.**
- `--zsh` / `--no-zsh`: set up zsh with Brasa's config (see [Shell](#shell-zsh-and-starship)) or skip it.
  **Without either option the installer asks.**
- `--no-prompt`: skips starship.
- `--root-prompt`: the same starship prompt for root (asks for sudo). root gets a root-owned copy in
  `/root/.config/starship.toml`, not a link into your home: starship runs the commands in its config,
  and root must not run a file your user can edit. Run it again after changing the prompt.
- `--no-gtk`: skips the GTK colors. By default Brasa links its `gtk.css` for GTK3 and GTK4/libadwaita
  (red accent, night-blue surfaces, red focus rings) and sets the desktop accent color to red; this also
  affects those apps inside GNOME. Any previous `gtk.css` is backed up. It also makes everything dark:
  the `adw-gtk3-dark` theme for GTK3 apps (gsettings and `~/.config/gtk-3.0/settings.ini`) and the dark
  style for GTK4 ones. A `GTK_THEME=` exported in `~/.profile`, `~/.zprofile`, `environment.d` or
  `/etc/environment` overrides all that in every GTK app (dialogs such as the polkit password prompt
  turn light when it names a theme that is not installed): the installer reports it.
  Qt apps (OBS, Krita, Kdenlive, Flameshot...) look the same: with Kvantum installed, the flat **Brasa**
  Kvantum theme (`config/kvantum/Brasa`, linked into `~/.config/Kvantum`, made by `docs/gen-kvantum.py`
  from Kvantum's libadwaita-like KvLibadwaitaMaiaDark: night-blue surfaces, rounded corners, red
  selection and progress); without Kvantum, the Fusion style with `config/qt/brasa-colors.conf`. Either
  is set in `~/.config/qt5ct/qt5ct.conf` and `qt6ct.conf` (backed up first; fonts and icons are left
  as they were). `QT_QPA_PLATFORMTHEME=qt5ct`
  turns it on: set by `hyprland.lua`, by the login screen (`env-vars` in `greeter/nwg-hello.json`, which
  covers sway too) and by `~/.config/environment.d/90-brasa-qt.conf`. Qt 6 apps take it for qt6ct as well.
- `--no-deps`: only links, installs no packages. Running it again is safe (it adds new links).
- `--dry-run`: shows what it would do without doing it.
- `--uninstall`: removes the links, restores the backups (`*.bak-brasa-DATE`) and, if the greeter was installed, switches back to the previous display manager. Packages stay.

**Package manager.** It detects `pacman`, `apt-get`, `dnf`, `zypper` or `apk` (and uses `doas`
when there is no `sudo`). The `PKGS` table in the installer has every dependency's name on each
system; `a|b` tries `a` and then `b`; `-` means not available. It checks what exists in the
repositories before installing and reports what it could not find at the end.

- **Arch/Manjaro**: SwayFX comes from the AUR (via `paru` or `yay`): it tries `swayfx` and, if that fails to
  build (it sometimes does against the current `scenefx`), `swayfx-git`. If any SwayFX is already installed it
  is left alone. If plain `sway` is installed it asks to replace it.
- **Debian/Ubuntu**: no SwayFX; plain sway is used, without blur or rounded corners.
- **Fedora/openSUSE/Alpine**: tries SwayFX, then sway.
- **Hyprland** must be **0.56 or later** (Lua configuration); the installer warns if it is older.
- If Hack Nerd Font is not in the repositories, it is downloaded into `~/.local/share/fonts`.

**When something fails.** Every package step is checked, and after installing the script verifies that
the pieces Brasa needs really exist (waybar, swaync, hyprlock, SwayFX instead of plain sway, Hyprland,
greetd/nwg-hello with `--greeter`…). Anything wrong is listed in red at the end with how to fix it,
and the script exits with an error instead of saying "Ready". The login screen is never switched if its
packages are missing. Running `./install.sh` again after fixing is safe.

**Dependencies** (logical names): waybar, swaync, fuzzel, rofi, nwg-bar, swaybg, swaylock,
swayidle, nm-applet, blueman, brightnessctl, playerctl, wireplumber, paplay, pavucontrol, jq,
python3, notify-send, gtk-launch, xdg-user-dirs, freedesktop sounds, grim, slurp, wl-clipboard,
cliphist, Thunar, polkit agent, xdg portals (gtk, wlr, hyprland), wtype, wdisplays, wlr-randr, starship, wayvnc,
openssl, btop, psmisc,
hyprlock, swayosd, foot, Hack Nerd Font, and the window manager: SwayFX and/or Hyprland.
With zsh: zsh, fzf and git.
With `--greeter`: greetd, nwg-hello and imagemagick (plus sway, if only Hyprland is installed).

## Key bindings

All of them are in `keys.tsv`, one line each, TAB-separated:

```
modifiers   key   action   description   [options]
SUPER+SHIFT q     wm kill  Kill the app (kill -9)
CTRL+SUPER  Up    exec wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+   Volume up   repeat
```

- **Modifiers**: `SUPER`, `SHIFT`, `CTRL`, `ALT`, joined with `+`; `-` for none.
- **Key**: xkb name (`d`, `Return`, `space`, `Left`, `F1`, `Print`, `XF86AudioMute`…).
  `1..9` repeats the line for each number, using `{n}` in the action and description.
- **Action**: `exec <command>` or `wm <verb>`, with `brasa-wm`'s verbs:
  `close`, `kill`, `fullscreen`, `float`, `focus DIR`, `move DIR`, `ws N`, `movews N`,
  `layout-next`, `reload`, `touchpad`.
- **Options**: `locked` (works on the lock screen), `repeat` (repeats while held), `release` (runs when
  the key is released and lets the key through: used for Caps Lock and Num Lock).

After editing: `Super+Shift+r` (regenerates with `brasa-gen` and reloads the window manager).
The cheat sheet (`Super+F1`) reads the same file, so it is always current; Enter runs the binding.

| Keys | Action |
|---|---|
| `Super+d` | App grid (rofi) |
| `Super+r` | Quick launcher (fuzzel) |
| `Super+Enter`, `Super+t` | foot |
| `Super+Shift+Enter`, `Super+Shift+t` | Ghostty |
| `Super+e` | Thunar |
| `Super+Shift+n` | Nautilus |
| `Super+z` | Zen |
| `Super+n` | Notification panel |
| `Super+F1` | Key bindings cheat sheet |
| `Super+.` | Emoji picker |
| `Super+p` | Displays and monitors |
| `Super+Alt+r` | Screen refresh rate menu |
| `Super+q` | Close the window (asks it to close) |
| `Super+Shift+q` | Kill the focused window's app (`kill -9` to its PID, nothing else) |
| `Super+f` | Fullscreen |
| `Super+Space` | Floating / tiled |
| `Super+w` | Window layout of the focused workspace. Hyprland: dwindle (each window splits the focused one) → master (one big window and a column) → scrolling (niri-like columns on a strip you scroll through); each workspace remembers its own. sway: tiles ↔ tabs |
| `Super+←↑→↓`, `Super+hjkl` | Move focus |
| `Super+Shift+←↑→↓`, `Super+Shift+hjkl` | Move the window |
| `Super+1…9` | Go to workspace |
| `Super+Shift+1…9` | Move the window to a workspace |
| `Super+Escape` | Lock the screen |
| `Ctrl+Super+Escape` | Remote keyboard on/off: every key goes to the focused window (automatic in VNC/RDP viewers) |
| `Super+Shift+e` | Exit dialog: lock, log out, suspend, reboot, power off, cancel |
| `Super+Shift+r` | Reload the window manager (and regenerate bindings) |
| `Super+Ctrl+r` | Reload bar, notifications and menus |
| `Super+Alt+Space` | Switch keyboard layout (es ↔ us altgr-intl) |
| `Super+Shift+b` | Blur on/off |
| `Super+Shift+a` | Animations on/off (Hyprland only) |
| `Super+Shift+f` | All effects on/off |
| `Print`, `Super+Shift+s` / `Shift+Print` | Screenshot of an area / of the screen |
| `Ctrl+Super+Space` | Play / pause |
| `Ctrl+Super+←` / `→` | Previous / next track |
| `Ctrl+Super+↑` / `↓` | Volume up / down |
| `Super` + drag / + right button | Move / resize (Hyprland; in sway, `Super`+drag) |

## Special keys

Bindings use the standard key names (`XF86…`), so they work on any laptop or keyboard that has
those keys; if a machine lacks a key, its binding is simply unused.

| Standard key | Action |
|---|---|
| Volume, mute, microphone, brightness, play/pause/next/previous/stop | What they say (also on the lock screen) |
| Keyboard light (`XF86KbdBrightnessUp/Down`, `XF86KbdLightOnOff`) | `brasa-kbd`, with a level notice |
| Display (`XF86Display`) and `Super+p` | `brasa-display` (wdisplays) |
| Emoji (`Super+.`) | `brasa-emoji` |
| Camera (`XF86WebCam`) | `brasa-cam`: which app uses it (blocking, if any, is done by the laptop) |
| Airplane (`XF86RFKill`, `XF86WLAN`) | `brasa-radio` |
| Touchpad (`XF86TouchpadToggle`) | on/off |
| `XF86Launch1` (MyASUS and similar) | System monitor (btop) |
| Calculator, search, explorer, browser, mail, sleep, power | What they say |

**On the Zenbook S 14 (UX5406)**, checked with a key capture: F1-F3 volume, F4-F5 brightness,
F6 sends `Super+P` (displays), F7 `Super+.` (emoji), F8 microphone, F9 camera (`KEY_CAMERA`),
F12 MyASUS (`KEY_PROG1`). The keyboard light is handled by the firmware and sends no event.

## Screen refresh rate

`brasa-refresh` (`Super+Alt+r`, or click the Hz module in a bar; right click cycles) works **per
monitor**: from a bar it changes that bar's monitor (the one under the mouse), from the key the
focused one:

- **Dynamic** (default): variable refresh rate (VRR / adaptive sync). The screen runs at the cap
  while something moves and slows down when the picture is still, saving battery. There is one
  dynamic option per rate the screen offers, e.g. *Dynamic (VRR) 48–120 Hz* and *48–60 Hz*.
- **Fixed**: any rate the screen offers at its current resolution.

Nothing is hard-coded: the rates come from the modes the focused screen reports through the
compositor, and the lower end of the variable range from the screen's EDID range descriptor
(`/sys/class/drm/*/edid`, readable without root; the Zenbook panel says 48-120 Hz). If the EDID has no
range, the menu shows the caps without the minimum.

Each bar shows the **real** rate of its own monitor: `config/waybar/hz` counts the frames that monitor actually produced
in the last 3 seconds (DRM wait-vblank ioctl on `/dev/dri/card*`, no root needed) and adds `VRR` when
variable refresh is on; the tooltip has the exact value and the mode. Each check is one syscall that
takes microseconds and never wakes the GPU; it keeps the vblank interrupt on, but so does the clock
with seconds (every redraw does), so it adds practically nothing. The compositor alone cannot give
this: with VRR it only reports the maximum. Waybar tells the script which monitor its bar is on
(`WAYBAR_OUTPUT_NAME`); the script finds the CRTC driving it (connector → encoder → CRTC through DRM
ioctls) and looks it up again if the monitor is re-plugged. If the monitor is not being driven, it
shows the compositor's mode instead of measuring another screen.

Middle click on the module switches to the compositor's mode only, like GNOME (`≤120Hz VRR`),
remembered in `~/.local/state/brasa/hz-mode-only`.

Each monitor's choice is saved separately (see [Several monitors](#several-monitors)) and restored at
login, after `Super+Shift+r` and when the monitor is plugged in again. On some OLED panels VRR causes slight brightness flicker in dark greys at low
rates; if you see it, use a fixed rate, or set `vrr = 3` in `hyprland.lua` (fullscreen video and
games only).

## Several monitors

Hyprland and sway give every monitor its own workspaces: a workspace lives on the monitor where it was
created, numbers are global (`Super+4` jumps to workspace 4 on whichever monitor holds it) and focus
moves freely between screens. There is one bar per monitor, and its workspace block shows that
monitor's workspaces; its Hz block measures **that** monitor.

`brasa-monitors` keeps each monitor's settings without touching the others. A monitor is identified by
make + model + serial, not by its port, so it is recognized on any port or dock.

- **Layout** (position, scale, rotation, resolution): arrange the monitors with `Super+P` (wdisplays);
  when wdisplays closes the layout is saved for that *set* of monitors
  (`~/.local/state/brasa/monitors/layouts/`). Home, office and laptop-alone each keep their own.
  `brasa-monitors save` saves the current layout by hand.
- **Refresh rate**: per monitor (`~/.local/state/brasa/monitors/refresh/<monitor>`), chosen with
  `brasa-refresh`. Default: dynamic when the monitor's EDID declares a VRR range, otherwise its highest
  rate. VRR is set per monitor, never globally.
- **Applied** at login, after `Super+Shift+r`, and automatically whenever a monitor is plugged or
  unplugged (`brasa-monitors watch`, listening to Hyprland's or sway's output events).

**How it applies them.** Through `wlr-randr`, i.e. the standard wlr-output-management protocol, on both
compositors: live, one monitor at a time, no reload; it also wins over whatever wdisplays set. On
Hyprland the same settings are loaded as monitor rules too (`hl.monitor()`, with the monitor's
description, so they follow it across ports) and written to `~/.local/state/brasa/monitors/hyprland.lua`,
which `hyprland.lua` loads. Hyprland re-applies its monitor rules on every live config change (effects,
window layout, a reload): with the saved values in them, nothing changes and the screen does not flash.
Each rule is loaded twice, because Hyprland takes a new VRR value on the following pass.

`brasa-monitors list` shows what it sees (names, ids, rates, VRR range).

## Bar (waybar)

Floating, with rounded corners, in three zones:

- **Left, resources, always visible** (each group with its icon):
  CPU % and GHz · temperature and both fans · RAM used · available · % and swap ·
  network ↑↓ · battery %, watts and voltage · screen Hz.
- **Center**: workspaces (the active one in red with a glow).
- **Right**: clock (hover: calendar, scroll changes month; click: full date) ·
  keyboard layout · wifi · bluetooth · volume (scroll) · brightness (scroll) · tray · bell.

The blocks are joined with triangles (``, ``) like mo's status bar: they are `custom/l1…l6`,
`c1-c2` and `r1-r3` modules whose color is the block that ends and whose background is the one that
starts (`style.css`). If they look bigger or smaller than the bar, adjust their `font-size`.

**Heat scale**: values change color with load: light blue at rest, white, amber, orange and red
with a glow when critical. Thresholds: CPU 50/75/90 %, RAM 70/85/93 %, temperature 65/80/90 °C,
fans 3000/4500/5500 rpm.

**Sensors** (`config/waybar/sens`), none needs root and all are generic: CPU temperature from
`coretemp` (Intel), `k10temp`/`zenpower` (AMD), `cpu_thermal` (ARM) or `acpitz` (plus NVMe, GPU and
wifi in the tooltip); every fan any hwmon driver exposes (vendor drivers first, two shown, all in the
tooltip); the first battery's voltage. CPU power in watts (RAPL) needs root, so it is not shown.

**Only what the machine has.** The installer runs `docs/gen-waybar-swaync.py`, which checks the
hardware and leaves out what is missing: no battery → no battery block; no temperature sensor and no
fans → no thermal block; no backlight → no brightness control; no bluetooth → no bluetooth icon. The
alternating block colors and the triangles are recomputed for the blocks that remain
(`config/waybar/segments.css`). On a desktop the left side is simply CPU · temperature · RAM · Hz.
Run it again after changing hardware: `python3 docs/gen-waybar-swaync.py config`.

To change modules, edit `docs/gen-waybar-swaync.py` and run
`python3 docs/gen-waybar-swaync.py config`, or edit `config/waybar/config.jsonc` directly.

## Notifications (swaync)

- Cards at the top right, night blue, red title, with icon/image and action buttons.
- Critical ones: red border and glow, they do not go away by themselves.
- **Sound**: a bell (`bell.oga`) for normal ones, `dialog-warning.oga` for critical ones, nothing
  for low-urgency ones (Brasa notices) or in "do not disturb". Script: `config/swaync/sound.sh`.
- **Panel** (`Super+n` or click the bell): history, "do not disturb", wifi and bluetooth buttons
  (red when on), mute, network; volume and brightness sliders; media player.

## Menus and dialogs

- **App grid** (`Super+d`, `bin/brasa-drawer`): rofi full screen, a 7-column grid with search; type to
  filter, one click or Enter opens, `Escape` or `Super+d` again closes. Opens in ~50 ms and uses ~14 MB
  only while open (nwg-drawer, used before, took ~1.5 s and ~70 MB). rofi does not handle touch
  input: with a finger, use the keyboard or the touchpad instead.
- **fuzzel** (`Super+r`): type and Enter. Also used by the cheat sheet, emoji and refresh menus.
- **nwg-bar** (`Super+Shift+e`): centered dialog with Lock · Log out · Suspend · Reboot · Power off · Cancel.
  Every button calls `brasa-session <action>`: nwg-bar runs commands without a shell (so no `~`,
  quotes or pipes) and quits right after the click, so the script detaches and logs each action
  (`journalctl -t brasa`). Suspend/reboot/power off use `systemctl` on systemd and `loginctl` on elogind systems.
- **Lock screen**: see [Lock screen](#lock-screen).

## Window managers

**SwayFX** (`config/sway/`): 10 px rounded corners, blur, soft red shadows, slight dimming of
unfocused windows, and blur behind the bar, notifications and launcher. With plain sway the effects
are skipped automatically (`fx.none.conf`).

**Hyprland** (`config/hypr/hyprland.lua`, Lua, 0.56+): the same plus animations with a small "pop"
when windows open, a red gradient border, and a three-finger touchpad swipe to switch workspaces.

Shared: keyboard `es` + `us altgr-intl` (compose on Insert), repeat 500 ms / 33 per second,
tap to click and natural scrolling on the touchpad, touchscreen and pen mapped to the internal
screen, `Future-cyan-cursors`, 133 % scale on `eDP-1`, dynamic refresh rate.

**Touchscreen gestures**: Hyprland supports them with the hyprgrass plugin, not in the installer yet
(it still has to be checked against the Lua configuration).

## Effects and transparency

`brasa-fx blur|anim|shadow|all` (`Super+Shift+b/a/f`) changes the state live and shows a short
silent notice. On Hyprland it uses `hyprctl eval 'hl.config(...)'`; on SwayFX, `swaymsg blur/shadows`.
After a reload or a new session, effects are on again. Turn them off while measuring something on the GPU.

Blur is light on purpose (Hyprland `size 5, passes 2`; SwayFX `blur_radius 5, blur_passes 2`).

**Transparent terminals and blur.** Hyprland 0.56 only blurs behind an app that uses the
`ext-background-effect-v1` protocol in the area the app asks for. **foot** uses that protocol, so
it needs `blur=yes` next to `alpha` in `~/.config/foot/foot.ini`:

```ini
[colors-dark]
alpha=0.82
blur=yes
```

Apps that do not use the protocol are blurred normally wherever they are transparent.

## On-screen indicators

**SwayOSD** shows a night-blue pill near the bottom of the screen, with a red edge and a red bar,
whenever something changes:

- **Volume** (and mute), **microphone** mute, **screen brightness**: the keys call
  `swayosd-client`, which changes the value and shows it.
- **Keyboard light**: with `brasa-kbd` (a segmented bar, e.g. 2 of 3), and also when the firmware
  changes it by itself (the Zenbook's Fn key), thanks to `keyboard_backlight = true`.
- **Caps Lock** and **Num Lock**.
- **Music**: play/pause/next/previous show the track.
- **Brasa's toggles** through `brasa-osd`: blur/animations/effects, refresh rate, keyboard layout,
  airplane mode, touchpad.

Settings in `config/swayosd/config.toml` (height, duration, max volume); look in `style.css`.
Without SwayOSD, `brasa-osd` falls back to a quiet notification.

## Lock screen

`brasa-lock` (`Super+Escape`, the exit dialog's Lock, idle and suspend) uses **hyprlock** on both
Hyprland and sway/SwayFX (`config/hypr/hyprlock.conf`), designed as the same centered card as the
login screen: blurred wallpaper, "Welcome back!", big red clock, date, round avatar with a red ring
(`~/.face`, linked by the installer to the account picture), user name, night-blue password field,
keyboard layout. Plain swaylock (`config/swaylock/config`) is only a fallback if hyprlock is missing.

The session locks after 15 minutes idle, the screen turns off after 20, and it always locks before
suspending (`brasa-autostart`, through swayidle).

## Login screen

**greetd + nwg-hello** replace GDM with a much lighter login screen styled like the rest of Brasa:
the wallpaper blurred and darkened, a centered night-blue card with rounded corners and a red edge,
a large red clock,
the date, your round avatar, the session selector (Hyprland, SwayFX, GNOME…), the user, the
password, and suspend/reboot/power off buttons. greetd starts a minimal sway that only runs
nwg-hello (`greeter/sway-config`) and exits when you log in. The centered card comes from
`greeter/brasa.glade`, a copy of nwg-hello's layout template with the form centered (nwg-hello
only loads templates from `/etc/nwg-hello`).

```
brasa-greeter test        # preview it in a window, nothing changes
./install.sh              # asks whether to use it (or ./install.sh --greeter to say yes up front)
brasa-greeter install     # same, on its own (asks; --yes skips the question)
brasa-greeter uninstall   # go back to the previous display manager
brasa-greeter bg          # regenerate the blurred background after changing the wallpaper
```

- `install` **asks before changing anything** (`--yes` skips the question), then copies `greeter/` to `/etc/nwg-hello` and `/etc/greetd/greetd.conf` (greetd reads it
  before `config.toml`, so package upgrades do not overwrite it), creates the blurred background in
  `/usr/share/nwg-hello/brasa-bg.jpg`, remembers the current display manager, disables it and enables
  greetd. **It takes effect on the next boot**; the running session is not touched.
- **If something goes wrong** and there is no login screen: `Ctrl+Alt+F3`, log in on the text
  console and run `~/brasa/bin/brasa-greeter uninstall`, then reboot.
- GNOME still works from greetd, but GNOME's "switch user" needs GDM.

## Shell: zsh and starship

**starship** (`config/starship/starship.toml`, linked to `~/.config/starship.toml`) draws the prompt
with the bar's look: night-blue blocks joined by triangles, red icons, white text.

- Line 1, left: OS icon · `user@host` (only over SSH) · directory (repository root in red) · git branch
  and status · project language (Go, Rust, C, Python, Node, Lua) · docker context.
- Line 1, right: duration of the last command (over 2 s) · time.
- Line 2: `❯` in red; `✘ <code>` in amber when the last command failed; background jobs and an active
  `sudo` are shown too.

It is added to the login shell: Brasa's `~/.zshrc` starts it; for bash or fish the installer appends
one marked line to `~/.bashrc` / `~/.config/fish/config.fish` (backed up first; `--uninstall` removes it).

**zsh** (`config/zsh/`), when chosen:

- The previous zsh config (`~/.zshrc`, `~/.zshrc_*`, `~/.zsh_*.zsh`, `~/.p10k.zsh`, `~/.zcompdump*`) is
  **moved to `~/zsh_backup/<date>/`**, and `~/.zshrc` becomes Brasa's: zap with zsh-autosuggestions,
  supercharge, zsh-syntax-highlighting and fzf-tab; case-insensitive completion; 100 000 lines of history
  without duplicates; emacs keys; syntax highlighting, suggestions and fzf in the theme's colors (named
  colors inside Zed, so they follow Zed's theme); starship.
- Your own settings live in `~/.config/zsh/`, created with an explanatory comment **only if they do not
  exist** (otherwise they are used as they are, never overwritten):
  - `path.zsh`: PATH and environment variables, loaded first;
  - `aliases.zsh`: aliases;
  - `tools.zsh`: tool set-ups and functions (nvm, rbenv, gcloud, completions…), loaded before the prompt;
  - `history`: the command history (`~/.zsh_history` is moved here; mode 600).
- zap is cloned into `~/.local/share/zap` if missing (plugins download on the first start), and zsh
  becomes the login shell (`chsh`, asks for the password).
- `~/.zshrc` is a copy, not a link, so installers that append to it still work; move what they add into
  `~/.config/zsh/tools.zsh` when convenient. Re-running the installer updates it.
- `--uninstall` restores the previous config from the latest `~/zsh_backup/<date>/` and moves the history
  back; `~/.config/zsh/` and the login shell stay.

## Remote desktop (VNC)

`brasa-remote` shares the desktop with **wayvnc**, which captures straight from the compositor: no
"which screen do you want to share?" prompt, and remote keyboard and mouse work. Everything is off
by default. Two independent servers, which can run at the same time:

| Port | What | Modes |
|---|---|---|
| 5900 | your remote desktop, a session of its own | `session` |
| 5901 | the local desktop, whoever is logged in on the machine | `screen`, `mirror`, `take` (one at a time) |

All of it can be controlled over SSH: `session` does not need a local desktop at all, and the local
modes find the running Hyprland or sway by themselves (also when run from inside the remote session).

```
brasa-remote session [stop|boot|noboot]  an independent remote-only session (the default for "on")
brasa-remote session size auto|WxH       auto: 1920x1080, then the viewer's size; WxH: always that size
brasa-remote session scale auto|N        auto: from the height; N: fixed, e.g. 1.25
brasa-remote session defaults            size and scale back to auto
brasa-remote session settings            show them
brasa-remote screen [WxH]                a virtual second monitor in the current session
brasa-remote mirror                      share the physical screen (the focused monitor)
brasa-remote take [WxH]                  take over the current session; the laptop goes dark
brasa-remote ssh [on|off|boot|noboot]   the SSH server, now and at boot
brasa-remote on | off | status | setup | help
```

- **session**: a second, separate desktop that only exists remotely: sway with no physical screen
  (`WLR_BACKENDS=headless`, `config/sway/remote.conf`), running as the user service
  `brasa-remote-session`. It has its own bar, workspaces and windows and does not touch the local
  session (which may be Hyprland, locked, or nobody logged in). Closing the viewer leaves it running
  with its windows; `session stop` ends it. `session boot` starts it when the machine boots, before
  anyone logs in (enables `loginctl enable-linger`); `session noboot` undoes it. Its size is
  `REMOTE_SIZE`. It only starts at boot after `session boot`. The service never inherits the local compositor's variables, so keys and scripts in
  the remote session act on the remote session only.
- **screen**: a headless output created in the current compositor (Hyprland `output create headless`,
  sway `create_output`) at the requested size, scale 1. Move windows/workspaces to it with
  `Super+Alt+Shift+←/→`. `off` removes it. To switch between screen / mirror / take: `off`, then the new one.
- **mirror**: the physical screen as it is, shared with whoever is in front of it.
- **take**: the physical screens are switched off, local keyboard, mouse and touchscreen are ignored,
  and the session moves to a virtual monitor. When the last viewer disconnects, or nobody connects
  within `REMOTE_TAKE_WAIT` seconds (180), everything comes back and the session is locked.
- **ssh**: turns the system's SSH server (`sshd`, or `ssh` on Debian/Ubuntu) on or off now, and
  makes it start at boot or not; asks for sudo. Turning it off keeps open connections (only new ones
  are refused), so it is safe to run over SSH, but you will not be able to come back in until it is
  on again. `status` shows it too. Every on/off of every mode shows a 5-second notification (a click
  closes it) with the mode, its port and what else is on.
- **Where it listens**: only on a VPN interface (ZeroTier `zt*`, WireGuard `wg*`, Tailscale, OpenVPN
  `tun*`...), never on every network. Settings in `~/.config/brasa/remote.conf`:
  `REMOTE_DEFAULT=` (mode for `on`), `REMOTE_ADDRESS=`, `REMOTE_PORT=` (session, 5900),
  `REMOTE_LOCAL_PORT=` (local modes, 5901), `REMOTE_SIZE=`, `REMOTE_SCALE=`, `REMOTE_RESIZE=`,
  `REMOTE_TAKE_WAIT=`.
- **Size and scale**: the remote session starts at `REMOTE_SIZE` (1920x1080) and then takes the
  viewer's size: TigerVNC asks for its window size, so in full screen (`Ctrl+Alt+Enter`) each remote
  pixel is one screen pixel, sharp. VNC tells the size but not the scale, so `REMOTE_SCALE=auto` picks
  it from the height: up to 1200 px 100 %, 1500 px 125 %, 1900 px 133 %, 2200 px 150 %, more 200 %.
  `brasa-remote session size|scale|defaults` change it (they write `remote.conf` and apply at once to
  a running session; whether it follows the viewer changes from the next start). By hand:
  `REMOTE_SIZE=`, `REMOTE_SCALE=1.5`, `REMOTE_RESIZE=no`. On a Hyprland viewer, TigerVNC (an X11
  app) is only sharp with the XWayland setting below.
- **Login**: the Linux user and password (PAM). Encrypted with RSA-AES (TigerVNC ≥ 1.13, RealVNC) or
  TLS / VeNCrypt. Keys live in `~/.config/wayvnc/` (mode 600). Remmina's VNC plugin offers neither and
  hangs at the handshake: use TigerVNC.
- wayvnc for screen / mirror / take runs as a transient user service (`brasa-wayvnc`), so it survives
  closing the terminal or SSH session that started it. Logs: `journalctl --user -u brasa-wayvnc` or
  `-u brasa-remote-session`.
- **Remote keyboard**: X11 viewers such as TigerVNC cannot grab the keyboard on Wayland, so the window
  manager would keep Super and its shortcuts. While a VNC/RDP/VM viewer has the focus (TigerVNC,
  FreeRDP, wlvncc, virt-viewer) Brasa switches to a "remote" key mode with no shortcuts: every key goes
  to the remote machine. Focusing another window (with the mouse) brings the shortcuts back, and
  `Ctrl+Super+Escape` leaves or re-enters the mode by hand, e.g. with the viewer in full screen.
  Hyprland does it in `hyprland.lua`; sway with `brasa-wm remote-watch`, started by `brasa-autostart`.
- **Clients**: TigerVNC Viewer (Windows, Linux, macOS) or RealVNC Viewer. In TigerVNC 1.16:
  `Ctrl+Alt` releases the keyboard grab, `Ctrl+Alt+G` grabs again, `Ctrl+Alt+M` opens the menu,
  `Ctrl+Alt+Enter` toggles full screen. `vncviewer 10.x.x.x:5900` (session) or `:5901` (local desktop).

## Quick settings (notification panel)

The panel (`Super+n` or the bell) has one grid of tiles, lit while on, and a row of menus whose current
choice is lit:

| Tile | Click |
|---|---|
| Wi-Fi · Bluetooth · Mute | on / off |
| Networks | Wi-Fi and network settings (nm-connection-editor) |
| Session | the remote session, `brasa-remote session` (port 5900) |
| SSH | the SSH server; asks for the password through the system's dialog (polkit) |
| Effects | blur, animations and shadows, like `Super+Shift+f` |
| Lock | lock the screen |
| Keys | the key bindings cheat sheet, like `Super+F1` |

| Menu | Choices |
|---|---|
| Layout | the focused workspace's window layout: dwindle, master, scrolling (like `Super+w`) |
| Battery | charge limit 60 / 80 / 90 / 100 % (laptops that support it, see below) |
| Power | power profile: power saver, balanced, performance (`brasa-power`, with power-profiles-daemon) |
| Remote | sharing this machine's desktop on port 5901: off, mirror, screen, take |
| Startup | what starts with the machine: the remote session, the SSH server (lit = on; the panel stays open) |
| Custom | your own buttons, from `~/.config/brasa/panel.conf` (only when that file has some) |

Choosing in a menu closes the panel (except Startup, whose entries are switches). swaync cannot show a
spinner on a button, so remote and SSH changes show a "Working…" notice at once, replaced by the result. The panel is generated for each machine by
`docs/gen-waybar-swaync.py`, so menus a machine cannot use are left out. swaync (0.12) has no tooltips
for these buttons, hence the words under the icons; and it mangles double quotes inside commands, so the
panel's commands avoid them.

### Your own buttons (Custom)

`~/.config/brasa/panel.conf` (yours, not in git), one button per line:

```
# icon | name | command                                   a button
# icon | name | on command | off command | state test     a switch, lit while the test succeeds
󰃢 | Free RAM  | ~/.local/bin/free-ram
󰢮 | GPU awake | ~/.local/bin/gpu-awake on | ~/.local/bin/gpu-awake off | systemctl -q is-active gpu-forcewake
```

After editing, `Super+Ctrl+r` (`brasa-reload`) rebuilds the panel. Each command runs through
`brasa-run`, which shows "Working…" at once and then the command's last line of output (or the error).
Commands cannot contain quotes (swaync mangles them): anything more complex goes in a script. For
actions that need root, the script can use `pkexec`, which asks for the password with a dialog (the
polkit agent Brasa starts), e.g. `pkexec sh -c 'sync; echo 3 > /proc/sys/vm/drop_caches'`.

## Battery charge limit

```
brasa-battery          show the current limit
brasa-battery 80       charge up to 80 % (20-100)
brasa-battery menu     choose 60 / 80 / 90 / 100 % (also: right click on the bar's battery)
```

Charging stops at the limit, which keeps a battery healthier when the laptop lives on the charger.
It needs a kernel driver that offers `charge_control_end_threshold` (ASUS, Lenovo, Dell, Huawei,
Samsung, Framework, System76...). The first change asks for sudo once: it installs
`/etc/udev/rules.d/90-brasa-battery.rules`, which applies the saved limit (`/etc/brasa/battery-limit`)
at every boot and lets your user change it without sudo afterwards. GNOME's "Preserve battery health"
sets the same value through UPower; the last one to change it wins.

## X11 apps on a scaled screen

TigerVNC and other X11 apps run through XWayland. With a fractional scale (133 %), a compositor
normally draws them at 100 % and blows them up, which makes them blurry. On Hyprland, Brasa uses the
same trick as GNOME instead: `xwayland:force_zero_scaling` lets them see the screen at its real
resolution, and `brasa-monitors apply` tells them the main monitor's scale through `Xft.dpi` (and the
cursor size through `Xcursor.size`), so the apps that honour it (GTK, Qt, FLTK 1.4, Electron...) draw
themselves at the right size, sharp. Old X11 apps that ignore `Xft.dpi` look small but sharp. Apps read
it when they start; with several monitors of different scales there is one value, the focused one's.
sway/SwayFX have no such option: there X11 apps are still scaled up.

## Wallpaper

Each screen can have its own wallpaper:

| Screen | Seen | Image | Drawn by |
|---|---|---|---|
| desktop | with the session open | `~/.local/share/backgrounds/brasa.png` | swaybg (Hyprland) or sway, as you |
| lock | when the screen locks | `~/.local/share/backgrounds/brasa-lock.png` | hyprlock / swaylock, as you |
| login | at boot and after logging out | `/usr/share/nwg-hello/brasa-bg.jpg` (blurred copy of `brasa-login.png`) | nwg-hello, as the `greeter` user |

```
brasa-wallpaper [SCREEN...] IMAGE             SCREEN: desktop, lock, login (one or several, any order)
brasa-wallpaper ~/Pictures/a.jpg              all three (lock and login follow the desktop)
brasa-wallpaper desktop ~/Pictures/a.jpg      only the desktop
brasa-wallpaper lock login ~/Pictures/b.jpg   the lock and login screens
brasa-wallpaper desktop login ~/Pictures/c.jpg
brasa-wallpaper lock default                  Brasa's own (any screens)
brasa-wallpaper lock login same               lock follows the desktop, login the lock screen
brasa-wallpaper reset                         Brasa's own everywhere, following each other again
brasa-wallpaper                               show the current ones
```

- By default the lock screen follows the desktop and the login screen follows the lock screen, so
  changing the desktop changes all three until one of them gets its own image.
- A copy of each image is kept in `~/.local/share/backgrounds/` (`brasa-custom.*`,
  `brasa-lock-custom.*`, `brasa-login-custom.*`), so the originals can be moved or deleted, and
  `./install.sh` leaves chosen wallpapers alone.
- The desktop changes at once on every running session (the remote one too); on Hyprland swaybg runs
  as the user service `brasa-swaybg-<display>`, so it survives the terminal or SSH session it was
  changed from. The lock screen changes the next time it locks. Neither needs root.
- The login screen runs before you log in, as another user that cannot read your home, so its blurred
  copy lives in `/usr/share/nwg-hello/` and writing it needs sudo. `brasa-wallpaper` only does that when
  the login image really changes, asking for the password when run from a terminal; otherwise it says
  to run `brasa-greeter bg`.
- Formats: png, jpg, webp, bmp.

## Colors

Taken from the user's terminal and mo's Redline theme:

| Use | Color |
|---|---|
| Background | `#00040f` |
| Bar blocks | `#0c1838`, `#08122b`, `#040b1f` |
| Accent (icons, active, borders) | `#ff0000` (window border: `#d00000` to `#4a0008`) |
| Text | `#ffffff` |
| Secondary text | `#c3cbe0` |
| Dimmed | `#5a6a90`, `#2c3a5e` |
| Idle / cool | `#99ccff` |
| Warning | `#ffaa33` |
| High | `#ff7a3d` |

## Trying it inside GNOME

`brasa-test` opens sway inside a window, with its own session bus so it does not clash with
GNOME's notifications. The modifier is **Alt** (GNOME keeps Super and Alt+Space). It looks a bit
blurry because GNOME scales the window to 133 %; the real session is sharp. From a TTY
(`Ctrl+Alt+F3`) you can also run `sway` directly at native resolution.

## Known issues and untested bits

- The Hyprland touchpad toggle uses `hl.device({ name, enabled })`, untested.
- **nwg-bar** uses icon theme names (`system-lock-screen`…); if the icon theme lacks them they show empty.
- Package names for apt, dnf, zypper and apk were written from memory; the installer reports the ones it cannot find.
- `Super+/` is not used for the cheat sheet because `/` needs Shift on a Spanish keyboard.
- On the Zenbook, the camera key's light is driven by the ASUS firmware; Linux exposes no control for it.
