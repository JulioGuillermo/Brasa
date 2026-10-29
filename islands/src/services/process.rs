//! Running other programs without blocking the GTK thread or leaving zombies behind.

use std::process::{Command, Stdio};

use crate::runtime::workers;

/// A short command (wpctl, brightnessctl): it runs, and is waited for, on a worker thread.
pub fn run(program: &str, arguments: &[&str]) {
    let mut command = Command::new(program);
    command.args(arguments).stdout(Stdio::null()).stderr(Stdio::null());
    workers::run(move || {
        let _ = command.status();
    });
}

/// A shell command that may keep running (an app, a settings window): it leaves in its own session,
/// so closing the islands does not close it.
pub fn detach(shell_command: &str) {
    let mut command = Command::new("setsid");
    command.args(["-f", "sh", "-c", shell_command]).stdout(Stdio::null()).stderr(Stdio::null());
    command.env_remove("LD_PRELOAD");
    workers::run(move || {
        let _ = command.status();
    });
}

/// Whether a shell test succeeds; blocks, so call it from a worker.
pub fn passes(shell_test: &str) -> bool {
    Command::new("sh")
        .args(["-c", shell_test])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
