//! The only part that knows which window manager runs: which monitor has the focus, which windows are
//! open and how to focus one. Wayland has no standard way to ask, so Hyprland and sway are asked through
//! their own sockets.

mod hyprland;
mod sway;
mod window;

pub use window::{Scope, Window};

use crate::runtime::events::{self, Event, EventSender};

/// Blocks for as long as the session lasts, sending the focused monitor each time it changes.
pub fn follow_focus(events: &EventSender) {
    let on_focus = |connector: String| events::send(events, Event::FocusedMonitor(connector));
    if hyprland::running() {
        hyprland::follow_focus(on_focus);
    } else if sway::running() {
        sway::follow_focus(on_focus);
    }
}

/// The most recently focused first.
pub fn windows(scope: Scope) -> Vec<Window> {
    if hyprland::running() {
        hyprland::windows(scope)
    } else if sway::running() {
        sway::windows(scope)
    } else {
        Vec::new()
    }
}

/// Also shows its workspace when it is on another one.
pub fn focus(window: &Window) {
    if hyprland::running() {
        hyprland::focus(&window.id);
    } else if sway::running() {
        sway::focus(&window.id);
    }
}
