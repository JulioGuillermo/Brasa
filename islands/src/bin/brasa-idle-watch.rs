//! brasa-idle-watch [SECONDS]: tells whether an app keeps the screen awake, for compositors that do not say
//! it themselves (mango). The standard idle protocol gives two timers: one that apps keeping the screen
//! awake (a video playing) hold back, and one that only counts keyboard and mouse. When the second has
//! fired and the first has not, an app is keeping the screen awake: the file KEPT_AWAKE exists then.
//! So it is known once nobody has touched the keyboard or the mouse for SECONDS (30 by default).

use std::path::PathBuf;

use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::{ext_idle_notification_v1 as notification, ext_idle_notifier_v1 as notifier};

const DEFAULT_SECONDS: u32 = 30;
const INPUT_IDLE_SINCE_VERSION: u32 = 2;

#[derive(Clone, Copy)]
enum Timer {
    /// Held back by apps keeping the screen awake.
    Idle,
    /// Keyboard and mouse only.
    InputIdle,
}

struct State {
    idle: bool,
    input_idle: bool,
    kept_awake_file: PathBuf,
}

impl State {
    fn update(&self) {
        let kept_awake = self.input_idle && !self.idle;
        if kept_awake {
            let _ = std::fs::write(&self.kept_awake_file, "");
        } else {
            let _ = std::fs::remove_file(&self.kept_awake_file);
        }
    }
}

impl Dispatch<notification::ExtIdleNotificationV1, Timer> for State {
    fn event(
        state: &mut Self,
        _: &notification::ExtIdleNotificationV1,
        event: notification::Event,
        timer: &Timer,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let idle = matches!(event, notification::Event::Idled);
        match timer {
            Timer::Idle => state.idle = idle,
            Timer::InputIdle => state.input_idle = idle,
        }
        state.update();
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event, _: &GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(_: &mut Self, _: &wl_seat::WlSeat, _: wl_seat::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<notifier::ExtIdleNotifierV1, ()> for State {
    fn event(_: &mut Self, _: &notifier::ExtIdleNotifierV1, _: notifier::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

fn kept_awake_file() -> PathBuf {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(runtime).join("brasa-kept-awake")
}

fn main() {
    let seconds = std::env::args().nth(1).and_then(|text| text.parse().ok()).unwrap_or(DEFAULT_SECONDS);
    let Ok(connection) = Connection::connect_to_env() else {
        eprintln!("brasa-idle-watch: no Wayland display");
        std::process::exit(1);
    };
    let Ok((globals, mut queue)) = registry_queue_init::<State>(&connection) else { std::process::exit(1) };
    let handle = queue.handle();
    let seat = globals.bind::<wl_seat::WlSeat, _, _>(&handle, 1..=1, ());
    let notifier = globals.bind::<notifier::ExtIdleNotifierV1, _, _>(&handle, INPUT_IDLE_SINCE_VERSION..=INPUT_IDLE_SINCE_VERSION, ());
    let (Ok(seat), Ok(notifier)) = (seat, notifier) else {
        eprintln!("brasa-idle-watch: the compositor has no ext-idle-notify version 2");
        std::process::exit(1);
    };
    let milliseconds = seconds.saturating_mul(1000);
    notifier.get_idle_notification(milliseconds, &seat, &handle, Timer::Idle);
    notifier.get_input_idle_notification(milliseconds, &seat, &handle, Timer::InputIdle);

    let mut state = State { idle: false, input_idle: false, kept_awake_file: kept_awake_file() };
    state.update();
    while queue.blocking_dispatch(&mut state).is_ok() {}
    let _ = std::fs::remove_file(&state.kept_awake_file);
}
