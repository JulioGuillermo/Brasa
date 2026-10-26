-- Brasa remote session on Hyprland (brasa-remote session, with "session wm hyprland"): the normal
-- Brasa Hyprland, started by the system service brasa-remote-hyprland@<user> on a virtual screen.
dofile(os.getenv("HOME") .. "/.config/hypr/hyprland.lua")

-- The virtual screen (vkms) stays off: the desktop is a headless output created by brasa-remote,
-- whose size follows the viewer. Its monitor rule lives in a file, so a config reload keeps the size.
hl.monitor({ output = "Virtual-1", disabled = true })
pcall(dofile, os.getenv("XDG_RUNTIME_DIR") .. "/brasa-remote/session-monitor.lua")

hl.on("hyprland.start", function()
    hl.exec_cmd("~/.local/bin/brasa-remote _session-vnc")
end)
