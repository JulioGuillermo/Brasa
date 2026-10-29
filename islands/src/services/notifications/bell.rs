//! The bar's bell (waybar's custom/notif block): the island writes its state to a file and tells the
//! bars to read it again.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::runtime::workers;
use crate::services::process;

const WAYBAR_SIGNAL: u8 = 8;

static LATEST_STATE: Mutex<String> = Mutex::new(String::new());
static WRITING: Mutex<()> = Mutex::new(());

/// What the bar shows: "notification" or "none", with "dnd-" before it while Do Not Disturb is on.
pub fn state_name(count: usize, dnd: bool) -> String {
    let state = if count > 0 { "notification" } else { "none" };
    if dnd { format!("dnd-{state}") } else { state.to_string() }
}

/// Returns at once; the file is written on a worker, and the last state asked for is the one that stays.
pub fn ring(count: usize, dnd: bool) {
    *LATEST_STATE.lock().unwrap() = state_name(count, dnd);
    workers::run(write_latest_and_signal);
}

fn write_latest_and_signal() {
    let _writing = WRITING.lock().unwrap();
    let state = LATEST_STATE.lock().unwrap().clone();
    let contents = format!(r#"{{"text": "", "alt": "{state}", "class": "{state}"}}"#);
    let path = bell_path();
    let unfinished = path.with_extension("json.new");
    if fs::write(&unfinished, contents).is_err() || fs::rename(&unfinished, &path).is_err() {
        return;
    }
    let user = current_user_id().to_string();
    process::run("pkill", &[&format!("-RTMIN+{WAYBAR_SIGNAL}"), "-u", &user, "-x", "waybar"]);
}

fn bell_path() -> PathBuf {
    let folder = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(folder).join("brasa-bell.json")
}

fn current_user_id() -> u32 {
    fs::metadata("/proc/self").map(|own| own.uid()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_four_states() {
        assert_eq!(state_name(0, false), "none");
        assert_eq!(state_name(3, false), "notification");
        assert_eq!(state_name(0, true), "dnd-none");
        assert_eq!(state_name(3, true), "dnd-notification");
    }
}
