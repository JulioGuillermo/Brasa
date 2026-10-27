#!/usr/bin/env bash
# Brasa installer: dependencies (for your package manager) and config links.
# Usage: ./install.sh [--wm swayfx|hyprland|both] [--greeter|--no-greeter] [--zsh|--no-zsh] [--no-gtk] [--no-prompt]
#                     [--root-prompt] [--no-deps] [--dry-run]
#        --root-prompt: the same starship prompt for root too (asks for sudo).
#        Without --greeter/--no-greeter or --zsh/--no-zsh it asks about the login screen and zsh.
#      ./install.sh --uninstall
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
WM=both GTK=1 PROMPT=1 ROOT_PROMPT=0 GREETER=ask ZSH=ask DEPS=1 DRY=0 UNINSTALL=0
STAMP=$(date +%Y%m%d-%H%M%S)

while [ $# -gt 0 ]; do
  case $1 in
    --wm) WM=$2; shift ;;
    --gtk) GTK=1 ;;
    --no-gtk) GTK=0 ;;
    --no-prompt) PROMPT=0 ;;
    --zsh) ZSH=1 ;;
    --no-zsh) ZSH=0 ;;
    --greeter) GREETER=1 ;;
    --no-greeter) GREETER=0 ;;
    --no-deps) DEPS=0 ;;
    --dry-run) DRY=1 ;;
    --uninstall) UNINSTALL=1 ;;
    --root-prompt) ROOT_PROMPT=1 ;;
    -h|--help) sed -n 2,7p "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
  shift
done
case $WM in swayfx|hyprland|both) ;; *) echo "--wm must be swayfx, hyprland or both" >&2; exit 1 ;; esac

r=$'\e[1;31m' b=$'\e[1;34m' g=$'\e[0;37m' n=$'\e[0m'
say()  { printf '%s▶%s %s\n' "$r" "$n" "$*"; }
info() { printf '  %s%s%s\n' "$g" "$*" "$n"; }
warn() { printf '%s!%s %s\n' "$r" "$n" "$*"; }
run()  { if [ $DRY = 1 ]; then info "[dry run] $*"; else "$@"; fi; }
PROBLEMS=()                               # everything that went wrong, summarized at the end
problem() { PROBLEMS+=("$*"); warn "$*"; }

# ---------------------------------------------------------------- links
# Config Brasa owns entirely (directories) and single files inside other apps' directories
DIRS="waybar swaync swayosd fuzzel nwg-bar sway hypr swaylock foot"
FILES=""                                   # none at the moment
# Links from older Brasa versions, removed on install (nwg-drawer was replaced by rofi)
OBSOLETE="nwg-drawer/brasa.css"
BINS=$(cd "$HERE/bin" && ls)

link() {  # source destination: back up whatever is there (unless it is our link) and link
  local src=$1 dst=$2
  if [ -L "$dst" ] && [ "$(readlink -f "$dst")" = "$(readlink -f "$src")" ]; then return; fi
  if [ -e "$dst" ] || [ -L "$dst" ]; then
    run mv "$dst" "$dst.bak-brasa-$STAMP"; info "backup: $dst.bak-brasa-$STAMP"
  fi
  run mkdir -p "$(dirname "$dst")"
  run ln -s "$src" "$dst"
}

unlink_ours() {  # remove the link if it points into Brasa and restore the latest backup
  local dst=$1
  [ -L "$dst" ] && [[ "$(readlink -f "$dst")" == "$HERE"* ]] || return 0
  run rm "$dst"; info "removed $dst"
  local bak; bak=$(ls -d "$dst".bak-brasa-* 2>/dev/null | sort | tail -1)
  [ -n "$bak" ] && { run mv "$bak" "$dst"; info "restored $dst"; }
}

# ---------------------------------------------------------------- shell prompt (starship)
MARK_BEGIN="# >>> brasa: starship prompt >>>"
MARK_END="# <<< brasa: starship prompt <<<"
shell_rc() {  # rc file and init line of the user's login shell
  case $(basename "${SHELL:-}") in
    zsh)  echo "$HOME/.zshrc|eval \"\$(starship init zsh)\"" ;;
    bash) echo "$HOME/.bashrc|eval \"\$(starship init bash)\"" ;;
    fish) echo "$HOME/.config/fish/config.fish|starship init fish | source" ;;
  esac
}
prompt_hook() {
  local rc line; IFS='|' read -r rc line <<<"$(shell_rc)"
  [ -n "$rc" ] || { problem "Unknown login shell ${SHELL:-?}: add starship to it by hand (https://starship.rs)"; return; }
  if [ -f "$rc" ] && grep -q "starship init" "$rc"; then info "$(basename "$rc") already starts starship"; return; fi
  [ -f "$rc" ] && { run cp -a "$rc" "$rc.bak-brasa-$STAMP"; info "backup: $rc.bak-brasa-$STAMP"; }
  run mkdir -p "$(dirname "$rc")"
  if [ $DRY = 1 ]; then info "[dry run] add starship to $rc"; return; fi
  printf '\n%s\n%s\n%s\n' "$MARK_BEGIN" "$line" "$MARK_END" >> "$rc"; info "starship added to $rc"
}
ZSH_MARK="# Brasa · zsh."
ZSH_BACKUP=~/zsh_backup/$STAMP
install_zsh() {
  local f moved=0
  # Every previous zsh config file goes to ~/zsh_backup/<date>/ (unless it is already Brasa's .zshrc)
  if ! head -1 ~/.zshrc 2>/dev/null | grep -qF "$ZSH_MARK"; then
    for f in ~/.zshrc ~/.zshrc_* ~/.zsh_*.zsh ~/.p10k.zsh ~/.zcompdump*; do
      [ -e "$f" ] || continue
      run mkdir -p "$ZSH_BACKUP"; run mv "$f" "$ZSH_BACKUP/"; moved=1
    done
    [ $moved = 1 ] && info "previous zsh config moved to $ZSH_BACKUP/"
  fi
  run install -m 644 "$HERE/config/zsh/zshrc" ~/.zshrc
  # Your own files: created from the commented templates only if they do not exist yet
  run mkdir -p ~/.config/zsh
  for f in path aliases tools; do
    [ -e ~/.config/zsh/$f.zsh ] || { run cp "$HERE/config/zsh/$f.zsh" ~/.config/zsh/$f.zsh; info "created ~/.config/zsh/$f.zsh"; }
  done
  # History lives with the rest now (a copy of the old one stays in the backup)
  if [ -f ~/.zsh_history ] && [ ! -e ~/.config/zsh/history ]; then
    run mkdir -p "$ZSH_BACKUP"; run cp -a ~/.zsh_history "$ZSH_BACKUP/"
    run mv ~/.zsh_history ~/.config/zsh/history; run chmod 600 ~/.config/zsh/history
    info "history moved to ~/.config/zsh/history"
  fi
  # zap, the plugin manager (plugins are fetched on the first zsh start)
  if [ ! -f "${XDG_DATA_HOME:-$HOME/.local/share}/zap/zap.zsh" ]; then
    run git clone -q --depth 1 -b release-v1 https://github.com/zap-zsh/zap "${XDG_DATA_HOME:-$HOME/.local/share}/zap" \
      || problem "Could not install zap (git clone): zsh works, but without plugins"
  fi
  # zsh as the login shell
  if [ "$(basename "${SHELL:-}")" != zsh ] && command -v zsh >/dev/null; then
    info "Making zsh your login shell (asks for your password)"
    run chsh -s "$(command -v zsh)" || problem "chsh failed: run  chsh -s $(command -v zsh)  by hand"
  fi
}
uninstall_zsh() {  # put the previous zsh config back from the latest ~/zsh_backup/<date>/
  head -1 ~/.zshrc 2>/dev/null | grep -qF "$ZSH_MARK" || return 0
  local bak; bak=$(ls -d ~/zsh_backup/*/ 2>/dev/null | sort | tail -1)
  [ -n "$bak" ] || return 0
  run rm ~/.zshrc
  [ -f ~/.config/zsh/history ] && [ ! -e ~/.zsh_history ] && run mv ~/.config/zsh/history ~/.zsh_history
  for f in "$bak".??* "$bak"*; do [ -e "$f" ] && [ "$(basename "$f")" != .zsh_history ] && run cp -a "$f" ~/; done
  info "restored the zsh config from $bak (your ~/.config/zsh files and login shell stay)"
}

check_local_bin() {  # Brasa's commands live in ~/.local/bin: is it in the login shell's PATH?
  [ "$ZSH" = 1 ] && return 0   # Brasa's .zshrc adds it
  local sh; sh=$(basename "${SHELL:-}")
  if ! "${SHELL:-sh}" -lic 'echo "$PATH"' 2>/dev/null | tr ':' '\n' | grep -qx "$HOME/.local/bin"; then
    problem "~/.local/bin is not in your $sh PATH: Brasa works, but to type brasa-* commands add  export PATH=\"\$HOME/.local/bin:\$PATH\"  to your shell config"
  fi
}

prompt_unhook() {  # remove only the block Brasa added
  local rc; IFS='|' read -r rc _ <<<"$(shell_rc)"
  [ -n "$rc" ] && [ -f "$rc" ] && grep -qF "$MARK_BEGIN" "$rc" || return 0
  run sed -i "/$MARK_BEGIN/,/$MARK_END/d" "$rc"; info "starship block removed from $rc"
}

# root: a root-owned copy of the config (not a link into your home: starship runs the commands in it, and
# root must not run a file your user can edit), plus the init block in root's bash/zsh rc files
ROOT_STARSHIP=/root/.config/starship.toml
root_prompt() {
  command -v starship >/dev/null || { problem "starship is not installed: root prompt skipped"; return; }
  if [ $DRY = 1 ]; then info "[dry run] copy starship.toml to $ROOT_STARSHIP and start it in root's shell"; return; fi
  sudo install -D -m 644 -o root -g root "$HERE/config/starship/starship.toml" "$ROOT_STARSHIP" ||
    { problem "Could not set up root's prompt (sudo)"; return; }
  local sh rc
  for sh in bash zsh; do
    rc=/root/.${sh}rc
    [ $sh = zsh ] && ! sudo test -f $rc && continue        # zsh only if root already has a .zshrc
    sudo grep -qF "$MARK_BEGIN" $rc 2>/dev/null && continue
    sudo test -f $rc && sudo cp -a $rc $rc.bak-brasa-$STAMP
    printf '\n%s\n%s\n%s\n' "$MARK_BEGIN" "command -v starship >/dev/null && eval \"\$(starship init $sh)\"" "$MARK_END" |
      sudo tee -a $rc >/dev/null
  done
  info "root's prompt: $ROOT_STARSHIP (a copy: run with --root-prompt again after changing the prompt)"
}
root_unprompt() {
  sudo test -f $ROOT_STARSHIP 2>/dev/null || return 0
  run sudo rm -f $ROOT_STARSHIP
  for rc in /root/.bashrc /root/.zshrc; do
    sudo grep -qF "$MARK_BEGIN" $rc 2>/dev/null && run sudo sed -i "/$MARK_BEGIN/,/$MARK_END/d" $rc
  done
  info "root's prompt removed"
}

if [ $UNINSTALL = 1 ]; then
  say "Removing Brasa links (packages are not uninstalled)"
  for d in $DIRS; do unlink_ours ~/.config/$d; done
  for f in $FILES; do unlink_ours ~/.config/$f; done
  for f in $BINS; do unlink_ours ~/.local/bin/$f; done
  unlink_ours ~/.config/brasa
  unlink_ours ~/.config/gtk-3.0/gtk.css
  unlink_ours ~/.config/gtk-4.0/gtk.css
  unlink_ours ~/.config/starship.toml
  unlink_ours ~/.config/Kvantum/Brasa
  prompt_unhook
  root_unprompt
  uninstall_zsh
  unlink_ours ~/.local/share/backgrounds/brasa.png
  readlink ~/.local/share/backgrounds/brasa.png 2>/dev/null | grep -q '/brasa-custom\.' &&
    run rm ~/.local/share/backgrounds/brasa.png ~/.local/share/backgrounds/brasa-custom.*
  run rm -f ~/.local/share/backgrounds/brasa-lock.png ~/.local/share/backgrounds/brasa-lock-custom.* \
    ~/.local/share/backgrounds/brasa-login.png ~/.local/share/backgrounds/brasa-login-custom.*
  [ -f /etc/greetd/.brasa-previous-dm ] && run "$HERE/bin/brasa-greeter" uninstall
  say "Done"; exit 0
fi

# ---------------------------------------------------------------- dependencies
# logical name    pacman                  apt                         dnf                       zypper                    apk
# "a|b": try a, then b.  "-": not available on that distro.  "@aur": from the AUR (alternatives tried in
# order until one installs: swayfx sometimes fails to build against the current scenefx, swayfx-git works).
PKGS='
waybar            waybar                  waybar                      waybar                    waybar                    waybar
swaync            swaync                  sway-notification-center    SwayNotificationCenter    SwayNotificationCenter    swaync
fuzzel            fuzzel                  fuzzel                      fuzzel                    fuzzel                    fuzzel
rofi              rofi                    rofi                        rofi                      rofi                      rofi
nwg-bar           nwg-bar                 nwg-bar                     nwg-bar                   nwg-bar                   nwg-bar
swaybg            swaybg                  swaybg                      swaybg                    swaybg                    swaybg
hyprlock          hyprlock                hyprlock                    hyprlock                  hyprlock                  hyprlock
wlr-randr         wlr-randr               wlr-randr                   wlr-randr                 wlr-randr                 wlr-randr
xrdb              xorg-xrdb               x11-xserver-utils           xrdb                      xrdb                      xrdb
swayosd           swayosd                 swayosd                     swayosd                   swayosd                   swayosd
wayvnc            wayvnc                  wayvnc                      wayvnc                    wayvnc                    wayvnc
openssl           openssl                 openssl                     openssl                   openssl                   openssl
starship          starship                starship                    starship                  starship                  starship
swaylock          swaylock                swaylock                    swaylock                  swaylock                  swaylock
swayidle          swayidle                swayidle                    swayidle                  swayidle                  swayidle
nm-applet         network-manager-applet  network-manager-gnome       network-manager-applet    NetworkManager-applet     network-manager-applet
blueman           blueman                 blueman                     blueman                   blueman                   blueman
bluetoothctl      bluez-utils             bluez                       bluez                     bluez                     bluez
brightnessctl     brightnessctl           brightnessctl               brightnessctl             brightnessctl             brightnessctl
playerctl         playerctl               playerctl                   playerctl                 playerctl                 playerctl
wireplumber       wireplumber             wireplumber                 wireplumber               wireplumber               wireplumber
paplay            libpulse                pulseaudio-utils            pulseaudio-utils          pulseaudio-utils          pulseaudio-utils
pavucontrol       pavucontrol             pavucontrol                 pavucontrol               pavucontrol               pavucontrol
jq                jq                      jq                          jq                        jq                        jq
python3           python                  python3                     python3                   python3                   python3
notify-send       libnotify               libnotify-bin               libnotify                 libnotify-tools           libnotify
gtk-launch        gtk3                    libgtk-3-bin                gtk3                      gtk3-tools                gtk+3.0
xdg-user-dirs     xdg-user-dirs           xdg-user-dirs               xdg-user-dirs             xdg-user-dirs             xdg-user-dirs
sounds            sound-theme-freedesktop sound-theme-freedesktop     sound-theme-freedesktop   sound-theme-freedesktop   sound-theme-freedesktop
grim              grim                    grim                        grim                      grim                      grim
slurp             slurp                   slurp                       slurp                     slurp                     slurp
wl-clipboard      wl-clipboard            wl-clipboard                wl-clipboard              wl-clipboard              wl-clipboard
cliphist          cliphist                cliphist                    cliphist                  cliphist                  cliphist
thunar            thunar                  thunar                      Thunar                    thunar                    thunar
tumbler           tumbler                 tumbler                     tumbler                   tumbler                   tumbler
thumbs-video      ffmpegthumbnailer       ffmpegthumbnailer           ffmpegthumbnailer         ffmpegthumbnailer         ffmpegthumbnailer
thumbs-extra      poppler-glib            tumbler-plugins-extra       poppler-glib              libpoppler-glib8          poppler-glib
thumbs-webp       webp-pixbuf-loader      webp-pixbuf-loader          webp-pixbuf-loader        webp-pixbuf-loader        webp-pixbuf-loader
polkit-agent      polkit-gnome            mate-polkit|policykit-1-gnome polkit-gnome              polkit-gnome              polkit-gnome
qt5ct             qt5ct                   qt5ct                       qt5ct                     qt5ct                     -
kvantum           kvantum                 qt-style-kvantum            kvantum                   kvantum-qt6               kvantum
kvantum-qt5       kvantum-qt5             -                           kvantum-qt5               kvantum-qt5               -
qt6ct             qt6ct                   qt6ct                       qt6ct                     qt6ct                     -
gtk-theme         adw-gtk-theme           adw-gtk3                    adw-gtk3-theme            adw-gtk3                  -
portal-gtk        xdg-desktop-portal-gtk  xdg-desktop-portal-gtk      xdg-desktop-portal-gtk    xdg-desktop-portal-gtk    xdg-desktop-portal-gtk
wtype             wtype                   wtype                       wtype                     wtype                     wtype
wdisplays         wdisplays               wdisplays                   wdisplays                 wdisplays                 wdisplays
btop              btop                    btop                        btop                      btop                      btop
fuser             psmisc                  psmisc                      psmisc                    psmisc                    psmisc
foot              foot                    foot                        foot                      foot                      foot
font              ttf-hack-nerd           -                           -                         -                         font-hack-nerd
'
PKGS_SWAY='
sway              swayfx|swayfx-git@aur              sway                        swayfx|sway               swayfx|sway               swayfx|sway
portal-wlr        xdg-desktop-portal-wlr  xdg-desktop-portal-wlr      xdg-desktop-portal-wlr    xdg-desktop-portal-wlr    xdg-desktop-portal-wlr
'
PKGS_HYPR='
hyprland          hyprland                hyprland                    hyprland                  hyprland                  hyprland
portal-hypr       xdg-desktop-portal-hyprland xdg-desktop-portal-hyprland xdg-desktop-portal-hyprland xdg-desktop-portal-hyprland xdg-desktop-portal-hyprland
hyprmod           hyprmod@aur             -                           -                         -                         -
'
PKGS_ZSH='
zsh               zsh                     zsh                         zsh                       zsh                       zsh
fzf               fzf                     fzf                         fzf                       fzf                       fzf
git               git                     git                         git                       git                       git
'
PKGS_GREETER='
greetd            greetd                  greetd                      greetd                    greetd                    greetd
nwg-hello         nwg-hello               nwg-hello                   nwg-hello                 nwg-hello                 nwg-hello
imagemagick       imagemagick             imagemagick                 ImageMagick               ImageMagick               imagemagick
'

detect_pm() {
  for pm in pacman apt-get dnf zypper apk; do command -v $pm >/dev/null && { echo $pm; return; }; done
  echo none
}
PM=$(detect_pm)
AUR=""; for h in paru yay; do command -v $h >/dev/null && { AUR=$h; break; }; done
col() { case $PM in pacman) echo 2 ;; apt-get) echo 3 ;; dnf) echo 4 ;; zypper) echo 5 ;; apk) echo 6 ;; esac; }
SUDO=sudo; command -v sudo >/dev/null || SUDO=doas

available() {  # is the package in the repositories?
  case $PM in
    pacman)  pacman -Si "$1" >/dev/null 2>&1 ;;
    apt-get) apt-cache show "$1" >/dev/null 2>&1 ;;
    dnf)     dnf -q info "$1" >/dev/null 2>&1 ;;
    zypper)  zypper -q info "$1" 2>/dev/null | grep -q "^Name" ;;
    apk)     apk search -qx "$1" 2>/dev/null | grep -qx "$1" ;;
  esac
}
install_repo() {
  [ $# -eq 0 ] && return 0
  case $PM in
    pacman)  run $SUDO pacman -S --needed "$@" ;;
    apt-get) run $SUDO apt-get install -y "$@" ;;
    dnf)     run $SUDO dnf install -y "$@" ;;
    zypper)  run $SUDO zypper install -y "$@" ;;
    apk)     run $SUDO apk add "$@" ;;
  esac
}

install_deps() {
  say "Package manager: $PM${AUR:+ (AUR via $AUR)}"
  if [ "$PM" = none ]; then
    warn "Unknown package manager: install the list in docs/brasa.md by hand"; return
  fi
  [ "$PM" = apt-get ] && run $SUDO apt-get update -qq
  local list="$PKGS" c; c=$(col)
  case $WM in swayfx) list+="$PKGS_SWAY" ;; hyprland) list+="$PKGS_HYPR" ;; both) list+="$PKGS_SWAY$PKGS_HYPR" ;; esac
  [ $GREETER = 1 ] && list+="$PKGS_GREETER"
  [ $ZSH = 1 ] && list+="$PKGS_ZSH"
  # The greeter runs on sway: add it when no sway/SwayFX is coming with the window manager
  [ $GREETER = 1 ] && [ "$WM" = hyprland ] && ! command -v sway >/dev/null && list+=$'\nsway sway sway sway sway sway\n'
  local repo=() aur=() missing=()
  while read -r line; do
    [ -z "$line" ] && continue
    local logical name; logical=$(awk '{print $1}' <<<"$line"); name=$(awk -v c="$c" '{print $c}' <<<"$line")
    if [ "$name" = - ]; then missing+=("$logical"); continue; fi
    if [[ $name == *@aur ]]; then aur+=("${name%@aur}"); continue; fi
    local alt found=""
    for alt in ${name//|/ }; do available "$alt" && { found=$alt; break; }; done
    if [ -n "$found" ]; then repo+=("$found"); else missing+=("$logical ($name)"); fi
  done <<<"$list"
  say "Installing ${#repo[@]} packages from the repositories"
  install_repo "${repo[@]}" || problem "The package manager failed (exit $?) installing: ${repo[*]}"
  if [ ${#aur[@]} -gt 0 ]; then
    if [ -n "$AUR" ]; then
      local want alt done_
      for want in "${aur[@]}"; do
        # SwayFX already there (any variant, e.g. swayfx-git)? Nothing to do
        if [[ $want == swayfx* ]] && sway --version 2>/dev/null | grep -qi swayfx; then
          info "SwayFX already installed: $(sway --version 2>/dev/null)"; continue
        fi
        done_=""
        for alt in ${want//|/ }; do
          say "Installing from the AUR: $alt (if sway is installed it will ask to replace it: say yes)"
          if run $AUR -S --needed "$alt"; then done_=$alt; break; fi
          warn "$alt did not install; trying the next alternative"
        done
        [ -n "$done_" ] || problem "Could not install from the AUR: ${want//|/ or }  ->  retry: $AUR -S ${want%%|*}"
      done
    else
      warn "No yay or paru: cannot install ${aur[*]} from the AUR"
      if [[ " ${aur[*]} " == *" swayfx"* ]]; then
        info "Installing plain sway instead of SwayFX (no effects)"
        install_repo sway || problem "Could not install sway"
      fi
    fi
  fi
  [ ${#missing[@]} -gt 0 ] && problem "Not available in $PM (install by hand if you need them): ${missing[*]}"
  if [[ " ${repo[*]} " == *" sway "* ]]; then
    info "Your distro has no SwayFX: plain sway is installed (no blur or rounded corners)"
  fi
}

install_font() {
  fc-list : family 2>/dev/null | grep -qi "Hack Nerd Font" && return
  say "Downloading Hack Nerd Font into ~/.local/share/fonts"
  local t; t=$(mktemp -d)
  run curl -fsSL -o "$t/Hack.tar.xz" https://github.com/ryanoasis/nerd-fonts/releases/latest/download/Hack.tar.xz &&
  run mkdir -p ~/.local/share/fonts/HackNerd && run tar -xJf "$t/Hack.tar.xz" -C ~/.local/share/fonts/HackNerd &&
  run fc-cache -f >/dev/null
  rm -rf "$t"
}

verify() {  # are the pieces Brasa relies on really there?
  local c miss=()
  for c in waybar swaync swayosd-server wlr-randr starship fuzzel rofi nwg-bar swaybg swayidle hyprlock foot grim slurp wl-copy playerctl \
           brightnessctl jq notify-send python3; do
    command -v $c >/dev/null || miss+=($c)
  done
  [ ${#miss[@]} -gt 0 ] && problem "Missing commands after installing: ${miss[*]}"
  case $WM in
    swayfx|both)
      if ! command -v sway >/dev/null; then problem "sway/SwayFX is not installed"
      elif ! sway --version 2>/dev/null | grep -qi swayfx; then
        problem "Plain sway is installed, not SwayFX: no blur or rounded corners${AUR:+  ->  $AUR -S swayfx (or swayfx-git if it fails to build), then ./install.sh --no-deps}"
      fi ;;
  esac
  case $WM in hyprland|both) command -v Hyprland >/dev/null || problem "Hyprland is not installed" ;; esac
  if [ $GREETER = 1 ]; then
    command -v greetd >/dev/null || problem "greetd is not installed: the login screen was not changed"
    command -v nwg-hello >/dev/null || problem "nwg-hello is not installed: the login screen was not changed"
  fi
}

check_hyprland() {
  command -v Hyprland >/dev/null || return 0
  local v; v=$(Hyprland --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1)
  if [ -n "$v" ] && [ "$(printf '%s\n0.56\n' "$v" | sort -V | head -1)" != 0.56 ]; then
    problem "Hyprland $v is older than 0.56: Brasa's config uses Lua and needs 0.56 or later"
  fi
}

# ---------------------------------------------------------------- main
say "Brasa in $HERE"
if [ "$GREETER" = ask ]; then
  GREETER=0
  dm=$(basename "$(readlink -f /etc/systemd/system/display-manager.service 2>/dev/null)" .service)
  if [ "$dm" = greetd ] && [ -f /etc/nwg-hello/brasa-sway-config ]; then
    GREETER=1   # already Brasa's: keep it up to date
  elif [ -t 0 ] && [ "$dm" != greetd ]; then
    read -r -p "Also use the Brasa login screen (greetd + nwg-hello)${dm:+ instead of $dm}? It applies from the next boot. [y/N] " a
    [[ $a == [yY]* ]] && GREETER=1
  fi
fi
if [ "$ZSH" = ask ]; then
  ZSH=0
  if head -1 ~/.zshrc 2>/dev/null | grep -qF "$ZSH_MARK"; then
    ZSH=1   # already Brasa's: keep it up to date
  elif [ -t 0 ]; then
    read -r -p "Set up zsh with Brasa's config and make it your login shell? Your ~/.zshrc is backed up. [y/N] " a
    [[ $a == [yY]* ]] && ZSH=1
  fi
fi
install_vkms() {  # the virtual screen of the remote session on Hyprland (brasa-remote session wm hyprland)
  modinfo vkms >/dev/null 2>&1 && return 0
  say "The kernel has no vkms module (virtual screen of the remote Hyprland session): installing it"
  [ $DRY = 1 ] && { info "[dry run] $HERE/bin/brasa-remote _vkms"; return 0; }
  "$HERE/bin/brasa-remote" _vkms || problem "No vkms: the remote session can only use sway"
}
[ $DEPS = 1 ] && { install_deps; install_font; }
[ $DEPS = 1 ] && case $WM in hyprland|both) install_vkms ;; esac
[ $DRY = 1 ] || verify
check_hyprland

say "Linking the configuration"
link "$HERE" ~/.config/brasa
for d in $DIRS; do link "$HERE/config/$d" ~/.config/$d; done
for f in $FILES; do link "$HERE/config/$f" ~/.config/$f; done
for f in $OBSOLETE; do
  [ -L ~/.config/$f ] && [[ "$(readlink ~/.config/$f)" == "$HERE"* ]] && run rm ~/.config/$f && info "removed old ~/.config/$f"
done
for f in $BINS; do link "$HERE/bin/$f" ~/.local/bin/$f; done
# HyprMod's file (Settings in the panel), present before its first start so it does not add its own
# require line to hyprland.lua, which belongs to the repository
[ $DRY = 1 ] || touch "$HERE/config/hypr/hyprland-gui.lua"
# The wallpaper, unless one was chosen with brasa-wallpaper
readlink ~/.local/share/backgrounds/brasa.png 2>/dev/null | grep -q '/brasa-custom\.' ||
  link "$HERE/wallpaper.png" ~/.local/share/backgrounds/brasa.png
# The lock screen's follows the desktop's, and the login screen's the lock's, until brasa-wallpaper changes them
[ -e ~/.local/share/backgrounds/brasa-lock.png ] || run ln -s brasa.png ~/.local/share/backgrounds/brasa-lock.png
[ -e ~/.local/share/backgrounds/brasa-login.png ] || run ln -s brasa-lock.png ~/.local/share/backgrounds/brasa-login.png
ini_set() {  # file key value: set one key of a GTK settings.ini (its [Settings] section)
  local f=$1 k=$2 v=$3
  [ $DRY = 1 ] && { info "[dry run] $f: $k=$v"; return; }
  mkdir -p "$(dirname "$f")"; [ -s "$f" ] || echo "[Settings]" > "$f"
  if grep -q "^$k=" "$f"; then sed -i "s|^$k=.*|$k=$v|" "$f"; else echo "$k=$v" >> "$f"; fi
}
if [ $GTK = 1 ]; then
  link "$HERE/config/gtk-3.0/gtk.css" ~/.config/gtk-3.0/gtk.css
  link "$HERE/config/gtk-4.0/gtk.css" ~/.config/gtk-4.0/gtk.css
  # Dark everywhere: GTK 3 apps take adw-gtk3-dark (Brasa's gtk.css goes on top), GTK 4 / libadwaita
  # apps the dark style; red accent where the desktop accent is followed (GNOME 47+)
  if command -v gsettings >/dev/null; then
    run gsettings set org.gnome.desktop.interface color-scheme prefer-dark 2>/dev/null
    run gsettings set org.gnome.desktop.interface accent-color red 2>/dev/null
    [ -d /usr/share/themes/adw-gtk3-dark ] && run gsettings set org.gnome.desktop.interface gtk-theme adw-gtk3-dark 2>/dev/null
  fi
  [ -d /usr/share/themes/adw-gtk3-dark ] && ini_set ~/.config/gtk-3.0/settings.ini gtk-theme-name adw-gtk3-dark
  ini_set ~/.config/gtk-3.0/settings.ini gtk-application-prefer-dark-theme 1
  ini_set ~/.config/gtk-4.0/settings.ini gtk-application-prefer-dark-theme 1
  # Qt apps, through qt5ct / qt6ct (QT_QPA_PLATFORMTHEME=qt5ct is set by hyprland.lua, the login screen
  # and environment.d; Qt 6 apps take it for qt6ct too): Brasa's flat Kvantum theme (config/kvantum,
  # made by docs/gen-kvantum.py) when Kvantum is installed, else the Fusion style with Brasa's colors
  if command -v kvantummanager >/dev/null; then
    qtstyle=kvantum qtpalette=false
    link "$HERE/config/kvantum/Brasa" ~/.config/Kvantum/Brasa
    kv=~/.config/Kvantum/kvantum.kvconfig
    if ! grep -qx "theme=Brasa" "$kv" 2>/dev/null; then
      [ -f "$kv" ] && run cp -a "$kv" "$kv.bak-brasa-$STAMP"
      [ $DRY = 1 ] || printf '[General]\ntheme=Brasa\n' > "$kv"
      info "Kvantum theme: Brasa"
    fi
  else
    qtstyle=Fusion qtpalette=true
  fi
  for v in qt5ct qt6ct; do
    f=~/.config/$v/$v.conf
    [ -f "$f" ] && ! grep -q "brasa-colors.conf" "$f" && run cp -a "$f" "$f.bak-brasa-$STAMP"
    [ $DRY = 1 ] && { info "[dry run] $f: style $qtstyle"; continue; }
    python3 - "$f" "$HERE/config/qt/brasa-colors.conf" "$qtstyle" "$qtpalette" <<'PY'
import os, re, sys
path, colors, style, palette = sys.argv[1:]
want = {"color_scheme_path": colors, "custom_palette": palette, "style": style}
lines = open(path).read().splitlines() if os.path.exists(path) else []
out, section, done = [], None, set()
for line in lines + ["[End]"]:
    m = re.match(r"\[(.+)\]$", line.strip())
    if m:
        if section == "Appearance":
            out += [f"{k}={v}" for k, v in want.items() if k not in done]; done |= set(want)
        section = m.group(1)
        if line != "[End]": out.append(line)
        continue
    key = line.split("=", 1)[0].strip()
    if section == "Appearance" and key in want:
        out.append(f"{key}={want[key]}"); done.add(key)
    else:
        out.append(line)
if not done:
    out = ["[Appearance]"] + [f"{k}={v}" for k, v in want.items()] + out
os.makedirs(os.path.dirname(path), exist_ok=True)
open(path, "w").write("\n".join(out) + "\n")
PY
  done
  run mkdir -p ~/.config/environment.d
  [ $DRY = 1 ] || echo "QT_QPA_PLATFORMTHEME=qt5ct" > ~/.config/environment.d/90-brasa-qt.conf
  # GTK_THEME overrides all of the above, in every GTK app
  for f in ~/.profile ~/.zprofile ~/.bash_profile ~/.pam_environment ~/.config/environment.d/*.conf /etc/environment; do
    [ -f "$f" ] && grep -qE "^[^#]*GTK_THEME=" "$f" &&
      problem "$f sets GTK_THEME, which overrides Brasa's dark theme in every GTK app: remove that line and log in again"
  done
fi
# Avatar for the lock screens: the account picture GNOME/AccountsService already has
if [ ! -e ~/.face ] && [ -r "/var/lib/AccountsService/icons/$USER" ]; then
  run ln -s "/var/lib/AccountsService/icons/$USER" ~/.face; info "~/.face -> account picture"
fi

# SwayFX effects only if what is installed is SwayFX
if sway --version 2>/dev/null | grep -qi swayfx; then fx=fx.swayfx.conf; else fx=fx.none.conf; fi
run ln -sfn "$fx" "$HERE/config/sway/fx.conf"; info "sway: $fx"

if [ $ZSH = 1 ]; then
  say "zsh"
  install_zsh
fi
if [ $PROMPT = 1 ]; then
  say "Shell prompt (starship)"
  link "$HERE/config/starship/starship.toml" ~/.config/starship.toml
  # Brasa's ~/.zshrc already starts starship; other shells get one line in their rc file
  [ "$ZSH" = 1 ] || prompt_hook
fi
if [ $ROOT_PROMPT = 1 ]; then
  say "Shell prompt for root"
  root_prompt
fi

check_local_bin

say "Generating the bar for this machine's hardware"
run python3 "$HERE/docs/gen-waybar-swaync.py" "$HERE/config" || problem "Could not generate the bar (docs/gen-waybar-swaync.py)"

say "Generating key bindings from keys.tsv"
run "$HERE/bin/brasa-gen" || problem "brasa-gen failed: key bindings were not generated"

if [ $GREETER = 1 ]; then
  say "Login screen (greetd + nwg-hello)"
  if command -v greetd >/dev/null && command -v nwg-hello >/dev/null; then
    run "$HERE/bin/brasa-greeter" install --yes || problem "brasa-greeter install failed: the display manager was not changed"
  fi
fi

if [ ${#PROBLEMS[@]} -gt 0 ]; then
  printf '\n%s✗ Finished with %d problem(s):%s\n' "$r" ${#PROBLEMS[@]} "$n"
  for p in "${PROBLEMS[@]}"; do printf '  %s•%s %s\n' "$r" "$n" "$p"; done
  printf '  Everything else is installed and linked. Fix the above and run ./install.sh again (it is safe to repeat).\n'
  exit 1
fi
say "Ready"
cat <<TXT

  To use it: log out and pick ${b}SwayFX${n} or ${b}Hyprland${n} on the login screen
  (gear icon at the bottom right in GDM).
  To try it without leaving GNOME: ${b}brasa-test${n} (sway in a window, Alt instead of Super).
  Key bindings cheat sheet: ${b}Super+F1${n}.   Undo: ${b}./install.sh --uninstall${n}
TXT
