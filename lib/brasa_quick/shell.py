"""Runs the panel's shell commands without blocking it."""
import subprocess

from gi.repository import Gio


def run(command: str):
    subprocess.Popen(["sh", "-c", command], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                     start_new_session=True)


def test(command: str, on_result):
    """on_result(True) when the command succeeds."""
    process = Gio.Subprocess.new(["sh", "-c", command],
                                 Gio.SubprocessFlags.STDOUT_SILENCE | Gio.SubprocessFlags.STDERR_SILENCE)
    process.wait_check_async(None, lambda _process, result: on_result(succeeded(process, result)))


def succeeded(process, result) -> bool:
    try:
        return process.wait_check_finish(result)
    except Exception:
        return False
