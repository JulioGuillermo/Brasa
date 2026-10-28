"""The bar's bell (waybar's custom/notif block): the island writes its state to a file and tells the
bars to read it again."""
import json
import os
import subprocess

PATH = os.path.join(os.environ.get("XDG_RUNTIME_DIR", "/tmp"), "brasa-bell.json")
WAYBAR_SIGNAL = 8


def ring(count: int, dnd: bool):
    state = "notification" if count else "none"
    if dnd:
        state = f"dnd-{state}"
    with open(f"{PATH}.new", "w") as bell:
        json.dump({"text": "", "alt": state, "class": state}, bell)
    os.replace(f"{PATH}.new", PATH)
    subprocess.Popen(["pkill", f"-RTMIN+{WAYBAR_SIGNAL}", "-u", str(os.getuid()), "-x", "waybar"])
