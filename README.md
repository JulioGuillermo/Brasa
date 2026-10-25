# Brasa

A light desktop for **SwayFX** or **Hyprland**: near-black night blue, pure red details and a
bar with triangle separators.

## Install

```bash
cd ~/brasa
./install.sh
```

It installs whatever is missing with your package manager (pacman, apt, dnf, zypper or apk),
links each config file into place and backs up anything that was there before.
Then log out and pick **SwayFX** or **Hyprland** on the login screen.

Only one of them? `./install.sh --wm swayfx` or `./install.sh --wm hyprland`.
It also asks whether to use the Brasa login screen instead of GDM (preview it first with `brasa-greeter test`).
And whether to set up **zsh** (your old config goes to `~/zsh_backup/`; your own paths, aliases and tools
live in `~/.config/zsh/`). The prompt is **starship** with the same look as the bar.
Try it without leaving GNOME? `brasa-test`.
Undo it? `./install.sh --uninstall`.

## Main key bindings

| Keys | What it does |
|---|---|
| `Super+F1` | **All key bindings**, searchable |
| `Super+d` / `Super+r` | App grid / quick launcher |
| `Super+Enter` or `Super+t` | Terminal (foot); with `Shift`, Ghostty |
| `Super+e` / `Super+Shift+n` | Files: Thunar / Nautilus |
| `Super+z` | Zen |
| `Super+q` / `Super+Shift+q` | Close / kill the window |
| `Super+n` | Notifications |
| `Super+1…9` | Workspaces (with `Shift`, move the window) |
| `Ctrl+Super+Space` `←` `→` `↑` `↓` | Music and volume |
| `Super+.` | Emoji |
| `Super+Alt+r` | Screen refresh rate: dynamic or fixed |
| `Super+Escape` | Lock the screen |
| `Super+Shift+e` | Log out, lock, power off… |

Special keys (volume, brightness, keyboard light, microphone, camera, emoji, displays, airplane…)
work on any laptop that has them. Key bindings live in `keys.tsv`; after editing, press `Super+Shift+r`.

Everything else (what each file does, colors, the bar, how to add things) is in
[docs/brasa.md](docs/brasa.md).
