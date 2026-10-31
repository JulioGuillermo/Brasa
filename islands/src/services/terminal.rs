//! What a terminal window is running, read from /proc: the program in the foreground of its shell
//! and that program's folder.

use std::fs;
use std::path::Path;

/// "~/brasa" while the shell waits, "nvim · ~/brasa" while a program runs in it. None when the
/// window's process has no shell under it.
pub fn running_in(window_pid: u32) -> Option<String> {
    let shell = children(window_pid).into_iter().find(|child| has_terminal(*child))?;
    let foreground = foreground_group(shell).unwrap_or(shell);
    let folder = home_as_tilde(&fs::read_link(format!("/proc/{foreground}/cwd")).ok()?);
    if foreground == shell {
        return Some(folder);
    }
    let program = fs::read_to_string(format!("/proc/{foreground}/comm")).ok()?;
    Some(format!("{}  ·  {folder}", program.trim()))
}

fn children(pid: u32) -> Vec<u32> {
    let Ok(tasks) = fs::read_dir(format!("/proc/{pid}/task")) else { return Vec::new() };
    tasks
        .flatten()
        .filter_map(|task| fs::read_to_string(task.path().join("children")).ok())
        .flat_map(|list| list.split_whitespace().filter_map(|child| child.parse().ok()).collect::<Vec<u32>>())
        .collect()
}

fn has_terminal(pid: u32) -> bool {
    stat_field(pid, TTY_FIELD).is_some_and(|tty| tty != 0)
}

fn foreground_group(pid: u32) -> Option<u32> {
    let group = stat_field(pid, FOREGROUND_GROUP_FIELD)?;
    u32::try_from(group).ok().filter(|group| *group > 0)
}

const TTY_FIELD: usize = 4;
const FOREGROUND_GROUP_FIELD: usize = 5;

/// A field of /proc/PID/stat counted after the program name, which sits in brackets and may hold spaces.
fn stat_field(pid: u32, field: usize) -> Option<i64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    field_after_name(&stat, field)
}

fn field_after_name(stat: &str, field: usize) -> Option<i64> {
    let (_, after_name) = stat.rsplit_once(')')?;
    after_name.split_whitespace().nth(field)?.parse().ok()
}

fn home_as_tilde(folder: &Path) -> String {
    let folder = folder.to_string_lossy().to_string();
    let home = std::env::var("HOME").unwrap_or_default();
    match folder.strip_prefix(&home) {
        Some(rest) if !home.is_empty() => format!("~{rest}"),
        _ => folder,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_fields_after_a_name_with_spaces() {
        let stat = "4242 (my shell) S 4200 4242 4242 34817 5000 4194560";
        assert_eq!(field_after_name(stat, TTY_FIELD), Some(34817));
        assert_eq!(field_after_name(stat, FOREGROUND_GROUP_FIELD), Some(5000));
    }

    #[test]
    fn this_test_has_no_shell_under_it() {
        assert_eq!(running_in(u32::MAX), None);
    }
}
