//! Follows the MPRIS players on the session bus and keeps one current: the last one that started playing.
//! Everything goes through GDBus, which does its I/O on its own thread and calls back on the GTK one.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gtk4::gio::prelude::*;
use gtk4::{gio, glib};

use super::track::{Status, Track};

const PREFIX: &str = "org.mpris.MediaPlayer2.";
const PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const BUS: &str = "org.freedesktop.DBus";
const CALL_TIMEOUT_MS: i32 = 2000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Change {
    /// Another song.
    Track,
    /// Played, paused or stopped.
    Status,
    /// Another player became the current one.
    Player,
    /// Same song, other details (its length or cover arrived).
    Details,
}

#[derive(Clone, Debug)]
pub struct Player {
    /// Its well-known name, org.mpris.MediaPlayer2.…
    pub name: String,
    owner: String,
    pub status: Status,
    pub track: Track,
}

impl Player {
    /// What to show while it gives no title: "firefox" from org.mpris.MediaPlayer2.firefox.instance_1_2
    pub fn short_name(&self) -> &str {
        let rest = self.name.strip_prefix(PREFIX).unwrap_or(&self.name);
        rest.split('.').next().unwrap_or(rest)
    }
}

pub struct Players {
    me: Weak<Players>,
    bus: gio::DBusConnection,
    players: RefCell<Vec<Player>>,
    current: RefCell<Option<String>>,
    on_change: Box<dyn Fn(Change)>,
    subscriptions: RefCell<Vec<gio::SignalSubscription>>,
}

impl Players {
    pub fn start(bus: &gio::DBusConnection, on_change: impl Fn(Change) + 'static) -> Rc<Players> {
        let players = Rc::new_cyclic(|me| Players {
            me: me.clone(),
            bus: bus.clone(),
            players: RefCell::new(Vec::new()),
            current: RefCell::new(None),
            on_change: Box::new(on_change),
            subscriptions: RefCell::new(Vec::new()),
        });
        players.listen();
        players.find_running();
        players
    }

    pub fn current(&self) -> Option<Player> {
        let current = self.current.borrow();
        let name = current.as_ref()?;
        self.players.borrow().iter().find(|player| &player.name == name).cloned()
    }

    pub fn is_playing(&self) -> bool {
        self.current().is_some_and(|player| player.status == Status::Playing)
    }

    /// Asks the current player where it is; on_position(None) when it does not answer.
    pub fn ask_position(&self, on_position: impl FnOnce(Option<f64>) + 'static) {
        let Some(player) = self.current() else {
            on_position(None);
            return;
        };
        self.bus.call(
            Some(&player.name),
            PATH,
            PROPERTIES,
            "Get",
            Some(&(PLAYER, "Position").to_variant()),
            Some(glib::VariantTy::new("(v)").unwrap()),
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
            None::<&gio::Cancellable>,
            move |reply| on_position(reply.ok().and_then(|reply| position_in(&reply))),
        );
    }

    pub fn play_pause(&self) {
        self.tell_current("PlayPause", None);
    }

    pub fn next(&self) {
        self.tell_current("Next", None);
    }

    pub fn previous(&self) {
        self.tell_current("Previous", None);
    }

    pub fn seek_to(&self, seconds: f64) {
        let Some(player) = self.current() else { return };
        let Ok(track) = glib::variant::ObjectPath::try_from(player.track.id.clone()) else { return };
        let microseconds = (seconds * 1e6) as i64;
        self.tell_current("SetPosition", Some((track, microseconds).to_variant()));
    }

    fn tell_current(&self, method: &str, arguments: Option<glib::Variant>) {
        let Some(player) = self.current() else { return };
        self.bus.call(
            Some(&player.name),
            PATH,
            PLAYER,
            method,
            arguments.as_ref(),
            None,
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
            None::<&gio::Cancellable>,
            |_| {},
        );
    }

    // ---- Following the bus

    fn listen(&self) {
        let me = self.me.clone();
        let names = self.bus.subscribe_to_signal(
            Some(BUS),
            Some(BUS),
            Some("NameOwnerChanged"),
            Some("/org/freedesktop/DBus"),
            Some("org.mpris.MediaPlayer2"),
            gio::DBusSignalFlags::MATCH_ARG0_NAMESPACE,
            move |signal| {
                let Some(players) = me.upgrade() else { return };
                let Some((name, _old, new)) = signal.parameters.get::<(String, String, String)>() else { return };
                if new.is_empty() {
                    players.forget(&name);
                } else {
                    players.meet(&name, &new);
                }
            },
        );
        let me = self.me.clone();
        let properties = self.bus.subscribe_to_signal(
            None,
            Some(PROPERTIES),
            Some("PropertiesChanged"),
            Some(PATH),
            Some(PLAYER),
            gio::DBusSignalFlags::NONE,
            move |signal| {
                let Some(players) = me.upgrade() else { return };
                let changed = signal.parameters.child_value(1);
                players.on_properties(signal.sender_name, &changed);
            },
        );
        self.subscriptions.borrow_mut().extend([names, properties]);
    }

    fn find_running(&self) {
        let me = self.me.clone();
        self.bus.call(
            Some(BUS),
            "/org/freedesktop/DBus",
            BUS,
            "ListNames",
            None,
            Some(glib::VariantTy::new("(as)").unwrap()),
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
            None::<&gio::Cancellable>,
            move |reply| {
                let Some(players) = me.upgrade() else { return };
                let Some((names,)) = reply.ok().and_then(|reply| reply.get::<(Vec<String>,)>()) else { return };
                for name in names.iter().filter(|name| name.starts_with(PREFIX)) {
                    players.ask_owner(name);
                }
            },
        );
    }

    fn ask_owner(&self, name: &str) {
        let me = self.me.clone();
        let player_name = name.to_string();
        self.bus.call(
            Some(BUS),
            "/org/freedesktop/DBus",
            BUS,
            "GetNameOwner",
            Some(&(name,).to_variant()),
            Some(glib::VariantTy::new("(s)").unwrap()),
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
            None::<&gio::Cancellable>,
            move |reply| {
                let Some(players) = me.upgrade() else { return };
                if let Some((owner,)) = reply.ok().and_then(|reply| reply.get::<(String,)>()) {
                    players.meet(&player_name, &owner);
                }
            },
        );
    }

    /// A player showed up: its properties are asked, and it joins when they come.
    fn meet(&self, name: &str, owner: &str) {
        let me = self.me.clone();
        let player = Player { name: name.to_string(), owner: owner.to_string(), status: Status::Stopped, track: Track::default() };
        self.bus.call(
            Some(name),
            PATH,
            PROPERTIES,
            "GetAll",
            Some(&(PLAYER,).to_variant()),
            Some(glib::VariantTy::new("(a{sv})").unwrap()),
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
            None::<&gio::Cancellable>,
            move |reply| {
                let Some(players) = me.upgrade() else { return };
                let Ok(reply) = reply else { return };
                let mut player = player;
                read_properties(&mut player, &reply.child_value(0));
                players.join(player);
            },
        );
    }

    fn join(&self, player: Player) {
        let playing = player.status == Status::Playing;
        let name = player.name.clone();
        {
            let mut players = self.players.borrow_mut();
            players.retain(|known| known.name != name);
            players.push(player);
        }
        if self.current.borrow().is_none() || playing {
            self.make_current(Some(name));
        }
    }

    fn forget(&self, name: &str) {
        self.players.borrow_mut().retain(|player| player.name != name);
        if self.current.borrow().as_deref() != Some(name) {
            return;
        }
        let next = {
            let players = self.players.borrow();
            let playing = players.iter().find(|player| player.status == Status::Playing);
            playing.or(players.first()).map(|player| player.name.clone())
        };
        self.make_current(next);
    }

    fn make_current(&self, name: Option<String>) {
        *self.current.borrow_mut() = name;
        (self.on_change)(Change::Player);
    }

    fn on_properties(&self, owner: &str, changed: &glib::Variant) {
        let (name, before, after) = {
            let mut players = self.players.borrow_mut();
            let Some(player) = players.iter_mut().find(|player| player.owner == owner) else { return };
            let before = player.clone();
            read_properties(player, changed);
            (player.name.clone(), before, player.clone())
        };
        let is_current = self.current.borrow().as_deref() == Some(name.as_str());
        if !is_current {
            if after.status == Status::Playing && before.status != Status::Playing {
                self.make_current(Some(name));
            }
            return;
        }
        if after.status != before.status {
            (self.on_change)(Change::Status);
        } else if !after.track.same_song(&before.track) {
            (self.on_change)(Change::Track);
        } else if after.track != before.track {
            (self.on_change)(Change::Details);
        }
    }
}

fn read_properties(player: &mut Player, properties: &glib::Variant) {
    let properties = glib::VariantDict::new(Some(properties));
    if let Some(status) = properties.lookup_value("PlaybackStatus", None).and_then(|value| value.str().map(Status::from_mpris)) {
        player.status = status;
    }
    if let Some(metadata) = properties.lookup_value("Metadata", None) {
        player.track = Track::from_metadata(&metadata);
    }
}

/// The reply of Get(Position): (v) holding microseconds.
fn position_in(reply: &glib::Variant) -> Option<f64> {
    let value = reply.child_value(0).as_variant()?;
    let microseconds = value.get::<i64>().or_else(|| value.get::<u64>().map(|position| position as i64))?;
    Some(microseconds.max(0) as f64 / 1e6)
}
