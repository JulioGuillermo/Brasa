"""mango 0.18 renamed most of its config keys and rule options. Brasa writes its mango config in the 0.18
names and, for an older mango, puts back the old ones."""
import json
import os
import re
import subprocess

NEW_NAMES_SINCE = (0, 18)

OLD_KEYS = {
    "key_mode": "keymode", "exec_once": "exec-once", "source_optional": "source-optional",
    "fade_in_begin_opacity": "fadein_begin_opacity", "fade_out_begin_opacity": "fadeout_begin_opacity",
    "animation_curve_opacity_fade_in": "animation_curve_opafadein",
    "animation_curve_opacity_fade_out": "animation_curve_opafadeout",
    "focus_direction_only_zone_overlap": "focusdir_only_zone_overlap", "sync_obj_enable": "syncobj_enable",
    "default_master_factor": "default_mfact", "default_master_count": "default_nmaster",
    "dwindle_vertical_split": "dwindle_vsplit", "dwindle_horizontal_split": "dwindle_hsplit",
    "overview_gap_inner": "overviewgappi", "overview_gap_outer": "overviewgappo", "numlock_on": "numlockon",
    "idle_inhibit_ignore_visible": "idleinhibit_ignore_visible",
    "idle_inhibit_when_fullscreen": "idleinhibit_when_fullscreen",
    "sloppy_focus": "sloppyfocus", "warp_cursor": "warpcursor", "smart_gaps": "smartgaps",
    "gap_inner_horizontal": "gappih", "gap_inner_vertical": "gappiv",
    "gap_outer_horizontal": "gappoh", "gap_outer_vertical": "gappov",
    "special_gap_inner_horizontal": "special_gappih", "special_gap_inner_vertical": "special_gappiv",
    "special_gap_outer_horizontal": "special_gappoh", "special_gap_outer_vertical": "special_gappov",
    "border_px": "borderpx", "root_color": "rootcolor", "shadows_color": "shadowscolor",
    "border_color": "bordercolor", "drop_color": "dropcolor", "split_color": "splitcolor",
    "focus_color": "focuscolor", "maximized_screen_color": "maximizescreencolor", "urgent_color": "urgentcolor",
    "scratchpad_color": "scratchpadcolor", "global_color": "globalcolor", "overlay_color": "overlaycolor",
    "monitor_rule": "monitorrule", "tag_rule": "tagrule", "layer_rule": "layerrule",
    "window_rule": "windowrule", "window_rule_once": "windowrule-once", "device_rule": "devicerule",
}
OLD_RULE_OPTIONS = {
    "tag_rule": {"master_count": "nmaster", "master_factor": "mfact"},
    "layer_rule": {"no_blur": "noblur", "no_animation": "noanim", "no_shadow": "noshadow"},
    "window_rule": {
        "is_floating": "isfloating", "app_id": "appid", "offset_x": "offsetx", "offset_y": "offsety",
        "no_focus": "nofocus", "no_fade_in": "nofadein", "no_fade_out": "nofadeout", "no_border": "isnoborder",
        "no_shadow": "isnoshadow", "no_radius": "isnoradius", "no_animation": "isnoanimation",
        "is_open_silent": "isopensilent", "is_tag_silent": "istagsilent",
        "is_named_scratchpad": "isnamedscratchpad", "is_unmanaged_global": "isunglobal",
        "is_global": "isglobal", "is_overlay": "isoverlay", "no_size_hint": "isnosizehint",
        "idle_inhibit_when_focus": "idleinhibit_when_focus", "is_term": "isterm",
        "force_fake_maximize": "force_fakemaximize", "no_swallow": "noswallow", "no_blur": "noblur",
        "is_fullscreen": "isfullscreen", "is_fake_fullscreen": "isfakefullscreen",
        "global_key_binding": "globalkeybinding",
    },
}
OLD_RULE_OPTIONS["window_rule_once"] = OLD_RULE_OPTIONS["window_rule"]
KEYS_ONLY_IN_NEW = {"auto_reload_config"}


def uses_new_names(version):
    """version: like "0.18.6"; without mango, the new names."""
    found = re.match(r"(\d+)\.(\d+)", version)
    if not found:
        return True
    return (int(found.group(1)), int(found.group(2))) >= NEW_NAMES_SINCE


def mango_version():
    """The running mango's version inside a mango session, else the installed one, like "0.18.6"; "" without
    mango."""
    said = ""
    if os.environ.get("MANGO_INSTANCE_SIGNATURE"):
        try:
            answer = subprocess.run(["mmsg", "get", "version"], capture_output=True, text=True, timeout=3)
            said = json.loads(answer.stdout)["version"]
        except (OSError, subprocess.SubprocessError, ValueError, KeyError):
            pass
    if not said:
        try:
            said = subprocess.run(["mango", "-v"], capture_output=True, text=True, timeout=3).stdout
        except (OSError, subprocess.SubprocessError):
            pass
    found = re.search(r"\d+\.\d+(\.\d+)?", said)
    return found.group(0) if found else ""


NEW_KEYS = {old: new for new, old in OLD_KEYS.items()}


def _rule_options(rule, new_names):
    """The renamed options of this kind of rule, from the names the text may have to the ones mango reads."""
    to_old = OLD_RULE_OPTIONS.get(NEW_KEYS.get(rule, rule), {})
    if new_names:
        return {old: new for new, old in to_old.items()}
    return to_old


def _renamed_option(part, options):
    option, colon, rest = part.partition(":")
    if not colon:
        return part
    return options.get(option, option) + colon + rest


def _renamed_line(line, new_names):
    name, equals, value = line.partition("=")
    if not equals or name.startswith("#"):
        return line
    if not new_names and name in KEYS_ONLY_IN_NEW:
        return None
    options = _rule_options(name, new_names)
    if options:
        value = ",".join(_renamed_option(part, options) for part in value.split(","))
    keys = NEW_KEYS if new_names else OLD_KEYS
    return keys.get(name, name) + "=" + value


def for_mango(text, new_names):
    """text, written with either names, in the names this mango reads."""
    lines = (_renamed_line(line, new_names) for line in text.splitlines())
    return "\n".join(line for line in lines if line is not None) + "\n"
