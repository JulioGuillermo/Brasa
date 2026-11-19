//! mango, through its command-line client mmsg: "mmsg get ..." answers with JSON, "mmsg watch ..." prints
//! a JSON line on every change. mango lists its windows in layout order, so the focus history is kept
//! here, from the focus changes seen while the islands run.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use serde_json::Value;

use super::window::{Scope, Window};

/// Window ids, the most recently focused first.
static FOCUS_HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn running() -> bool {
    std::env::var_os("MANGO_INSTANCE_SIGNATURE").is_some()
}

pub fn follow_focus(on_focus: impl Fn(String)) {
    std::thread::spawn(follow_window_focus);
    let mut last = String::new();
    watch("all-monitors", |monitors| {
        let Some(connector) = focused_monitor(&monitors) else { return };
        if connector != last {
            last = connector.clone();
            on_focus(connector);
        }
    });
}

pub fn windows(scope: Scope) -> Vec<Window> {
    let Some(clients) = get("all-clients") else { return Vec::new() };
    let tags = match scope {
        Scope::Workspace => get("all-monitors").and_then(|monitors| active_tags(&monitors)),
        Scope::All => None,
    };
    let history = FOCUS_HISTORY.lock().map(|history| history.clone()).unwrap_or_default();
    windows_in(&clients, tags.as_deref(), &history)
}

/// mango shows the window's tag when it is on another one.
pub fn focus(id: &str) {
    let _ = Command::new("mmsg").args(["dispatch", "focusid", &format!("client,{id}")]).output();
}

fn follow_window_focus() {
    watch("focusing-client", |client| {
        let Some(id) = client["id"].as_i64() else { return };
        if let Ok(mut history) = FOCUS_HISTORY.lock() {
            remember_focus(&mut history, id.to_string());
        }
    });
}

fn remember_focus(history: &mut Vec<String>, id: String) {
    history.retain(|known| *known != id);
    history.insert(0, id);
}

/// The monitor with the focus: mango marks it "active".
fn focused_monitor(monitors: &Value) -> Option<String> {
    let monitors = monitors["monitors"].as_array()?;
    let focused = monitors.iter().find(|monitor| monitor["active"] == true).or(monitors.first())?;
    Some(focused["name"].as_str()?.to_string())
}

fn active_tags(monitors: &Value) -> Option<Vec<i64>> {
    let monitors = monitors["monitors"].as_array()?;
    let focused = monitors.iter().find(|monitor| monitor["active"] == true).or(monitors.first())?;
    Some(focused["active_tags"].as_array()?.iter().filter_map(Value::as_i64).collect())
}

/// The windows on those tags when some are given (all of them otherwise): the focused one first, then by
/// focus history, the never focused ones last in mango's order.
fn windows_in(clients: &Value, tags: Option<&[i64]>, history: &[String]) -> Vec<Window> {
    let Some(clients) = clients["clients"].as_array() else { return Vec::new() };
    let mut found: Vec<(usize, Window)> = clients
        .iter()
        .filter(|client| client["is_minimized"] != true && client["is_scratchpad"] != true)
        .filter(|client| tags.is_none_or(|tags| on_any_tag(client, tags)))
        .filter_map(|client| {
            let window = window_of(client)?;
            Some((focus_rank(&window, history), window))
        })
        .collect();
    found.sort_by_key(|(rank, _)| *rank);
    found.into_iter().map(|(_, window)| window).collect()
}

fn focus_rank(window: &Window, history: &[String]) -> usize {
    if window.focused {
        return 0;
    }
    match history.iter().position(|id| *id == window.id) {
        Some(position) => position + 1,
        None => usize::MAX,
    }
}

fn on_any_tag(client: &Value, tags: &[i64]) -> bool {
    client["tags"].as_array().into_iter().flatten().filter_map(Value::as_i64).any(|tag| tags.contains(&tag))
}

fn window_of(client: &Value) -> Option<Window> {
    let tags: Vec<String> =
        client["tags"].as_array().into_iter().flatten().filter_map(Value::as_i64).map(|tag| tag.to_string()).collect();
    Some(Window {
        id: client["id"].as_i64()?.to_string(),
        app_id: text(&client["appid"]),
        title: text(&client["title"]),
        workspace: tags.join("+"),
        focused: client["is_focused"] == true,
        capture_id: text(&client["foreign_toplevel_id"]),
        pid: client["pid"].as_u64().and_then(|pid| u32::try_from(pid).ok()),
    })
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

fn get(what: &str) -> Option<Value> {
    let output = Command::new("mmsg").args(["get", what]).output().ok()?;
    serde_json::from_slice(&output.stdout).ok()
}

/// Blocks while mmsg runs, calling `on_change` with each update.
fn watch(what: &str, mut on_change: impl FnMut(Value)) {
    let Ok(mut child) = Command::new("mmsg").args(["watch", what]).stdout(Stdio::piped()).spawn() else { return };
    let Some(stdout) = child.stdout.take() else { return };
    for line in BufReader::new(stdout).lines() {
        let Ok(line) = line else { break };
        if let Ok(update) = serde_json::from_str(&line) {
            on_change(update);
        }
    }
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clients() -> Value {
        serde_json::json!({ "clients": [
            { "id": 1, "appid": "foot", "title": "one", "tags": [1], "is_focused": false, "pid": 10,
              "foreign_toplevel_id": "aa" },
            { "id": 2, "appid": "zen", "title": "two", "tags": [2], "is_focused": false, "pid": 11 },
            { "id": 3, "appid": "thunar", "title": "three", "tags": [1], "is_focused": true, "pid": 12 },
            { "id": 4, "appid": "hidden", "title": "", "tags": [1], "is_minimized": true },
        ]})
    }

    #[test]
    fn lists_the_focused_window_first_then_by_focus_history() {
        let history = vec!["3".to_string(), "2".to_string(), "1".to_string()];
        let ids: Vec<String> = windows_in(&clients(), None, &history).into_iter().map(|window| window.id).collect();
        assert_eq!(ids, ["3", "2", "1"]);
    }

    #[test]
    fn keeps_only_the_windows_on_the_shown_tags() {
        let windows = windows_in(&clients(), Some(&[1]), &[]);
        let ids: Vec<&str> = windows.iter().map(|window| window.id.as_str()).collect();
        assert_eq!(ids, ["3", "1"]);
        assert_eq!(windows[1].capture_id, "aa");
        assert_eq!(windows[1].workspace, "1");
    }

    #[test]
    fn a_window_focused_again_moves_to_the_front_of_the_history() {
        let mut history = vec!["2".to_string(), "1".to_string()];
        remember_focus(&mut history, "1".to_string());
        assert_eq!(history, ["1", "2"]);
    }

    #[test]
    fn the_focused_monitor_is_the_active_one() {
        let monitors = serde_json::json!({ "monitors": [
            { "name": "eDP-1", "active": false, "active_tags": [1] },
            { "name": "DP-2", "active": true, "active_tags": [3, 4] },
        ]});
        assert_eq!(focused_monitor(&monitors).as_deref(), Some("DP-2"));
        assert_eq!(active_tags(&monitors), Some(vec![3, 4]));
    }
}
