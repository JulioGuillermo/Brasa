//! What `brasa-islands <command>` asks the running process to do.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    /// Start the islands, hidden (brasa-autostart).
    Start,
    /// Music and notifications (Super+m).
    Toggle,
    /// The notification centre (Super+n, the bar's bell).
    Centre,
    /// Do Not Disturb on or off.
    Dnd,
    /// The app launcher (Super+d).
    Launcher,
    /// The current track for a moment.
    Peek,
    Hide,
    /// Read island.conf again.
    Reload,
    /// Quick settings (Super+a).
    Quick,
    /// The window switcher on this workspace's windows, or the next window when it is open (Super+Tab).
    Switch,
    /// The same on every window (Super+Shift+Tab).
    SwitchAll,
    /// The window switcher on every window, until one is chosen (Super+Ctrl+Tab).
    Windows,
}

impl Command {
    pub fn parse(word: Option<&str>) -> Option<Command> {
        let command = match word.unwrap_or("start") {
            "start" => Command::Start,
            "toggle" => Command::Toggle,
            "centre" => Command::Centre,
            "dnd" => Command::Dnd,
            "launcher" => Command::Launcher,
            "peek" => Command::Peek,
            "hide" => Command::Hide,
            "reload" => Command::Reload,
            "quick" => Command::Quick,
            "switch" => Command::Switch,
            "switch-all" => Command::SwitchAll,
            "windows" => Command::Windows,
            _ => return None,
        };
        Some(command)
    }
}
