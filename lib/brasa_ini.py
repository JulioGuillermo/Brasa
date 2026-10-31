"""Set one key of an ini file (GTK's settings.ini, qt5ct.conf...) and leave every other line as it was."""
import os


def set_key(path, section, key, value):
    try:
        with open(path, encoding="utf-8") as file:
            lines = file.read().splitlines()
    except FileNotFoundError:
        lines = []
    header = f"[{section}]"
    if header not in (line.strip() for line in lines):
        lines += [header]
    output = []
    in_section = False
    written = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("["):
            if in_section and not written:
                output.append(f"{key}={value}")
                written = True
            in_section = stripped == header
        elif in_section and stripped.partition("=")[0].strip() == key:
            if not written:
                output.append(f"{key}={value}")
                written = True
            continue
        output.append(line)
    if not written:
        output.append(f"{key}={value}")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as file:
        file.write("\n".join(output) + "\n")
