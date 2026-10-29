//! sway's IPC: each message is "i3-ipc", its length and its type (both u32), then JSON.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

const MAGIC: &[u8] = b"i3-ipc";
const SUBSCRIBE: u32 = 2;
const GET_OUTPUTS: u32 = 3;

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

fn focused_output() -> Option<String> {
    let mut stream = connect()?;
    send(&mut stream, GET_OUTPUTS, "")?;
    let outputs = receive(&mut stream)?;
    let focused = outputs.as_array()?.iter().find(|output| output["focused"] == true)?;
    Some(focused["name"].as_str()?.to_string())
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

fn receive(stream: &mut UnixStream) -> Option<serde_json::Value> {
    let mut header = [0u8; 14];
    stream.read_exact(&mut header).ok()?;
    let length = u32::from_ne_bytes(header[6..10].try_into().ok()?) as usize;
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload).ok()?;
    serde_json::from_slice(&payload).ok()
}
