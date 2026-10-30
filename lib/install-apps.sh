# Sourced by install.sh: which app each brasa-app role opens (apps.conf), asked once and installed.
# Uses install.sh's HERE, ASK, DRY, say, info and run.

APPS_CONF="$HERE/apps.conf"

APP_ROLES="terminal terminal2 browser files files2 calculator whiteboard"
declare -A APP_TITLE=(
  [terminal]="Terminal (Super+Enter)"
  [terminal2]="Second terminal (Super+Shift+Enter)"
  [browser]="Browser (Super+b)"
  [files]="File manager (Super+e)"
  [files2]="Second file manager (Super+Shift+n)"
  [calculator]="Calculator (Super+c)"
  [whiteboard]="Draw on the screen (Super+x)"
)
declare -A APP=()

# command           pacman                  apt                     dnf                     zypper                  apk
APP_PKGS='
foot                foot                    foot                    foot                    foot                    foot
kitty               kitty                   kitty                   kitty                   kitty                   kitty
ghostty             ghostty                 -                       -                       -                       -
zen-browser         zen-browser-bin@aur     -                       -                       -                       -
firefox             firefox                 firefox-esr|firefox     firefox                 MozillaFirefox          firefox
google-chrome-stable google-chrome@aur      -                       -                       -                       -
chromium            chromium                chromium                chromium                chromium                chromium
thunar              thunar                  thunar                  Thunar                  thunar                  thunar
nautilus            nautilus                nautilus                nautilus                nautilus                nautilus
nemo                nemo                    nemo                    nemo                    nemo                    nemo
pcmanfm             pcmanfm                 pcmanfm                 pcmanfm                 pcmanfm                 pcmanfm
dolphin             dolphin                 dolphin                 dolphin                 dolphin                 dolphin
qalculate-gtk       qalculate-gtk           qalculate-gtk           qalculate-gtk           qalculate-gtk           qalculate-gtk
gnome-calculator    gnome-calculator        gnome-calculator        gnome-calculator        gnome-calculator        gnome-calculator
wayscriber          wayscriber-bin@aur      -                       -                       -                       -
'

configured_app() {  # role: its line in apps.conf, if any
  sed -n "s/^$1=//p" "$APPS_CONF" 2>/dev/null | tail -1
}

role_options() {  # role: the apps offered, without the one the main role took (terminal2, files2)
  local role=$1 app
  for app in $("$HERE/bin/brasa-app" --options "$role"); do
    case $role in
      terminal2|files2) [ "$app" = "${APP[${role%2}]}" ] && continue ;;
    esac
    echo "$app"
  done
}

can_be_none() {
  [ "$1" != terminal ] && [ "$1" != files ]
}

default_app() {  # role: apps.conf's choice, else the first installed option, else the first option (or none)
  local role=$1 app
  app=$(configured_app "$role")
  if [ -n "$app" ]; then
    echo "$app"
    return
  fi
  for app in $(role_options "$role"); do
    if command -v "$app" >/dev/null; then
      echo "$app"
      return
    fi
  done
  case $role in
    terminal2|files2) echo none ;;
    *) role_options "$role" | head -1 ;;
  esac
}

ask_app() {  # role default: a numbered menu; Enter keeps the default
  local role=$1 default=$2 options=() app i mark answer
  mapfile -t options < <(role_options "$role")
  can_be_none "$role" && options+=(none)
  [[ " ${options[*]} " == *" $default "* ]] || options=("$default" "${options[@]}")
  printf '\n%s\n' "${APP_TITLE[$role]}" >&2
  for i in "${!options[@]}"; do
    app=${options[$i]}
    mark=""
    if [ "$app" = none ]; then mark=" (no app)"
    elif command -v "$app" >/dev/null; then mark=" (installed)"
    fi
    [ "$app" = "$default" ] && mark+="  <- Enter"
    printf '  %d) %s%s\n' $((i + 1)) "$app" "$mark" >&2
  done
  while true; do
    read -r -p "  Number, or the command of another app: " answer
    if [ -z "$answer" ]; then
      echo "$default"
      return
    fi
    if [[ $answer =~ ^[0-9]+$ ]] && [ "$answer" -ge 1 ] && [ "$answer" -le ${#options[@]} ]; then
      echo "${options[$((answer - 1))]}"
      return
    fi
    if [[ $answer =~ ^[A-Za-z0-9._+-]+$ ]]; then
      echo "$answer"
      return
    fi
  done
}

choose_apps() {  # fills APP; asks for the roles apps.conf lacks, or for every role with $1 = all
  local which=$1 role default
  for role in $APP_ROLES; do
    default=$(default_app "$role")
    if [ $ASK = 1 ] && { [ "$which" = all ] || [ -z "$(configured_app "$role")" ]; }; then
      APP[$role]=$(ask_app "$role" "$default")
    else
      APP[$role]=$default
    fi
  done
}

save_apps() {
  local role
  if [ $DRY = 1 ]; then
    for role in $APP_ROLES; do info "[dry run] apps.conf: $role=${APP[$role]}"; done
    return
  fi
  {
    echo "# Which app each brasa-app key opens (none: no app). ./install.sh --apps asks again."
    for role in $APP_ROLES; do echo "$role=${APP[$role]}"; done
  } > "$APPS_CONF"
}

app_package_lines() {  # the APP_PKGS lines of the chosen apps that are not installed yet
  local role app
  for role in $APP_ROLES; do
    app=${APP[$role]}
    [ "$app" = none ] && continue
    command -v "$app" >/dev/null && continue
    grep -E "^$app " <<<"$APP_PKGS"
  done | sort -u
}

missing_apps() {  # the chosen apps still not installed
  local role app
  for role in $APP_ROLES; do
    app=${APP[$role]}
    [ "$app" = none ] && continue
    command -v "$app" >/dev/null || echo "$app ($role)"
  done
}
