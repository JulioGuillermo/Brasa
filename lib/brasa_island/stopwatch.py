"""Counts how long the current track has been playing, for players that do not report the position."""
import time


class Stopwatch:
    def __init__(self):
        self.counted = 0.0
        self.running_since = None

    def restart(self, playing: bool):
        self.set(0.0, playing)

    def set(self, seconds: float, playing: bool):
        self.counted = seconds
        self.running_since = time.monotonic() if playing else None

    def run(self, playing: bool):
        if playing and self.running_since is None:
            self.running_since = time.monotonic()
        if not playing and self.running_since is not None:
            self.counted = self.seconds()
            self.running_since = None

    def seconds(self) -> float:
        if self.running_since is None:
            return self.counted
        return self.counted + time.monotonic() - self.running_since
