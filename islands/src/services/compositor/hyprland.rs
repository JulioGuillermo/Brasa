use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

const FOCUS_EVENT: &str = "focusedmon>>";

pub fn running() -> bool {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

pub fn follow_focus(on_focus: impl Fn(String)) {
    if let Some(connector) = focused_monitor() {
        on_focus(connector);
    }
    let Ok(stream) = UnixStream::connect(socket(".socket2.sock")) else { return };
    for line in BufReader::new(stream).lines() {
        let Ok(line) = line else { return };
        if let Some(connector) = focused_in_event(&line) {
            on_focus(connector.to_string());
        }
    }
}

/// "focusedmon>>DP-2,3" -> "DP-2"
fn focused_in_event(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(FOCUS_EVENT)?;
    rest.split(',').next()
}

fn focused_monitor() -> Option<String> {
    let mut stream = UnixStream::connect(socket(".socket.sock")).ok()?;
    stream.write_all(b"j/monitors").ok()?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).ok()?;
    let monitors: Vec<serde_json::Value> = serde_json::from_str(&reply).ok()?;
    let focused = monitors.iter().find(|monitor| monitor["focused"] == true)?;
    Some(focused["name"].as_str()?.to_string())
}

fn socket(name: &str) -> PathBuf {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
    PathBuf::from(runtime).join("hypr").join(signature).join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_monitor_of_a_focus_event() {
        assert_eq!(focused_in_event("focusedmon>>DP-2,3"), Some("DP-2"));
        assert_eq!(focused_in_event("workspace>>3"), None);
    }
}
