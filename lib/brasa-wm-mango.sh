# brasa-wm's verbs in mango, sourced by brasa-wm (it sets $verb, $arg and REMOTE_VIEWERS, pick_layout and
# only_sway).
# mango is driven through mmsg: "mmsg dispatch NAME,ARGS" and "mmsg get WHAT" (JSON).

dispatch() { mmsg dispatch "$1" >/dev/null; }

# The layouts Super+w goes through, in config.conf's circle_layout order, and their names
LAYOUT_ORDER=(dwindle tile scroller vertical_scroller center_tile monocle grid)
declare -A LAYOUT_NAMES=(
  [dwindle]="dwindle" [tile]="master" [scroller]="scrolling, side by side"
  [vertical_scroller]="scrolling, one above the other" [center_tile]="master in the middle"
  [monocle]="one at a time" [grid]="grid")
# mmsg reports a layout by its symbol
declare -A LAYOUT_OF_SYMBOL=(
  [DW]=dwindle [T]=tile [S]=scroller [VS]=vertical_scroller [CT]=center_tile [M]=monocle [G]=grid
  [K]=deck [RT]=right_tile [VT]=vertical_tile [VG]=vertical_grid [VK]=vertical_deck [F]=fair [VF]=vertical_fair)

focused_monitor() { mmsg get all-monitors | jq -c '[.monitors[] | select(.active)][0] // .monitors[0]'; }
current_layout() { echo "${LAYOUT_OF_SYMBOL[$(focused_monitor | jq -r .layout_symbol)]:-dwindle}"; }
current_tag() { focused_monitor | jq -r '.active_tags[0]'; }
layout_name() { echo "${LAYOUT_NAMES[$1]:-$1}"; }

next_layout() {
  local i
  for i in "${!LAYOUT_ORDER[@]}"; do
    if [ "${LAYOUT_ORDER[$i]}" = "$1" ]; then
      echo "${LAYOUT_ORDER[$(( (i + 1) % ${#LAYOUT_ORDER[@]} ))]}"
      return
    fi
  done
  echo dwindle
}

# The panel and the menu may name a layout the Hyprland way: take the nearest mango one
mango_layout() {
  case $1 in
    master)                        echo tile ;;
    scrolling|focus)               echo scroller ;;
    scrolling-down|focus-down)     echo vertical_scroller ;;
    *)                             echo "$1" ;;
  esac
}

LAYOUTS_CONF=$(dirname "$(readlink -f "$0")")/../config/mango/layouts.conf

# Remembered per tag as rules config.conf loads; dwindle, the default, is not written
# Remembered per monitor, as each monitor has its own tags (a rule without monitor_name, from before, is
# replaced the first time that tag changes)
save_layout() {
  local monitor=$1 tag=$2 layout=$3
  {
    echo "# Written by brasa-wm (Super+w): each tag's window layout, per monitor"
    grep '^tagrule=' "$LAYOUTS_CONF" 2>/dev/null | grep -v -E "^tagrule=id:$tag,(monitor_name:$monitor,)?layout_name:"
    [ "$layout" = dwindle ] || echo "tagrule=id:$tag,monitor_name:$monitor,layout_name:$layout"
  } > "$LAYOUTS_CONF.new"
  mv "$LAYOUTS_CONF.new" "$LAYOUTS_CONF"
}

set_layout() {
  local tag monitor; tag=$(current_tag); monitor=$(focused_monitor | jq -r .name)
  dispatch "setlayout,$1"
  save_layout "$monitor" "$tag" "$1"
  ~/.local/bin/brasa-osd view-grid-symbolic "Tag $tag: $(layout_name "$1")"
}

# ID -> "FLIP_X FLIP_Y": true when the tiled window has neighbours on its left (above) and none on its
# right (below), so it sits on the far side of the split
split_sides() {
  mmsg get all-clients | jq -r --argjson id "$1" '
    (.clients[] | select(.id == $id)) as $c
    | [.clients[] | select(.id != $id and .is_visible and (.is_floating | not) and .monitor == $c.monitor)] as $others
    | def beside: .y < $c.y + $c.height and $c.y < .y + .height;
      def above_below: .x < $c.x + $c.width and $c.x < .x + .width;
      def flip(before; after): if $c.is_floating then false else (before and (after | not)) end;
    "\(flip(any($others[]; beside and .x + .width <= $c.x + 2); any($others[]; beside and .x >= $c.x + $c.width - 2))) \(flip(any($others[]; above_below and .y + .height <= $c.y + 2); any($others[]; above_below and .y >= $c.y + $c.height - 2)))"'
}

each_monitor() { mmsg get all-monitors | jq -r '.monitors[].name'; }

case $verb in
  close)        dispatch killclient ;;
  kill)         dispatch killclient,force ;;
  fullscreen)   dispatch togglefullscreen ;;
  float)        dispatch togglefloating ;;
  focus)        dispatch "focusdir,$arg" ;;
  move)         dispatch "move_client,$arg" ;;
  ws)           dispatch "view,$arg,0" ;;
  movews)       dispatch "tag,$arg,0" ;;
  focusmon)     dispatch "focusmon,$arg" ;;
  movewsmon)    dispatch "tagmon,$arg" ;;
  layout-next)  dispatch switch_keyboard_layout
                ~/.local/bin/brasa-osd input-keyboard-symbolic "$(mmsg get keyboardlayout | jq -r .layout)" ;;
  reload)       ~/.local/bin/brasa-gen >/dev/null; dispatch reload_config
                ~/.local/bin/brasa-fx restore; ~/.local/bin/brasa-monitors apply --settle ;;   # the config's rules reset them
  layout-cycle|group-layout-cycle|bar-layout-click)
                set_layout "$(next_layout "$(current_layout)")" ;;
  layout-set)   set_layout "$(mango_layout "$arg")" ;;
  layout-current) current_layout ;;
  layout-menu)  MENU_IDS=("${LAYOUT_ORDER[@]}") MENU_NAMES=()
                for l in "${LAYOUT_ORDER[@]}"; do MENU_NAMES+=("$(layout_name "$l")"); done
                picked=$(pick_layout "$(current_layout)") && set_layout "$picked" ;;
  resize)       # wider, narrower, taller or shorter
                case $arg in
                  wider) dx=40 dy=0 ;; narrower) dx=-40 dy=0 ;; taller) dx=0 dy=40 ;; shorter) dx=0 dy=-40 ;;
                  *) exit 1 ;;
                esac
                # In the tiled layouts mango moves the split between windows: on the far side of it, a
                # window grows the other way
                case $(current_layout) in scroller|vertical_scroller) ;; *)
                  read -r flip_x flip_y < <(split_sides "$(mmsg get focusing-client | jq .id)")
                  [ "$flip_x" = true ] && dx=$(( -dx ))
                  [ "$flip_y" = true ] && dy=$(( -dy )) ;;
                esac
                dispatch "resizewin,$(printf '%+d' "$dx"),$(printf '%+d' "$dy")" ;;
  width-cycle)  # the focused window's width, through config.conf's scroller_proportion_preset
                case $(current_layout) in
                  scroller|vertical_scroller) dispatch switch_proportion_preset ;;
                  *) ~/.local/bin/brasa-osd view-grid-symbolic "Window widths: only in the scrolling layouts (Super+w)" ;;
                esac ;;
  split-toggle) if [ "$(current_layout)" = dwindle ]; then dispatch dwindle_toggle_current_split
                else ~/.local/bin/brasa-osd view-grid-symbolic "Split: only in the dwindle layout (Super+w)"; fi ;;
  focus-parent|focus-child) only_sway ;;
  dpms)         if [ "$arg" = off ]; then action=sleep_monitor; else action=wakeup_monitor; fi
                for monitor in $(each_monitor); do dispatch "$action,$monitor"; done ;;
  remote-keys)  if [ "$(mmsg get keymode | jq -r .keymode)" = remote ]; then dispatch setkeymode,default; msg=off
                else dispatch setkeymode,remote; msg="on · Ctrl+Super+Esc to leave"; fi
                ~/.local/bin/brasa-osd input-keyboard-symbolic "Remote keyboard $msg" ;;
  remote-watch) # the remote key mode while a viewer has the focus, as Hyprland's config does
                mmsg watch focusing-client | jq --unbuffered -r '.appid // ""' | while read -r app; do
                  mode=$(mmsg get keymode | jq -r .keymode)
                  if [[ $app =~ $REMOTE_VIEWERS ]]; then
                    [ "$mode" = remote ] || { dispatch setkeymode,remote
                      ~/.local/bin/brasa-osd input-keyboard-symbolic "Remote keyboard on · Ctrl+Super+Esc to leave"; }
                  elif [ "$mode" = remote ]; then dispatch setkeymode,default; fi
                done ;;
  keys-capture) if [ "$arg" = on ]; then dispatch setkeymode,remote; else dispatch setkeymode,default; fi ;;
  exit)         dispatch quit ;;
  touchpad)     st=${XDG_RUNTIME_DIR:-/tmp}/brasa-touchpad-off
                if [ -e "$st" ]; then rm -f "$st"; state=on; else touch "$st"; state=off; fi
                dispatch toggle_trackpad_enable
                ~/.local/bin/brasa-osd input-touchpad-symbolic "Touchpad $state" ;;
esac
