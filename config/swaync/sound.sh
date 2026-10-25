#!/usr/bin/env bash
# Notification sound: a bell for normal ones, a warning for critical ones,
# nothing in "do not disturb" or for low-urgency ones (brasa-fx notices, screenshots...).
[ "$(swaync-client -D 2>/dev/null)" = true ] && exit 0
s=/usr/share/sounds/freedesktop/stereo
case $SWAYNC_URGENCY in
  Low) exit 0 ;;
  Critical) f=$s/dialog-warning.oga ;;
  *) f=$s/bell.oga ;;
esac
[ -f "$f" ] && exec paplay "$f"
