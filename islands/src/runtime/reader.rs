//! Threads that wait on the system's event streams (the window manager, the sound server) and turn what
//! they read into Events. Each blocks on its own stream, so none of them costs anything while quiet.

use std::thread;

use super::events::EventSender;
use crate::services::{audio, compositor};

pub fn start(events: &EventSender) {
    let to_focus = events.clone();
    spawn("compositor", move || compositor::follow_focus(&to_focus));
    let to_audio = events.clone();
    spawn("audio", move || audio::follow_volume(&to_audio));
}

fn spawn(name: &str, body: impl FnOnce() + Send + 'static) {
    thread::Builder::new().name(name.into()).spawn(body).expect("a reader thread");
}
