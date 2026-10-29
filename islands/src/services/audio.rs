//! The default output's volume, through PipeWire's wpctl; pactl tells when it changes.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use super::process;
use crate::runtime::events::{self, Event, EventSender};

const SINK: &str = "@DEFAULT_AUDIO_SINK@";

pub fn volume() -> f64 {
    let Ok(output) = Command::new("wpctl").args(["get-volume", SINK]).output() else { return 0.0 };
    parse_volume(&String::from_utf8_lossy(&output.stdout)).unwrap_or(0.0)
}

pub fn set_volume(level: f64) {
    let level = format!("{:.2}", level.clamp(0.0, 1.0));
    process::run("wpctl", &["set-volume", SINK, &level]);
}

/// Blocks for as long as the sound server runs, sending the volume each time a sink changes.
pub fn follow_volume(events: &EventSender) {
    let mut last = volume();
    events::send(events, Event::Volume(last));
    let Ok(mut pactl) = Command::new("pactl").arg("subscribe").stdout(Stdio::piped()).stderr(Stdio::null()).spawn()
    else {
        return;
    };
    let Some(output) = pactl.stdout.take() else { return };
    for line in BufReader::new(output).lines() {
        let Ok(line) = line else { break };
        if !line.contains("on sink #") {
            continue;
        }
        let now = volume();
        if now != last {
            last = now;
            events::send(events, Event::Volume(now));
        }
    }
    let _ = pactl.wait();
}

/// "Volume: 0.28 [MUTED]" -> 0.28
fn parse_volume(reply: &str) -> Option<f64> {
    reply.split_whitespace().nth(1)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_wpctl_volume() {
        assert_eq!(parse_volume("Volume: 0.28 [MUTED]\n"), Some(0.28));
        assert_eq!(parse_volume("Volume: 1.00\n"), Some(1.0));
        assert_eq!(parse_volume(""), None);
    }
}
