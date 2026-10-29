//! The only part that knows which window manager runs: which monitor has the focus. Wayland has no
//! standard way to ask, so Hyprland and sway are asked through their own sockets.

mod hyprland;
mod sway;

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
