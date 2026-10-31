use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde_json::Value;

use super::window::{Scope, Window};

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

pub fn windows(scope: Scope) -> Vec<Window> {
    let Some(clients) = request_json("j/clients") else { return Vec::new() };
    let workspace = match scope {
        Scope::Workspace => request_json("j/activeworkspace").and_then(|workspace| workspace["id"].as_i64()),
        Scope::All => None,
    };
    windows_in(&clients, workspace)
}

pub fn focus(address: &str) {
    request(&format!("dispatch hl.dsp.focus({{ window = \"address:{address}\" }})"));
}

/// "focusedmon>>DP-2,3" -> "DP-2"
fn focused_in_event(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(FOCUS_EVENT)?;
    rest.split(',').next()
}

fn focused_monitor() -> Option<String> {
    let monitors = request_json("j/monitors")?;
    let focused = monitors.as_array()?.iter().find(|monitor| monitor["focused"] == true)?;
    Some(focused["name"].as_str()?.to_string())
}

/// The mapped clients, on that workspace when one is given, by focus history.
fn windows_in(clients: &Value, workspace: Option<i64>) -> Vec<Window> {
    let Some(clients) = clients.as_array() else { return Vec::new() };
    let mut found: Vec<(i64, Window)> = clients
        .iter()
        .filter(|client| client["mapped"] == true)
        .filter(|client| workspace.is_none() || client["workspace"]["id"].as_i64() == workspace)
        .map(|client| {
            let history = client["focusHistoryID"].as_i64().unwrap_or(i64::MAX);
            let window = Window {
                id: text(&client["address"]),
                app_id: text(&client["class"]),
                title: text(&client["title"]),
                workspace: text(&client["workspace"]["name"]),
                focused: history == 0,
                capture_id: text(&client["stableId"]),
                pid: client["pid"].as_u64().and_then(|pid| u32::try_from(pid).ok()),
            };
            (history, window)
        })
        .collect();
    found.sort_by_key(|(history, _)| *history);
    found.into_iter().map(|(_, window)| window).collect()
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

fn request_json(command: &str) -> Option<Value> {
    serde_json::from_str(&request(command)?).ok()
}

fn request(command: &str) -> Option<String> {
    let mut stream = UnixStream::connect(socket(".socket.sock")).ok()?;
    stream.write_all(command.as_bytes()).ok()?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).ok()?;
    Some(reply)
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

    #[test]
    fn lists_the_mapped_clients_by_focus_history() {
        let clients = serde_json::json!([
            { "address": "0xa", "mapped": true, "class": "foot", "title": "one",
              "workspace": { "id": 1, "name": "1" }, "focusHistoryID": 2 },
            { "address": "0xb", "mapped": true, "class": "zen", "title": "two",
              "workspace": { "id": 2, "name": "2" }, "focusHistoryID": 0 },
            { "address": "0xc", "mapped": false, "class": "ghost", "title": "",
              "workspace": { "id": 1, "name": "1" }, "focusHistoryID": 1 },
        ]);
        let all = windows_in(&clients, None);
        let ids: Vec<&str> = all.iter().map(|window| window.id.as_str()).collect();
        assert_eq!(ids, ["0xb", "0xa"]);
        assert!(all[0].focused);
        assert_eq!(all[1].workspace, "1");

        let on_one = windows_in(&clients, Some(1));
        assert_eq!(on_one.len(), 1);
        assert_eq!(on_one[0].app_id, "foot");
    }
}
