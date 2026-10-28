"""The default output's volume, through wpctl (PipeWire)."""
import subprocess

SINK = "@DEFAULT_AUDIO_SINK@"


def read() -> float:
    try:
        reply = subprocess.run(["wpctl", "get-volume", SINK], capture_output=True, text=True, timeout=1).stdout
        return float(reply.split()[1])
    except (OSError, subprocess.SubprocessError, IndexError, ValueError):
        return 0.0


def write(level: float):
    subprocess.Popen(["wpctl", "set-volume", SINK, f"{level:.2f}"],
                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
