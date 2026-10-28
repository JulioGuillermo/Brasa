"""The sound of a new notification: a bell, or a warning for critical ones; none for low ones."""
import os
import subprocess

from .notice import LOW, Notice

SOUNDS = "/usr/share/sounds/freedesktop/stereo"


def chime(notice: Notice):
    if notice.urgency == LOW:
        return
    sound = os.path.join(SOUNDS, "dialog-warning.oga" if notice.critical else "bell.oga")
    if os.path.exists(sound):
        subprocess.Popen(["paplay", sound], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
