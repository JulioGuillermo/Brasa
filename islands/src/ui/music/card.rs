//! The full card: cover, texts, progress, play controls and volume.

use gtk4::gdk;
use gtk4::prelude::*;

use super::clock::clock;
use super::cover::CoverView;
use super::label::label;
use crate::ui::components::round_button::round_button;

const COVER_SIDE: i32 = 84;
const VOLUME_WIDTH: i32 = 120;

pub struct Controls {
    pub play_pause: Box<dyn Fn()>,
    pub previous: Box<dyn Fn()>,
    pub next: Box<dyn Fn()>,
    pub seek: Box<dyn Fn(f64)>,
    pub volume_moved: Box<dyn Fn()>,
}

pub struct Card {
    root: gtk4::Box,
    cover: CoverView,
    title: gtk4::Label,
    artist: gtk4::Label,
    album: gtk4::Label,
    position: gtk4::Label,
    length: gtk4::Label,
    progress: gtk4::Scale,
    progress_row: gtk4::Box,
    play: gtk4::Button,
    volume: gtk4::Scale,
}

impl Card {
    pub fn new(controls: Controls) -> Card {
        let cover = CoverView::new(COVER_SIDE);
        let title = label(&["island-title", "island-big"], 0.0, 28);
        let artist = label(&["island-artist"], 0.0, 32);
        let album = label(&["island-album"], 0.0, 32);
        let position = label(&["island-time"], 1.0, 0);
        let length = label(&["island-time"], 0.0, 0);
        let progress = gtk4::Scale::builder().hexpand(true).css_classes(["island-progress"]).draw_value(false).build();
        let volume = volume_scale();

        let Controls { play_pause, previous, next, seek, volume_moved } = controls;
        progress.connect_change_value(move |_, _, seconds| {
            seek(seconds);
            gtk4::glib::Propagation::Proceed
        });
        volume.connect_change_value(move |_, _, _| {
            volume_moved();
            gtk4::glib::Propagation::Proceed
        });
        let play = round_button("media-playback-start-symbolic", &["island-play"], play_pause);

        let progress_row = gtk4::Box::builder().spacing(8).build();
        progress_row.append(&position);
        progress_row.append(&progress);
        progress_row.append(&length);

        let root = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(6).css_classes(["island-full"]).build();
        root.append(&top_row(&cover, &title, &artist, &album));
        root.append(&progress_row);
        root.append(&controls_row(&play, previous, next, &volume));
        Card { root, cover, title, artist, album, position, length, progress, progress_row, play, volume }
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.root
    }

    pub fn show(&self, title: &str, artist: &str, album: &str, state_icon: &str) {
        self.title.set_text(title);
        self.artist.set_text(artist);
        self.album.set_text(album);
        self.play.set_icon_name(state_icon);
    }

    pub fn show_cover(&self, picture: Option<&gdk::Texture>) {
        self.cover.show(picture);
    }

    /// 0 hides the progress row. Returns whether the row appeared or disappeared.
    pub fn show_length(&self, seconds: f64) -> bool {
        let known = seconds > 0.0;
        self.progress.set_sensitive(known);
        self.progress.set_range(0.0, if known { seconds } else { 1.0 });
        let length_text = if known { clock(seconds) } else { String::new() };
        self.length.set_text(&length_text);
        if !known {
            self.progress.set_value(0.0);
        }
        let changed = self.progress_row.is_visible() != known;
        self.progress_row.set_visible(known);
        changed
    }

    pub fn has_progress(&self) -> bool {
        self.progress.is_sensitive()
    }

    pub fn show_position(&self, seconds: f64) {
        self.progress.set_value(seconds);
        self.position.set_text(&clock(seconds));
    }

    pub fn show_position_text(&self, text: &str) {
        self.position.set_text(text);
    }

    pub fn clear_position(&self) {
        self.position.set_text("");
    }

    pub fn volume(&self) -> f64 {
        self.volume.value()
    }

    pub fn set_volume(&self, level: f64) {
        self.volume.set_value(level);
    }
}

fn volume_scale() -> gtk4::Scale {
    let adjustment = gtk4::Adjustment::new(0.0, 0.0, 1.0, 0.05, 0.1, 0.0);
    gtk4::Scale::builder()
        .width_request(VOLUME_WIDTH)
        .css_classes(["island-volume"])
        .draw_value(false)
        .adjustment(&adjustment)
        .build()
}

fn top_row(cover: &CoverView, title: &gtk4::Label, artist: &gtk4::Label, album: &gtk4::Label) -> gtk4::Box {
    let texts = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(2).valign(gtk4::Align::Center).hexpand(true).build();
    texts.append(title);
    texts.append(artist);
    texts.append(album);
    let top = gtk4::Box::builder().spacing(14).build();
    top.append(cover.widget());
    top.append(&texts);
    top
}

fn controls_row(play: &gtk4::Button, previous: Box<dyn Fn()>, next: Box<dyn Fn()>, volume: &gtk4::Scale) -> gtk4::Box {
    let buttons = gtk4::Box::builder().spacing(6).build();
    buttons.append(&round_button("media-skip-backward-symbolic", &[], previous));
    buttons.append(play);
    buttons.append(&round_button("media-skip-forward-symbolic", &[], next));

    let sound = gtk4::Box::builder().spacing(4).halign(gtk4::Align::End).hexpand(true).build();
    sound.append(&gtk4::Image::builder().icon_name("audio-volume-high-symbolic").css_classes(["island-volume-icon"]).build());
    sound.append(volume);

    let controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    controls.append(&buttons);
    controls.append(&sound);
    controls
}
