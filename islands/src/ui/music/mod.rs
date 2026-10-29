//! The music section: what is playing, with cover, progress, controls and volume. Its widget shows one
//! of two views: a line (the peek, or beside notifications) or the full card.

mod card;
mod clock;
mod cover;
mod hold;
mod label;
mod line;

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};

use crate::runtime::events::EventSender;
use crate::services::audio;
use crate::services::media::cover as cover_source;
use crate::services::media::players::{Change, Players};
use crate::services::media::track::Status;
use card::{Card, Controls};
use clock::clock;
use hold::Hold;
use line::Line;

const POSITION_POLL: Duration = Duration::from_millis(500);
const VOLUME_WRITE_EVERY: Duration = Duration::from_millis(40);
const NOTHING_PLAYING: &str = "Nothing playing";
const ICON_PLAYING: &str = "media-playback-pause-symbolic";
const ICON_NOT_PLAYING: &str = "media-playback-start-symbolic";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    Line,
    Full,
}

pub struct Music {
    stack: gtk4::Stack,
    line: Line,
    card: Card,
    players: Option<Rc<Players>>,
    events: EventSender,
    on_peek: Box<dyn Fn()>,
    on_resize: Box<dyn Fn()>,
    shown: Cell<bool>,
    position_poll: RefCell<Option<glib::SourceId>>,
    cover_url: RefCell<String>,
    seek_hold: Hold,
    volume_hold: Hold,
    volume_write_waiting: Cell<bool>,
}

impl Music {
    /// events: for the covers it has the workers load. on_peek: a new track or play/pause, for the
    /// island to peek. on_resize: its height changed.
    pub fn new(events: &EventSender, on_peek: impl Fn() + 'static, on_resize: impl Fn() + 'static) -> Rc<Music> {
        let music = Rc::new_cyclic(|me: &Weak<Music>| Music {
            stack: build_stack(),
            line: Line::new(),
            card: Card::new(controls(me)),
            players: start_players(me),
            events: events.clone(),
            on_peek: Box::new(on_peek),
            on_resize: Box::new(on_resize),
            shown: Cell::new(false),
            position_poll: RefCell::new(None),
            cover_url: RefCell::new(String::new()),
            seek_hold: Hold::default(),
            volume_hold: Hold::default(),
            volume_write_waiting: Cell::new(false),
        });
        music.stack.add_named(music.line.widget(), Some("line"));
        music.stack.add_named(music.card.widget(), Some("full"));
        music.refresh();
        music
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.stack.clone().upcast()
    }

    pub fn set_view(&self, view: View) {
        let name = match view {
            View::Line => "line",
            View::Full => "full",
        };
        self.stack.set_visible_child_name(name);
    }

    /// While shown it follows the position; hidden it costs nothing.
    pub fn set_shown(self: &Rc<Self>, shown: bool) {
        if shown == self.shown.get() {
            return;
        }
        self.shown.set(shown);
        if let Some(source) = self.position_poll.borrow_mut().take() {
            source.remove();
        }
        if shown {
            self.follow_position();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.players.as_ref().is_some_and(|players| players.is_playing())
    }

    /// Whether its two views are still crossfading in size.
    pub fn is_resizing(&self) -> bool {
        self.stack.is_transition_running()
    }

    pub fn show_volume(&self, level: f64) {
        if self.volume_hold.is_active() {
            return;
        }
        self.card.set_volume(level);
    }

    pub fn show_cover(&self, url: &str, picture: Option<&gdk::Texture>) {
        if url != *self.cover_url.borrow() {
            return;
        }
        self.line.show_cover(picture);
        self.card.show_cover(picture);
    }

    fn on_players_change(&self, change: Change) {
        self.refresh();
        if matches!(change, Change::Track | Change::Status) {
            (self.on_peek)();
        }
    }

    fn refresh(&self) {
        let Some(player) = self.players.as_ref().and_then(|players| players.current()) else {
            self.show_nothing();
            return;
        };
        let track = &player.track;
        let title = if track.title.is_empty() { player.short_name() } else { &track.title };
        let artists = track.artists.join(", ");
        let state_icon = if player.status == Status::Playing { ICON_PLAYING } else { ICON_NOT_PLAYING };
        self.line.show(title, &artists, state_icon);
        self.card.show(title, &artists, &track.album, state_icon);
        self.show_length(track.length_seconds);
        self.load_cover(cover_source::cover_url(track));
    }

    fn show_nothing(&self) {
        self.line.show(NOTHING_PLAYING, "", ICON_NOT_PLAYING);
        self.card.show(NOTHING_PLAYING, "", "", ICON_NOT_PLAYING);
        self.card.clear_position();
        self.show_length(0.0);
        self.load_cover(String::new());
    }

    fn show_length(&self, seconds: f64) {
        if self.card.show_length(seconds) {
            (self.on_resize)();
        }
    }

    fn load_cover(&self, url: String) {
        if url == *self.cover_url.borrow() {
            return;
        }
        *self.cover_url.borrow_mut() = url.clone();
        if url.is_empty() {
            self.show_cover(&url, None);
            return;
        }
        cover_source::load(&self.events, &url);
    }

    fn follow_position(self: &Rc<Self>) {
        self.ask_position();
        let me = Rc::downgrade(self);
        let source = glib::timeout_add_local(POSITION_POLL, move || {
            let Some(music) = me.upgrade() else { return glib::ControlFlow::Break };
            music.ask_position();
            glib::ControlFlow::Continue
        });
        *self.position_poll.borrow_mut() = Some(source);
    }

    fn ask_position(self: &Rc<Self>) {
        let Some(players) = &self.players else { return };
        if self.seek_hold.is_active() || !self.card.has_progress() {
            return;
        }
        let me = Rc::downgrade(self);
        players.ask_position(move |seconds| {
            let Some(music) = me.upgrade() else { return };
            if music.seek_hold.is_active() {
                return;
            }
            match seconds {
                Some(seconds) => music.card.show_position(seconds),
                None => music.card.clear_position(),
            }
        });
    }

    fn seek(&self, seconds: f64) {
        let Some(players) = &self.players else { return };
        if players.current().is_none() {
            return;
        }
        self.seek_hold.start();
        self.card.show_position_text(&clock(seconds));
        players.seek_to(seconds);
    }

    fn volume_moved(self: &Rc<Self>) {
        self.volume_hold.start();
        if self.volume_write_waiting.replace(true) {
            return;
        }
        let me = Rc::downgrade(self);
        glib::timeout_add_local_once(VOLUME_WRITE_EVERY, move || {
            let Some(music) = me.upgrade() else { return };
            music.volume_write_waiting.set(false);
            audio::set_volume(music.card.volume());
        });
    }
}

fn build_stack() -> gtk4::Stack {
    gtk4::Stack::builder()
        .transition_type(gtk4::StackTransitionType::Crossfade)
        .interpolate_size(true)
        .vhomogeneous(false)
        .hhomogeneous(false)
        .build()
}

fn start_players(me: &Weak<Music>) -> Option<Rc<Players>> {
    let bus = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>).ok()?;
    let me = me.clone();
    Some(Players::start(&bus, move |change| {
        if let Some(music) = me.upgrade() {
            music.on_players_change(change);
        }
    }))
}

fn controls(me: &Weak<Music>) -> Controls {
    Controls {
        play_pause: player_action(me, Players::play_pause),
        previous: player_action(me, Players::previous),
        next: player_action(me, Players::next),
        seek: {
            let me = me.clone();
            Box::new(move |seconds| {
                if let Some(music) = me.upgrade() {
                    music.seek(seconds);
                }
            })
        },
        volume_moved: {
            let me = me.clone();
            Box::new(move || {
                if let Some(music) = me.upgrade() {
                    music.volume_moved();
                }
            })
        },
    }
}

fn player_action(me: &Weak<Music>, action: fn(&Players)) -> Box<dyn Fn()> {
    let me = me.clone();
    Box::new(move || {
        let Some(music) = me.upgrade() else { return };
        if let Some(players) = &music.players {
            action(players);
        }
    })
}
