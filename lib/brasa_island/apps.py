"""The installed apps (their .desktop files): listing, matching a search, launching."""
import os
import re
import shlex
import subprocess

from gi.repository import Gdk, Gio, GLib

TERMINAL = "foot"
ISLAND_ONLY_VARIABLES = ("LD_PRELOAD", "PYTHONPATH")
FIELD_CODE = re.compile(r"%[a-zA-Z]")
NAME_STARTS, WORD_STARTS, NAME_CONTAINS, ELSEWHERE = range(4)


def installed() -> list[Gio.DesktopAppInfo]:
    apps = [app for app in Gio.AppInfo.get_all() if isinstance(app, Gio.DesktopAppInfo) and app.should_show()]
    return sorted(apps, key=lambda app: app.get_display_name().casefold())


def rank(app: Gio.DesktopAppInfo, search: str):
    """How well the app matches a search (lower is better), or None when it does not."""
    name = app.get_display_name().casefold()
    if name.startswith(search):
        return NAME_STARTS
    if any(word.startswith(search) for word in name.split()):
        return WORD_STARTS
    if search in name:
        return NAME_CONTAINS
    others = [app.get_generic_name() or "", app.get_description() or "", app.get_executable() or "",
              *(app.get_keywords() or [])]
    if any(search in other.casefold() for other in others):
        return ELSEWHERE
    return None


def matching(apps, search: str) -> list[Gio.DesktopAppInfo]:
    search = search.strip().casefold()
    if not search:
        return list(apps)
    ranked = [(rank(app, search), position, app) for position, app in enumerate(apps)]
    return [app for score, _position, app in sorted(r for r in ranked if r[0] is not None)]


def terminal_command(app: Gio.DesktopAppInfo) -> list[str]:
    command = FIELD_CODE.sub("", app.get_commandline() or app.get_executable())
    return [TERMINAL, "-e", *shlex.split(command)]


def app_environment() -> dict:
    """The island's environment without what only the island needs: the layer-shell preload breaks
    other GTK and Electron apps."""
    return {name: value for name, value in os.environ.items() if name not in ISLAND_ONLY_VARIABLES}


def launch(app: Gio.DesktopAppInfo):
    if app.get_boolean("Terminal"):
        subprocess.Popen(terminal_command(app), env=app_environment(), start_new_session=True,
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return
    context = Gdk.Display.get_default().get_app_launch_context()
    for name in ISLAND_ONLY_VARIABLES:
        context.unsetenv(name)
    try:
        app.launch([], context)
    except GLib.Error:
        pass
