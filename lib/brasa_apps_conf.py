"""apps.conf: which app each brasa-app role opens. The roles and their known apps come from brasa-app."""
import os
import subprocess

BRASA = os.path.realpath(os.path.expanduser("~/.config/brasa"))
APPS_CONF = os.path.join(BRASA, "apps.conf")
BRASA_APP = os.path.join(BRASA, "bin", "brasa-app")
NONE = "none"
HEADER = "# Which app each brasa-app key opens (none: no app). ./install.sh --apps asks again."
ROLES = [
    ("terminal", "Terminal", "Super+Enter, Super+t"),
    ("terminal2", "Second terminal", "Super+Shift+Enter, Super+Shift+t"),
    ("browser", "Browser", "Super+b"),
    ("files", "File manager", "Super+e"),
    ("files2", "Second file manager", "Super+Shift+n"),
    ("calculator", "Calculator", "Super+c, Calculator key"),
    ("whiteboard", "Draw on the screen", "Super+x, Launch1 key"),
]
OPTIONAL = {"terminal2", "files2", "browser", "calculator", "whiteboard"}


def known_apps(role):
    result = subprocess.run([BRASA_APP, "--options", role], capture_output=True, text=True)
    return result.stdout.split()


def load():
    """{role: app} as apps.conf sets it; a role it lacks is missing."""
    chosen = {}
    try:
        with open(APPS_CONF, encoding="utf-8") as file:
            lines = file.read().splitlines()
    except FileNotFoundError:
        return chosen
    for line in lines:
        role, equals, app = line.partition("=")
        if equals and not role.startswith("#"):
            chosen[role.strip()] = app.strip()
    return chosen


def save(chosen):
    lines = [HEADER] + [f"{role}={chosen[role]}" for role, _, _ in ROLES if chosen.get(role)]
    with open(APPS_CONF, "w", encoding="utf-8") as file:
        file.write("\n".join(lines) + "\n")


def which_app(role):
    """The app brasa-app opens for the role now, empty when none."""
    result = subprocess.run([BRASA_APP, "--which", role], capture_output=True, text=True)
    return result.stdout.strip()
