//! sway's IPC: each message is "i3-ipc", its length and its type (both u32), then JSON.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use serde_json::Value;

use super::window::{Scope, Window};

const MAGIC: &[u8] = b"i3-ipc";
const RUN_COMMAND: u32 = 0;
const GET_WORKSPACES: u32 = 1;
const SUBSCRIBE: u32 = 2;
const GET_OUTPUTS: u32 = 3;
const GET_TREE: u32 = 4;
const SCRATCHPAD: &str = "__i3_scratch";

pub fn running() -> bool {
    std::env::var_os("SWAYSOCK").is_some()
}

pub fn follow_focus(on_focus: impl Fn(String)) {
    if let Some(connector) = focused_output() {
        on_focus(connector);
    }
    let Some(mut stream) = connect() else { return };
    if send(&mut stream, SUBSCRIBE, r#"["workspace"]"#).is_none() {
        return;
    }
    while let Some(event) = receive(&mut stream) {
        if event["change"] == "focus" {
            if let Some(output) = event["current"]["output"].as_str() {
                on_focus(output.to_string());
            }
        }
    }
}

pub fn windows(scope: Scope) -> Vec<Window> {
    let Some(tree) = ask(GET_TREE, "") else { return Vec::new() };
    let mut windows = Vec::new();
    collect(&tree, "", &mut windows);
    if scope == Scope::Workspace {
        let current = focused_workspace().unwrap_or_default();
        windows.retain(|window| window.workspace == current);
    }
    windows
}

pub fn focus(con_id: &str) {
    ask(RUN_COMMAND, &format!("[con_id={con_id}] focus"));
}

/// The windows under a node, depth first, each container's children in the order they last had the
/// focus: so the most recently focused window comes first.
fn collect(node: &Value, workspace: &str, windows: &mut Vec<Window>) {
    let workspace = match node["type"].as_str() {
        Some("workspace") => workspace_name(node),
        _ => workspace.to_string(),
    };
    if let Some(window) = window_of(node, &workspace) {
        windows.push(window);
        return;
    }
    for child in children_by_focus(node) {
        collect(child, &workspace, windows);
    }
}

fn workspace_name(node: &Value) -> String {
    match node["name"].as_str() {
        Some(SCRATCHPAD) => "scratchpad".to_string(),
        Some(name) => name.to_string(),
        None => String::new(),
    }
}

/// A leaf with a program behind it: Wayland ones carry an app_id, X11 ones a class.
fn window_of(node: &Value, workspace: &str) -> Option<Window> {
    let is_container = matches!(node["type"].as_str(), Some("con") | Some("floating_con"));
    if !is_container || node["pid"].is_null() {
        return None;
    }
    let app_id = node["app_id"].as_str().or_else(|| node["window_properties"]["class"].as_str()).unwrap_or_default();
    Some(Window {
        id: node["id"].as_i64()?.to_string(),
        app_id: app_id.to_string(),
        title: node["name"].as_str().unwrap_or_default().to_string(),
        workspace: workspace.to_string(),
        focused: node["focused"] == true,
        pid: node["pid"].as_u64().and_then(|pid| u32::try_from(pid).ok()),
    })
}

fn children_by_focus(node: &Value) -> Vec<&Value> {
    let mut children: Vec<&Value> = ["nodes", "floating_nodes"]
        .iter()
        .filter_map(|key| node[key].as_array())
        .flatten()
        .collect();
    let focus_order: Vec<i64> = node["focus"].as_array().into_iter().flatten().filter_map(Value::as_i64).collect();
    children.sort_by_key(|child| {
        let id = child["id"].as_i64();
        focus_order.iter().position(|focused| Some(*focused) == id).unwrap_or(usize::MAX)
    });
    children
}

fn focused_workspace() -> Option<String> {
    let workspaces = ask(GET_WORKSPACES, "")?;
    let focused = workspaces.as_array()?.iter().find(|workspace| workspace["focused"] == true)?;
    Some(focused["name"].as_str()?.to_string())
}

fn focused_output() -> Option<String> {
    let outputs = ask(GET_OUTPUTS, "")?;
    let focused = outputs.as_array()?.iter().find(|output| output["focused"] == true)?;
    Some(focused["name"].as_str()?.to_string())
}

fn ask(kind: u32, payload: &str) -> Option<Value> {
    let mut stream = connect()?;
    send(&mut stream, kind, payload)?;
    receive(&mut stream)
}

fn connect() -> Option<UnixStream> {
    UnixStream::connect(std::env::var("SWAYSOCK").ok()?).ok()
}

fn send(stream: &mut UnixStream, kind: u32, payload: &str) -> Option<()> {
    let mut message = MAGIC.to_vec();
    message.extend((payload.len() as u32).to_ne_bytes());
    message.extend(kind.to_ne_bytes());
    message.extend(payload.as_bytes());
    stream.write_all(&message).ok()
}

fn receive(stream: &mut UnixStream) -> Option<Value> {
    let mut header = [0u8; 14];
    stream.read_exact(&mut header).ok()?;
    let length = u32::from_ne_bytes(header[6..10].try_into().ok()?) as usize;
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload).ok()?;
    serde_json::from_slice(&payload).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_windows_most_recently_focused_first() {
        let tree = serde_json::json!({
            "id": 1, "type": "root", "focus": [3, 2],
            "nodes": [
                { "id": 2, "type": "output", "focus": [20], "nodes": [
                    { "id": 20, "type": "workspace", "name": "1", "focus": [],
                      "nodes": [{ "id": 21, "type": "con", "pid": 5, "app_id": "foot", "name": "term" }] }
                ]},
                { "id": 3, "type": "output", "focus": [30], "nodes": [
                    { "id": 30, "type": "workspace", "name": "2", "focus": [32, 31],
                      "nodes": [{ "id": 31, "type": "con", "pid": 6, "app_id": null,
                                  "window_properties": { "class": "Steam" }, "name": "steam" }],
                      "floating_nodes": [{ "id": 32, "type": "floating_con", "pid": 7, "app_id": "zen",
                                           "name": "web", "focused": true }] }
                ]}
            ]
        });
        let mut windows = Vec::new();
        collect(&tree, "", &mut windows);
        let ids: Vec<&str> = windows.iter().map(|window| window.id.as_str()).collect();
        assert_eq!(ids, ["32", "31", "21"]);
        assert!(windows[0].focused);
        assert_eq!(windows[1].app_id, "Steam");
        assert_eq!(windows[2].workspace, "1");
    }
}
