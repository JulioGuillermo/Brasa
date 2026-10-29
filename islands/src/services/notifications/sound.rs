//! The sound of a new notification: a bell, or a warning for critical ones; none for low ones.

use std::path::Path;

use super::notice::{LOW, Notice};
use crate::services::process;

const SOUNDS: &str = "/usr/share/sounds/freedesktop/stereo";

pub fn chime(notice: &Notice) {
    if notice.urgency == LOW {
        return;
    }
    let file = if notice.is_critical() { "dialog-warning.oga" } else { "bell.oga" };
    let sound = Path::new(SOUNDS).join(file);
    if sound.exists() {
        process::run("paplay", &[&sound.to_string_lossy()]);
    }
}
