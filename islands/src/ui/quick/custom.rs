//! Your own tiles, from ~/.config/brasa/panel.conf, one per line:
//!   icon | name | command                              a button
//!   icon | name | on command | off command | test      a switch, lit while the test succeeds

use std::path::PathBuf;

use super::items::{self, Item};

const RUN: &str = "~/.local/bin/brasa-run";

pub fn path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".config/brasa/panel.conf")
}

pub fn tiles() -> Vec<Item> {
    match std::fs::read_to_string(path()) {
        Ok(text) => parse(&text),
        Err(_) => Vec::new(),
    }
}

pub fn parse(text: &str) -> Vec<Item> {
    let mut found = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('|').map(str::trim).collect();
        match tile(&fields) {
            Some(item) => found.push(item),
            None => eprintln!("panel.conf: skipped (needs 3 or 5 fields): {line}"),
        }
    }
    found
}

fn tile(fields: &[&str]) -> Option<Item> {
    match fields {
        [icon, name, command] => Some(items::action(icon, name, &through_run(name, command))),
        [icon, name, on, off, state] => {
            Some(items::switch(icon, name, &through_run(name, on), &through_run(name, off), state))
        }
        _ => None,
    }
}

fn through_run(name: &str, command: &str) -> String {
    format!("{RUN} {} {command}", name.replace(' ', "_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_fields_make_a_button_that_goes_through_brasa_run() {
        let found = parse("X | Do it | echo hi\n");
        assert_eq!(found, vec![items::action("X", "Do it", "~/.local/bin/brasa-run Do_it echo hi")]);
    }

    #[test]
    fn five_fields_make_a_switch_with_its_own_test() {
        let found = parse("X | VPN | vpn up | vpn down | pgrep vpn\n");
        let on = "~/.local/bin/brasa-run VPN vpn up";
        let off = "~/.local/bin/brasa-run VPN vpn down";
        assert_eq!(found, vec![items::switch("X", "VPN", on, off, "pgrep vpn")]);
    }

    #[test]
    fn comments_blank_and_malformed_lines_are_skipped() {
        let found = parse("# mine\n\n  # indented\nX | only two\nX | a | b | c\nY | ok | run\n");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name(), "ok");
    }
}
