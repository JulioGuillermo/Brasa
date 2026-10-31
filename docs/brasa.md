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
9. [Notifications](#notifications)
10. [Menus and dialogs](#menus-and-dialogs)
11. [Window managers](#window-managers)
12. [Effects and transparency](#effects-and-transparency)
13. [On-screen indicators](#on-screen-indicators)
14. [Island](#island)
15. [Lock screen](#lock-screen)
16. [Login screen](#login-screen)
17. [Shell: zsh and starship](#shell-zsh-and-starship)
18. [Remote desktop (VNC and RDP)](#remote-desktop-vnc-and-rdp)
19. [Wallpaper](#wallpaper)
20. [Icons and cursors](#icons-and-cursors)
21. [Colors](#colors)
22. [Trying it inside GNOME](#trying-it-inside-gnome)
23. [Known issues and untested bits](#known-issues-and-untested-bits)

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
| `wallpaper.svg` | Source of `wallpaper.png`: `rsvg-convert -w 3840 -h 2160 wallpaper.svg -o wallpaper.png` | — |
| `data/wallpapers/` | Other wallpapers: the earlier campfire and peaks drafts, and the previous default | — |
| `data/emoji.tsv` | 3963 Unicode emoji with English names and CLDR keywords | — |
| `bin/brasa-gen` | Generates the sway and Hyprland bindings from `keys.tsv` | `~/.local/bin/` |
| `bin/brasa-wm` | Window actions that work in both (close, kill, focus, workspaces, exit…) | `~/.local/bin/` |
| `bin/brasa-fx` | Turns blur, animations and shadows on/off | `~/.local/bin/` |
| `bin/brasa-refresh` | Screen refresh rate per monitor: dynamic (VRR) or fixed | `~/.local/bin/` |
| `bin/brasa-monitors` | Saves and applies each monitor's layout and refresh rate; re-applies on hotplug | `~/.local/bin/` |
| `bin/brasa-bar` | Starts waybar, one bar per monitor: folded on narrow monitors | `~/.local/bin/` |
| `bin/brasa-hz-real` | Switches the bar's Hz between the real rate and the compositor's mode | `~/.local/bin/` |
| `bin/brasa-keys` | Key bindings window (`Super+F1`): every binding the window manager really has, by kind | `~/.local/bin/` |
| `lib/brasa_ui.py`, `lib/brasa.css` | The look of Brasa's own windows (the notification panel's), and no window buttons under Hyprland or sway (Escape closes) | read from `~/.config/brasa` |
| `lib/brasa_binds.py` | Reads those bindings from Hyprland (`hyprctl binds` and the Lua files) or sway (its config) | read from `~/.config/brasa` |
| `bin/brasa-sessions` | Sessions window (`Super+Ctrl+s`): remote session, sharing, SSH, dialogs and keyring here | `~/.local/bin/` |
| `bin/brasa-follow` | Sends the dialogs of the user's D-Bus (passwords, keyring, portals) to this session; locks and unlocks the keyring | `~/.local/bin/` |
| `bin/brasa-clean` | Clean menu (`Super+Ctrl+c`): a window to clean `/tmp`, and freeing RAM | `~/.local/bin/` |
| `bin/brasa-tmpclean` | Finds the leftovers in `/tmp` that nothing uses, and removes them | `~/.local/bin/` |
| `bin/brasa-exit` | Centered exit dialog (`Super+Shift+e`) | `~/.local/bin/` |
| `bin/brasa-shot` | Screenshots | `~/.local/bin/` |
| `bin/brasa-freeze` | Still image over every monitor while `brasa-shot area` (`Shift+Print`) picks the area | `~/.local/bin/` |
| `islands/` | `brasa-islands`, in Rust: the island (music, notifications, `Super+m`), the app launcher (`Super+d`), quick settings (`Super+a`) and the window switcher (`Super+Tab`), one process | `~/.local/bin/` (built by `install.sh` and `brasa-reload`) |
| `bin/brasa-island-settings`, `lib/brasa_island_config.py` | Island window (`Super+Shift+m`, quick settings tile **Island**): music, notifications, launcher, quick settings, where each one sits | `~/.local/bin/` |
| `bin/brasa-reload` | Reloads bar, notifications and menus (`Super+Ctrl+r`) | `~/.local/bin/` |
| `bin/brasa-emoji` | Emoji picker (`Super+.`) | `~/.local/bin/` |
| `bin/brasa-kbd` | Keyboard backlight, for any `*kbd_backlight*` LED: up, down, toggle, set N, get, max | `~/.local/bin/` |
| `bin/brasa-display` | Displays and monitors (wdisplays) | `~/.local/bin/` |
| `bin/brasa-here` | Is a program running in this graphical session? (the local and the remote session each run their own bar, lock...) | `~/.local/bin/` |
| `bin/brasa-battery` | Battery charge limit (e.g. 80 %), kept across reboots; right click on the bar's battery | `~/.local/bin/` |
| `bin/brasa-power` | Power profile (power-profiles-daemon) with an on-screen notice | `~/.local/bin/` |
| `bin/brasa-run` | Runs a custom panel button's command and shows how it went | `~/.local/bin/` |
| `bin/brasa-keyboard` | Keyboard layouts: pick, add, remove (searchable menu) | `~/.local/bin/` |
| `bin/brasa-wallpaper` | Wallpapers of the desktop, lock and login screens, together or each on its own | `~/.local/bin/` |
| `bin/brasa-drawer` | Full-screen app grid with rofi (`Super+Shift+d`, again to close) | `~/.local/bin/` |
| `bin/brasa-radio` | Airplane mode (rfkill) | `~/.local/bin/` |
| `bin/brasa-cam` | Tells which app is using the camera | `~/.local/bin/` |
| `bin/brasa-lock` | Locks with hyprlock (swaylock if hyprlock is missing) (`Super+Escape`) | `~/.local/bin/` |
| `bin/brasa-session` | lock / logout / suspend / reboot / poweroff for the exit dialog (logged: `journalctl -t brasa`) | `~/.local/bin/` |
| `bin/brasa-greeter` | Installs, removes or previews the login screen | `~/.local/bin/` |
| `bin/brasa-osd` | On-screen indicator for Brasa's own toggles | `~/.local/bin/` |
| `bin/brasa-remote` | Remote desktop over VNC (wayvnc) or RDP (hypr-rdp): own session, virtual monitor, mirror or take-over | `~/.local/bin/` |
| `bin/brasa-autostart` | What starts with the session | `~/.local/bin/` |
| `bin/brasa-test` | Try it inside GNOME | `~/.local/bin/` |
| `config/waybar/` | Bar: modules, style, sensors (`sens`) and real refresh rate (`hz`) | `~/.config/waybar` |
| `config/swaync/` | The old notification panel (swaync), no longer started | `~/.config/swaync` |
| `config/swayosd/` | On-screen indicators: settings and style | `~/.config/swayosd` |
| `config/fuzzel/` | Quick launcher | `~/.config/fuzzel` |
| `config/rofi/drawer.rasi` | App grid: rofi config and theme (any `~/.config/rofi` is left alone) | read from `~/.config/brasa` |
| `config/nwg-bar/` | Exit dialog buttons and style | `~/.config/nwg-bar` |
| `config/sway/` | sway/SwayFX: `config` (Super), `brasa.conf` (shared), `fx.*.conf` (effects), `test.conf` (Alt), `binds.conf` (generated) | `~/.config/sway` |
| `config/hypr/` | Hyprland: `hyprland.lua`, `binds.lua` (generated), `hyprlock.conf` and `remote.lua` (the remote session) | `~/.config/hypr` |
| `config/swaylock/` | swaylock fallback | `~/.config/swaylock` |
| `config/foot/foot.ini` | Terminal: colors, transparency with blur, bell | `~/.config/foot` |
| `greeter/` | Login screen: greetd, nwg-hello and the sway that runs it | `/etc/greetd`, `/etc/nwg-hello` (copied with sudo) |
| `remote/` | The remote session on Hyprland: virtual screen in seat1 (udev), system service, polkit rule; `hypr-rdp/`: the RDP server's upstream commit and Brasa's patch | `/etc/udev/rules.d`, `/etc/systemd/system`, `/usr/share/polkit-1/rules.d` (copied with sudo by `brasa-remote session wm hyprland`) |
| `config/kvantum/Brasa/` | Flat Kvantum theme for Qt apps (made by `docs/gen-kvantum.py`) | `~/.config/Kvantum/Brasa` |
| `config/qt/brasa-colors.conf` | Brasa colors for Qt apps without Kvantum (qt5ct / qt6ct, Fusion style) | read from `~/.config/brasa` |
| `config/gtk-3.0/gtk.css`, `config/gtk-4.0/gtk.css` | Brasa colors for GTK3 and GTK4/libadwaita apps | `~/.config/gtk-3.0/`, `~/.config/gtk-4.0/` |
| `docs/gen-waybar-swaync.py` | Generator for waybar's `config.jsonc` and `compact.jsonc` and swaync's `config.json` | — |

Everything is a symlink, so editing a file in `~/brasa` changes the live configuration.
Generated files (`binds.conf`, `binds.lua`, `fx.conf`) are not edited by hand.

## Installer

```
./install.sh [--wm swayfx|hyprland|both] [--greeter|--no-greeter] [--zsh|--no-zsh] [--no-gtk] [--no-prompt]
             [--root-prompt] [--no-deps] [--dry-run] [--apps] [--yes]
./install.sh --uninstall
```

- `--wm`: which window manager to install (both by default).
- `--greeter` / `--no-greeter`: install greetd + nwg-hello as the login screen instead of the current display manager, or skip it (see [Login screen](#login-screen)). **Without either option the installer asks**, or, when the Brasa login screen is already in use, updates it.
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
- **Apps:** the first time, it asks which app each key opens: terminal, second terminal, browser, file
  manager, second file manager, calculator and whiteboard (see `apps.conf` under [Key bindings](#key-bindings)).
  Each question lists the known ones, marks the installed ones, and Enter keeps the one shown; any other
  command can be typed. The answers go to `~/.config/brasa/apps.conf` and the chosen apps get installed
  (Zen, Chrome and Wayscriber from the AUR). Later runs ask nothing and install what is missing, except
  for a role Brasa added since. `--apps` asks again, with the current choice as Enter.
- `--yes`: asks nothing: no login screen or zsh unless they are already Brasa's, and the default apps
  (the ones already set, else the first installed).
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
cliphist, Thunar and its thumbnails (tumbler, ffmpegthumbnailer, poppler-glib, webp-pixbuf-loader),
polkit agent, xdg portals (gtk, wlr, hyprland), wtype, wdisplays, wlr-randr, starship, wayvnc,
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

**Your own bindings** go in `~/.config/brasa/my-keys.tsv`, same format, not in `keys.tsv`: git ignores
that file, so a pull never touches it. Each of its lines adds a binding, or replaces Brasa's with the same
keys; the action `none` removes Brasa's. The description can be left out (the command is shown instead).

```
SUPER        b   exec firefox            Firefox instead of the browser brasa-app picks
SUPER+ALT    o   exec obs                OBS
SUPER+SHIFT  n   none
```

`Super+Shift+r` applies it. A wrong line is skipped and reported in a notification, with its number.
The key bindings window shows them first, under **Your own**.

**Brasa settings** (`Super+Ctrl+F1`, or the keyboard button of the quick settings panel) edits both files
without opening them, in sway and Hyprland alike:

- **Key bindings**: your own, with add (+), edit (click the row) and remove. To set the keys, press
  **Record** and then the keys: while it listens the window manager's own bindings are paused, so
  `Super+...` reaches the window too. It says whether those keys are free or replace one of Brasa's.
  What they do: **open an app** (picked from a searchable list of the installed ones, with their icons;
  runs `gtk-launch` with its id), **run a file** (picked in a file dialog), **run a command**, or
  **nothing** (turns off Brasa's binding on those keys). Each change is saved and applied at once.
- **Apps**: which app each `brasa-app` key opens, as in `apps.conf`. **Automatic** follows the list
  (the row says which one it opens now); an app not installed says so (`./install.sh` installs it).

After editing: `Super+Shift+r` (regenerates with `brasa-gen` and reloads the window manager).
The key bindings window (`Super+F1`) does not read this file: it asks the running window manager for
every binding it really has (Hyprland: `hyprctl binds` plus the `hl.bind` calls of `~/.config/hypr/*.lua`;
sway: its config), so your own and HyprMod's show up too. Each one says what it does, what it runs and
what kind it is (apps, Brasa, commands, windows, workspaces, sound, brightness, media, mouse). Type to
search; Enter or a click runs it.

| Keys | Action |
|---|---|
| `Super+d` | App launcher island |
| `Super+Shift+d` | App grid, full screen (rofi) |
| `Super+r` | Quick launcher (fuzzel) |
| `Super+Enter`, `Super+t` | Terminal (foot) |
| `Super+Shift+Enter`, `Super+Shift+t` | Second terminal (Ghostty) |
| `Super+Ctrl+Enter` | System monitor (btop, in the terminal) |
| `Super+e` | Files (Thunar) |
| `Super+Shift+n` | Second file manager (Nautilus) |
| `Super+b` | Browser (Zen) |
| `Super+c` | Calculator |
| `Super+x` | Draw on the screen (Wayscriber) |
| `Super+n` | Notification centre (the island) |
| `Super+a` | Quick settings |
| `Super+Tab` | Window switcher, this workspace (hold Super, Tab for the next one) |
| `Super+Shift+Tab` | Window switcher, every workspace |
| `Super+Ctrl+Tab` | Window switcher, every workspace, open until you choose |
| `Super+F1` | Key bindings window |
| `Super+Ctrl+F1` | Brasa settings: your own key bindings, the apps the keys open |
| `Super+Ctrl+s` | Sessions window |

| `Super+Ctrl+c` | Clean: /tmp or RAM |
| `Super+.` | Emoji picker |
| `Super+p` | Displays and monitors |
| `Super+Alt+r` | Screen refresh rate menu |
| `Super+q` | Close the window (asks it to close) |
| `Super+Shift+q` | Kill the focused window's app (`kill -9` to its PID, nothing else) |
| `Super+f` | Fullscreen |
| `Super+Space` | Floating / tiled |
| `Super+w` | Window layout of the focused workspace, the next one; each workspace keeps its own. Hyprland: dwindle (each window splits the focused one) → master (one big window and a column) → scrolling (niri-like: windows on a strip you scroll through, each half the screen) side by side → scrolling one above the other → focus side by side → focus one above the other (like scrolling, but each window nearly fills the screen, kept in the middle so a sliver of the neighbours shows on both sides; the first and the last stay at the edge, the sliver all on their one neighbour), remembered across logins. sway: side by side → one above the other → tabs → stacked |
| `Super+Shift+w` | Window layout of the focused workspace, picked from a list (fuzzel) with the current one selected. Same layouts as `Super+w`, in Hyprland and sway |
| `Super+←↑→↓`, `Super+hjkl` | Move focus |
| `Super+Shift+←↑→↓`, `Super+Shift+hjkl` | Move the window |
| `Super+1…9` | Go to workspace |
| `Super+Shift+1…9` | Move the window to a workspace |
| `Super+Escape` | Lock the screen |
| `Super+Shift+Escape` | Suspend (it locks first) |
| `Ctrl+Super+Escape` | Remote keyboard on/off: every key goes to the focused window (automatic in VNC/RDP viewers) |
| `Super+Shift+e` | Exit dialog: lock, log out, suspend, reboot, power off, cancel |
| `Super+Shift+r` | Reload the window manager (and regenerate bindings) |
| `Super+Ctrl+r` | Reload bar, notifications and menus |
| `Super+Alt+Space` | Switch keyboard layout (es ↔ us altgr-intl) |
| `Super+Shift+b` | Blur on/off |
| `Super+Shift+a` | Animations on/off (Hyprland only) |
| `Super+Shift+f` | All effects on/off |
| `Print`, `Super+Shift+s` / `Shift+Print` / `Ctrl+Print` | Screenshot of an area / of an area on a frozen screen / of the screen |
| `Ctrl+Super+Space` | Play / pause |
| `Ctrl+Super+←` / `→` | Previous / next track |
| `Ctrl+Super+↑` / `↓` | Volume up / down |
| `Super` + drag / + right button | Move / resize (Hyprland; in sway, `Super`+drag) |

**Which app each one opens** (terminal, browser, files...) is `brasa-app`'s choice: the one set in
`~/.config/brasa/apps.conf`, or else the first installed of its list. One line per role, `none` for no app:

```
terminal=kitty
files2=none
```

| Role | Keys | Known apps, in order of preference |
|---|---|---|
| `terminal` | `Super+Enter`, `Super+t` | foot, kitty, ghostty. A command after it runs inside: `brasa-app terminal btop` |
| `terminal2` | `Super+Shift+Enter`, `Super+Shift+t` | another of those three |
| `browser` | `Super+b` | zen-browser, firefox, google-chrome-stable, chromium |
| `files` | `Super+e` | thunar, nautilus, nemo, pcmanfm, dolphin |
| `files2` | `Super+Shift+n` | another of those |
| `calculator` | `Super+c`, Calculator key | qalculate-gtk, gnome-calculator |
| `whiteboard` | `Super+x`, `XF86Launch1` | wayscriber: toggles its daemon (`wayscriber.service`) when it runs, else opens it |

Any other program works too. The change applies on the next key press, with no reload. The installer
writes this file: it asks once for each role, and again with `./install.sh --apps`.

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
| `XF86Launch1` (MyASUS and similar) | Draw on the screen (Wayscriber) |
| Calculator, search, explorer, browser, mail, sleep, power | What they say |

**On the Zenbook S 14 (UX5406)**, checked with a key capture: F1-F3 volume, F4-F5 brightness,
F6 sends `Super+P` (displays), F7 `Super+.` (emoji), F8 microphone, F9 camera (`KEY_CAMERA`),
F12 MyASUS (`KEY_PROG1`). The keyboard light is handled by the firmware and sends no event.

## Screen refresh rate

`brasa-refresh` (`Super+Alt+r`, or click the Hz module in a bar; scrolling on it cycles) works **per
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

**Folding.** Right click on the CPU, temperature, RAM, network, volts or Hz block folds it to its
short form, and another right click unfolds it:

| Block | Folded |
| --- | --- |
| CPU | % (without the GHz) |
| Temperature and fans | the temperature |
| RAM | % (without used, available and swap) |
| Network | ↓ alone |
| Volts, Hz | the icon |

`brasa-bar` starts waybar with one bar per monitor: monitors narrower than 1280 logical pixels (a
monitor turned upright, a small laptop) get the bar already folded, clock short too
(`config/waybar/compact.jsonc`), the others unfolded (`config.jsonc`). `brasa-monitors` runs it again
after every monitor change, so turning a monitor switches its bar. The limit is `NARROW_BELOW` in
`bin/brasa-bar`, or `BRASA_BAR_NARROW_BELOW` in the environment. A fold lasts until the bar restarts.

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
`python3 docs/gen-waybar-swaync.py config`, then `brasa-bar --reload` (or `Super+Ctrl+r`). Editing
`config/waybar/config.jsonc` directly works too, until the generator runs again.

## Notifications

The island is the notification server (`org.freedesktop.Notifications`): swaync is no longer started.
See [Island](#island) for how they look.

- Critical ones: red border, they do not go away by themselves.
- **Sound**: a bell (`bell.oga`) for normal ones, `dialog-warning.oga` for critical ones, nothing
  for low-urgency ones (Brasa notices) or in "do not disturb".
- **Bell** in the bar: click opens the notification centre (like `Super+n`), right click switches
  "do not disturb". The island writes its state to `$XDG_RUNTIME_DIR/brasa-bell.json` and sends the
  bar signal `RTMIN+8`.
- **Closed by their app**: when an app takes a notification back (WhatsApp Web once the chat is read),
  its toast goes but the notification stays, folded at the bottom of the centre under *Closed by their
  app · N* (a click unfolds it). The bell counts only the others.
- **Kept across restarts** in `~/.local/state/brasa/notifications.json`, with Do Not Disturb, and the
  pictures beside it. After a restart their apps no longer know them: their buttons are grey and a click
  opens the app instead. The centre keeps the latest 50; the oldest closed by their app go first.

## Menus and dialogs

- **App grid** (`Super+Shift+d`, `bin/brasa-drawer`): rofi full screen, a 7-column grid with search; type to
  filter, one click or Enter opens, `Escape` or `Super+Shift+d` again closes. Opens in ~50 ms and uses ~14 MB
  only while open (nwg-drawer, used before, took ~1.5 s and ~70 MB). rofi does not handle touch
  input: with a finger, use the keyboard or the touchpad instead.
- **fuzzel** (`Super+r`): type and Enter. Also used by the emoji and refresh menus.
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
screen, the cursor theme you choose ([Icons and cursors](#icons-and-cursors)), 133 % scale on `eDP-1`, dynamic refresh rate.

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

## Island

A black shape that grows out of the screen edge, as if the screen bent inwards: black where it
touches the edge, Brasa blue at the far end, a thin red rim. Every size moves on springs, so it flows
between states, even when interrupted halfway. It shows two things, each one can be switched off:

- **Music**: cover, title, artist, album, progress (drag to seek; hidden when the player gives no
  length), previous / play-pause / next and the output volume. Any MPRIS player counts (browsers,
  Spotify, mpv...); the current one is the last that started playing. A new track or play/pause peeks
  one line for a moment.
- **Notifications**: one that arrives shows up in the island with its
  buttons (Accept / Reject of a Bluetooth pairing...), next to the music line if something plays. It
  leaves on its own after a few seconds; critical ones and ones with buttons wait for an answer. The
  pointer on them keeps them.
- **Open** (`Super+m`, or rest the pointer on the edge, in the middle): the full music view and, below
  it, the notification centre: every notification, newest nearest the music, scrolling, with Do Not
  Disturb, Clear all and a button that closes the island. `x` on one deletes it; a click on it does
  what its app offers (opens the chat, the mail...). `Super+n` and the bar's bell open it with the
  centre even when the centre is off for `Super+m`.

- **App launcher** (`Super+d`): a search over every installed app, as a grid
  of icons or a list, small, medium or large. Typing filters; arrows (Tab in the grid) move, Enter opens,
  Escape closes. It can sit on its own edge (by default the bottom one), with its own monitor and side;
  in the same place as the music and notifications, they share one island and `Super+d` switches it to
  the launcher. The pointer on its own island's edge opens it too.
- **Monitor**: with none chosen, each island shows up on the monitor that has the focus when it opens,
  and its pointer edge works on every monitor: the island comes out on the one the pointer is on.
- **Keys**: each panel's key opens and closes it at once, even halfway through its animation. A panel
  opened with its key (or the bar) takes the keyboard, so Escape closes it, and closes the other one
  you had open; one that opens on its own (a notification, the pointer) leaves the keyboard alone.
  A click anywhere off a panel opened with its key closes it too, on any monitor; that click does not
  reach the window under it. The quick panel (`Super+a`) works the same way.
- **Window switcher** (`Super+Tab`): an island on the left edge, halfway down, with the open windows
  as cards on a turning drum, the most recently used first. The card in front, bigger and ringed in
  red, is the chosen one. Hold Super: each Tab turns to the next window, and letting Super go focuses
  it, on its workspace. `Super+Tab` shows this workspace's windows, `Super+Shift+Tab` every one.
  `Super+Ctrl+Tab` opens it on every window until you choose: arrows, Tab or the wheel turn it, Enter
  or a click on a card focuses that window, Escape or a click off it closes it. Hyprland and sway.
  A window whose title only repeats its app's name (a terminal's "foot") shows what runs in it: its
  folder, or the program in front with its folder (`nvim  ·  ~/brasa`).
  Each card fills with a small picture of its window, taken as the switcher opens, against the
  card's right side and melting into it; the icon then shrinks to the corner. The one in front is
  taken again twice a second while open, so a video or a terminal moves in it. A window that cannot
  be pictured keeps its big icon. On Hyprland, a monitor that is off pictures nothing.

**Settings**: the **Island** window (`Super+Shift+m`) switches the music, the
notification centre of `Super+m` and the launcher on or off, and picks for each island its monitor, edge (top or
bottom) and side, plus the launcher's look, opening with the pointer (a switch for each island), peek
time and colour. Each change is saved and the island shows up in its new place at once.

They live in `~/.config/brasa/island.conf`; edited by hand, `Super+Ctrl+r` applies them:

```
media=on            # the music
notification_centre=on  # off: Super+m shows no list (Super+n still does)
edge=top            # top or bottom
align=center        # left, center or right
monitor=DP-2        # empty: the focused one
hover=on            # open it with the pointer; off: only Super+m
peek_seconds=2.5    # 0: never
color=#0c1838       # the far end of the gradient
launcher=on
launcher_edge=bottom    # the launcher's own place: same keys with launcher_ in front
launcher_align=center
launcher_monitor=
launcher_view=grid      # or list
launcher_size=medium    # small, medium or large
launcher_hover=on       # its own pointer switch; missing: same as hover
quick_monitor=          # the quick settings' monitor; empty: the focused one
quick_hover=off         # on: resting the pointer in its bottom corner opens it
```

### Built in Rust

The islands and quick settings are one Rust program, `islands/` (`brasa-islands`): one process, about
50 MB, no CPU while idle, the outlines drawn by the GPU. `install.sh` builds it (it needs `cargo`), and
`brasa-reload` rebuilds it after a `git pull`. By hand: `cd ~/brasa/islands && cargo build --release`.
`brasa-islands toggle|centre|dnd|launcher|peek|hide|reload|quick|switch|switch-all|windows` tells the running one what to do.

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
- GNOME still works from greetd, but GNOME's "switch user" needs GDM. GNOME only starts on a session
  that logind marks as `wayland`, and greetd opens every session as `tty` unless the greeter says
  otherwise: `XDG_SESSION_TYPE=wayland` in the `env-vars` of `greeter/nwg-hello.json` does it
  (Hyprland and sway did not need it, they change the type themselves).

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

## Remote desktop (VNC and RDP)

`brasa-remote` shares the desktop with **wayvnc**, which captures straight from the compositor: no
"which screen do you want to share?" prompt, and remote keyboard and mouse work. Everything is off
by default. Two independent servers, which can run at the same time:

| Port | What | Modes |
|---|---|---|
| 5900 | your remote desktop, a session of its own | `session` |
| 5901 | the local desktop, whoever is logged in on the machine | `screen`, `mirror`, `take` (one at a time) |

All of it can be controlled over SSH: `session` does not need a local desktop at all, and the local
modes find the running Hyprland or sway by themselves (also when run from inside the remote session).

**Where it listens**: by default only on the VPN's address. The **Network** group of the Sessions
window (or `brasa-remote listen`, `brasa-remote port`) picks every network or some of this machine's
addresses, and the two ports; a running session follows at once (its viewers reconnect), the shared
desktop from its next start.

**VNC or RDP for the remote session**: one of them at a time, chosen in the Sessions window (Protocol) or
with `brasa-remote protocol vnc|rdp`. RDP is served by **hypr-rdp**, built from source with Brasa's patch
(`remote/hypr-rdp/`: the upstream commit and `sway.patch`, which adds sway and `--resize-output`). It
works with Windows Remote Desktop and any RDP client, sends the sound, syncs the clipboard (text and
images) and uses the viewer's keyboard layout; the screen follows the viewer's window, as with VNC.

- The first time RDP is chosen a terminal opens to build it (a few minutes, `~/.cache/brasa/hypr-rdp`,
  installed as `~/.local/bin/hypr-rdp`); `install.sh` and `brasa-reload` keep it up to date afterwards.
- **Sign-in**: your Linux password (PAM, `/etc/pam.d/hypr-rdp`, installed with sudo), for clients that
  send it over TLS without NLA (Windows asks to save it first), or a password of its own
  (`~/.config/brasa/rdp-password`, mode 600), which every client accepts.
- **Sound**: on the device you connect from (the default; it comes back when you disconnect), on both,
  or off. The remote session shares PipeWire with this machine's own session.
- RDP listens on one address, the first one of "Listen on", on port 3389. Until RDP can sign in, the
  session keeps serving VNC.
- Sharing this machine's own desktop (screen, mirror, take) stays on VNC.

**Firewall**: with **ufw** on, exactly the addresses and ports the VNC and RDP servers and the SSH server listen
on are opened, to any source, while they are on, and closed again when they stop. A rule of yours on that
port is left alone. It is done by `/usr/local/sbin/brasa-firewall`, which `install.sh` (or the first
`brasa-remote session` in a terminal) installs with sudo, together with a sudo rule to run it without a
password and a hook on the SSH service. Its rules carry the comment `brasa` in `sudo ufw status`.

```
brasa-remote session [stop|boot|noboot]  an independent remote-only session (the default for "on")
brasa-remote session wm sway|hyprland    its window manager (sway, or Hyprland if it is the only one)
brasa-remote session size auto|WxH       auto: 1920x1080, then the viewer's size; WxH: always that size
brasa-remote session scale auto|N        auto: from the height; N: fixed, e.g. 1.25
brasa-remote session defaults            size and scale back to auto
brasa-remote session settings            show them
brasa-remote screen [WxH]                a virtual second monitor in the current session
brasa-remote mirror                      share the physical screen (the focused monitor)
brasa-remote take [WxH]                  take over the current session; the laptop goes dark
brasa-remote ssh [on|off|boot|noboot]   the SSH server, now and at boot
brasa-remote listen [vpn|any|IP...]      where to listen: the VPN (default), every network, or addresses
brasa-remote port session|local|rdp N    the session's VNC port (5900), the local desktop's (5901), RDP's (3389)
brasa-remote protocol [vnc|rdp]          how the remote session is reached
brasa-remote rdp login linux|password    RDP sign-in: the Linux password, or one of its own
brasa-remote rdp password                set that one (read from the input)
brasa-remote rdp sound remote|both|off   where the session sounds while connected over RDP
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
- **session on Hyprland** (`session wm hyprland`, saved as `REMOTE_WM=hyprland`; the default when
  Hyprland is installed and sway is not): Hyprland cannot
  start without a screen, so it gets a virtual one, the kernel's `vkms`, in a seat of its own
  (`seat1`). Each seat has its own active session, so the local desktop and the remote one run at the
  same time and the local one never sees the virtual screen. The system service
  `brasa-remote-hyprland@<user>` loads `vkms`, opens a logind session in `seat1` for you (no second
  password: the VNC login is the only one) and starts Hyprland with `config/hypr/remote.lua`, which is
  `hyprland.lua` plus: the virtual screen off, and a headless output `HEADLESS-REMOTE` that `brasa-remote`
  creates and serves, so the size follows the viewer as on sway. The first time asks for sudo to
  install `remote/`, and `vkms` if the kernel lacks it (`install.sh` already does that with
  `--wm hyprland|both`: Arch and Debian have it built in, Ubuntu's older kernels in
  `linux-modules-extra-<kernel>`, Fedora in `kernel-modules-extra`, openSUSE in `kernel-default-extra`); after that a polkit rule lets you start and stop your own session without a
  password, also over SSH. `boot`/`noboot` enable the system service (sudo). Switching `wm` moves a
  running session and its start at boot to the other one. Logs: `journalctl -u brasa-remote-hyprland@$USER`.
  On a machine with GDM, GDM also tries a login screen on `seat1` while the session runs, fails and
  gives up: harmless. `brasa-monitors` does nothing in either remote session, so your saved monitors
  stay the local ones.
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
  `REMOTE_DEFAULT=` (mode for `on`), `REMOTE_WM=` (sway or hyprland), `REMOTE_ADDRESS=`, `REMOTE_PORT=` (session, 5900),
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

## Quick settings

`brasa-islands quick` (`Super+a`, or the gear next to the bell in the bar): it grows out of the bottom right
corner of its monitor (**Monitor** in the Island window, `quick_monitor`; none: the focused one), joined to the bottom edge and to the side edge like the island, with the
same springs, colours and red rim. The bottom left corner instead: **Corner** in the Island window,
`quick_side` in `island.conf`. Escape or a click anywhere else closes it. With **Open with the pointer**
(`quick_hover`), resting the pointer in that corner of a monitor opens it there, and it closes when the
pointer leaves it.

- **Tiles**: square and black with a red line along the bottom, icon and name inside in red. **Size** (Island window, `quick_size`): small, six a
  page; medium, twelve a page; large, every tile at once. One turn of the **mouse wheel**, or one swipe
  on the touchpad, moves exactly one page; the dots on the side show which page, a click on one jumps there.
- A **switch** tile turns red while on; a click turns it on or off.
- A **menu** tile has a **›** after its name: it opens its choices in the card, the current one lit; the arrow goes back.
- Icons are Nerd Font glyphs, so they look the same with any icon theme.
- While open it follows changes made elsewhere: the sliders when you use the volume or brightness keys,
  the switches (Wi-Fi, mute…) within two seconds.
- Below: volume and, with a backlight, brightness; then the battery, the settings button (the Island
  window: corner, size), Brasa settings (`brasa-settings`: your key bindings, the apps the keys open, the icon and cursor themes),
  HyprMod (Hyprland settings, when installed, only in Hyprland) and the power button (`brasa-exit`).

**Keyboard only**:

| Key | Does |
|---|---|
| Arrows | move between tiles; down from the last row turns the page, then goes on to the sliders and the bottom buttons |
| Page Down / Page Up | the next / previous page |
| Enter, Space | press the tile or button |
| Left / Right on a slider | change it |
| Escape | back from a menu (Left and Backspace too); on the tiles, close |

| Tile | Click |
|---|---|
| Wi-Fi · Bluetooth · Airplane · Mute | on / off (Bluetooth only with Bluetooth) |
| Effects | blur, animations and shadows, like `Super+Shift+f` |
| Networks | Wi-Fi and network settings (nm-connection-editor) |
| Keyboard | keyboard layouts: pick one, add or remove (`brasa-keyboard`) |
| Sessions | the Sessions window (below), like `Super+Ctrl+s` |
| Island | the Island window, like `Super+Shift+m` |
| Keys | the key bindings window, like `Super+F1` |
| Lock | lock the screen |

| Menu | Choices |
|---|---|
| Layout | the focused workspace's window layout (like `Super+w`). Hyprland: dwindle, master, scrolling and focus, each side by side (→) or one above the other (↓). sway: side by side, above/below, tabs, stacked |
| Power | power profile: power saver, balanced, performance (`brasa-power`, with power-profiles-daemon) |
| Charge limit | 60 / 80 / 90 / 100 % (laptops that support it, see below) |
| Key light | keyboard backlight level, the levels the driver offers (`brasa-kbd set N`; only with one) |
| Clean | Clean /tmp…: a window lists the leftovers nobody uses, with their size, and removes them when you confirm (asks for the password: it also looks at root's). Free RAM: drops the page cache and tells how much was freed |

Your own tiles from `~/.config/brasa/panel.conf` come last. Tiles a machine cannot use are left out.
The card is built again at every opening, so a change to `panel.conf` shows at once.

### Sessions window

`Super+Ctrl+s` or the Sessions tile. Changes apply at once; it refreshes every 5 seconds.

| Section | What |
|---|---|
| This session | **Dialogs and passwords**: where they open now; *Bring here* sends them to this session. **Keyring**: locked or not; Lock / Unlock |
| Remote session | on, start at boot, window manager (when both are installed), size, scale; who is connected (`brasa-remote session …`) |
| Share this desktop | off, mirror, screen or take (port 5901) |
| SSH server | on, start at boot (asks for the password through polkit) |

**Bring here** (`brasa-follow`): password prompts, the keyring's dialog, file choosers and screen
sharing come from programs the user's D-Bus or systemd starts, and they open on the screen of the
session that last told the bus where it is. *Bring here* tells the bus (and systemd) this session's
screen, and starts again the user services that have a window on another session's screen (found by
their connection to that screen's socket, whatever program they are), so nothing stays behind. It
only uses standards (D-Bus activation environment, the Secret Service API), so it works with any
keyring: GNOME Keyring, KWallet, KeePassXC… Unlock asks in this session.

```
brasa-follow [here]               dialogs open here from now on
brasa-follow where                where they open now
brasa-follow keyring status|lock|unlock
```

### Your own tiles

`~/.config/brasa/panel.conf` (yours, not in git), one tile per line. The icon is a Nerd Font glyph or a
symbolic icon name (`…-symbolic`):

```
# icon | name | command                                   a button
# icon | name | on command | off command | state test     a switch, lit while the test succeeds
󰌆 | VPN       | nmcli connection up work
󰢮 | GPU awake | ~/.local/bin/gpu-awake on | ~/.local/bin/gpu-awake off | systemctl -q is-active gpu-forcewake
```

The next opening of the panel shows them. Each command runs through `brasa-run`, which shows
"Working…" at once and then the command's last line of output (or the error). For
actions that need root, the script can use `pkexec`, which asks for the password with a dialog (the
polkit agent Brasa starts), e.g. `pkexec sh -c 'sync; echo 3 > /proc/sys/vm/drop_caches'`.

## Keyboard layouts

```
brasa-keyboard                 a menu: your layouts (pick one to type with), Add…, Remove…
brasa-keyboard list            your layouts, the active one marked
brasa-keyboard add fr [oss]    add a layout (and variant); remove fr [oss]; use N
```

Also from the panel (Keyboard tile) and a right click on the bar's language indicator (a left click
switches to the next layout, like `Super+Alt+Space`). "Add" searches every layout and variant the
system knows (about 600). Your layouts live in `~/.config/brasa/keyboard.conf` (not in git); Hyprland
loads them when it starts (`~/.local/state/brasa/keyboard.lua`), on sway `brasa-autostart` applies
them. Without that file, the layouts in `hyprland.lua` / `sway/brasa.conf` are used.

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

## Icons and cursors

The installer downloads these themes from gnome-look.org into `~/.local/share/icons` (list in
`data/themes.tsv`):

| Kind | Themes |
|---|---|
| icons | **KanadaIcons** (default), Baphomet, LED |
| cursors | **KanadaCursors** (default), Oxygen 05 Vibrant Red, Fuchsia-Red, Breeze Dark Red, GoogleDot-Red |

Brasa settings (`Super+Ctrl+F1`), **Appearance** page, lists every theme installed on the machine, with
a preview, and applies the one you pick at once. The choice is saved in `~/.config/brasa/appearance.conf`
and goes to GTK (gsettings and `settings.ini`), Qt (qt5ct / qt6ct), the window manager's cursor and
`~/.icons/default` (X11 apps). Hyprland reads the cursor at start from `~/.local/state/brasa/cursor.lua`;
sway gets it from `brasa-autostart`. Some apps change their icons only when they start again.

```
brasa-themes download                  the themes of data/themes.tsv not installed yet
brasa-themes list icons|cursors        the installed ones, the chosen one marked
brasa-themes icons NAME | cursor NAME  choose one (its folder name) and use it now
brasa-themes apply                     use the chosen ones again
```

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
- `Super+/` is not used for the key bindings window because `/` needs Shift on a Spanish keyboard.
- On the Zenbook, the camera key's light is driven by the ASUS firmware; Linux exposes no control for it.
