//! The full card: a big cover melting into the island and reaching under the texts, and beside it the
//! texts, the progress (times above a bar as wide as the texts) and a row of small controls.

use gtk4::gdk;
use gtk4::prelude::*;

use super::clock::clock;
use super::backdrop::Backdrop;
use super::label::label;
use crate::ui::components::round_button::round_button;
use crate::ui::shell::shapes::Shape;

const COVER_SIDE: i32 = 128;
const PLACEHOLDER_ICON: &str = "audio-x-generic-symbolic";
const VOLUME_WIDTH: i32 = 90;

pub struct Controls {
    pub play_pause: Box<dyn Fn()>,
    pub previous: Box<dyn Fn()>,
    pub next: Box<dyn Fn()>,
    pub seek: Box<dyn Fn(f64)>,
    pub volume_moved: Box<dyn Fn()>,
}

pub struct Card {
    root: Backdrop,
    cover_slot: gtk4::Box,
    placeholder: gtk4::Image,
    title: gtk4::Label,
    artist: gtk4::Label,
    album: gtk4::Label,
    position: gtk4::Label,
    length: gtk4::Label,
    progress: gtk4::Scale,
    times: gtk4::Box,
    play: gtk4::Button,
    volume: gtk4::Scale,
    volume_icon: gtk4::Image,
}

impl Card {
    pub fn new(controls: Controls) -> Card {
        let title = label(&["island-title", "island-big"], 0.0, 28);
        let artist = label(&["island-artist"], 0.0, 32);
        let album = label(&["island-album"], 0.0, 32);
        let position = label(&["island-time"], 0.0, 0);
        let length = label(&["island-time"], 1.0, 0);
        let progress = gtk4::Scale::builder().hexpand(true).css_classes(["island-progress"]).draw_value(false).build();
        let volume = volume_scale();
        let volume_icon = gtk4::Image::builder().icon_name(volume_icon_name(0.0)).css_classes(["island-volume-icon"]).build();

        let Controls { play_pause, previous, next, seek, volume_moved } = controls;
        progress.connect_change_value(move |_, _, seconds| {
            seek(seconds);
            gtk4::glib::Propagation::Proceed
        });
        let icon_for_moves = volume_icon.clone();
        volume.connect_change_value(move |_, _, level| {
            icon_for_moves.set_icon_name(Some(volume_icon_name(level)));
            volume_moved();
            gtk4::glib::Propagation::Proceed
        });
        let play = round_button("media-playback-start-symbolic", &["island-play"], play_pause);

        let times = times_row(&position, &length);
        let progress_block = progress_block(&times, &progress);
        let controls = controls_row(&play, previous, next, &volume_icon, &volume);
        let column = beside_cover(&[&title, &artist, &album], &progress_block, &controls);
        let placeholder = gtk4::Image::builder().icon_name(PLACEHOLDER_ICON).pixel_size(COVER_SIDE / 2).hexpand(true).build();
        let cover_slot = gtk4::Box::builder().width_request(COVER_SIDE).height_request(COVER_SIDE).valign(gtk4::Align::Center).build();
        cover_slot.append(&placeholder);
        let root = Backdrop::new(16, Shape::Card.dimensions().padding.1 as f32);
        root.add_css_class("island-full");
        root.append(&cover_slot);
        root.append(&column);
        let card = Card { root, cover_slot, placeholder, title, artist, album, position, length, progress, times, play, volume, volume_icon };
        card.show_cover(None);
        card
    }

    pub fn widget(&self) -> &Backdrop {
        &self.root
    }

    pub fn show(&self, title: &str, artist: &str, album: &str, state_icon: &str) {
        self.title.set_text(title);
        self.artist.set_text(artist);
        self.album.set_text(album);
        self.artist.set_visible(!artist.is_empty());
        self.album.set_visible(!album.is_empty());
        self.play.set_icon_name(state_icon);
    }

    /// Without a picture, a music icon on a plain tile holds the cover's place.
    pub fn show_cover(&self, picture: Option<&gdk::Texture>) {
        self.root.show(picture);
        self.placeholder.set_visible(picture.is_none());
        if picture.is_some() {
            self.cover_slot.remove_css_class("island-cover");
        } else {
            self.cover_slot.add_css_class("island-cover");
        }
    }

    /// 0 hides the times and greys the bar, which stays to keep the controls apart from the texts.
    /// Returns whether the times appeared or disappeared.
    pub fn show_length(&self, seconds: f64) -> bool {
        let known = seconds > 0.0;
        self.progress.set_sensitive(known);
        self.progress.set_range(0.0, if known { seconds } else { 1.0 });
        let length_text = if known { clock(seconds) } else { String::new() };
        self.length.set_text(&length_text);
        if !known {
            self.progress.set_value(0.0);
        }
        let changed = self.times.is_visible() != known;
        self.times.set_visible(known);
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
        self.volume_icon.set_icon_name(Some(volume_icon_name(level)));
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

fn volume_icon_name(level: f64) -> &'static str {
    match level {
        level if level <= 0.0 => "audio-volume-muted-symbolic",
        level if level < 0.34 => "audio-volume-low-symbolic",
        level if level < 0.67 => "audio-volume-medium-symbolic",
        _ => "audio-volume-high-symbolic",
    }
}

/// The elapsed on the left and the length on the right.
fn times_row(position: &gtk4::Label, length: &gtk4::Label) -> gtk4::Box {
    position.set_hexpand(true);
    let times = gtk4::Box::builder().css_classes(["island-times"]).build();
    times.append(position);
    times.append(length);
    times
}

fn progress_block(times: &gtk4::Box, progress: &gtk4::Scale) -> gtk4::Box {
    let block = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).margin_top(6).build();
    block.append(times);
    block.append(progress);
    block
}

/// The texts at the top, the progress, and the controls at the foot, as tall as the cover.
fn beside_cover(texts: &[&gtk4::Label], progress_block: &gtk4::Box, controls: &gtk4::Box) -> gtk4::Box {
    let column = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(2).valign(gtk4::Align::Center).hexpand(true).build();
    for text in texts {
        column.append(*text);
    }
    column.append(progress_block);
    column.append(controls);
    column
}

/// Previous, play and next on the left, the volume on the right.
fn controls_row(
    play: &gtk4::Button,
    previous: Box<dyn Fn()>,
    next: Box<dyn Fn()>,
    volume_icon: &gtk4::Image,
    volume: &gtk4::Scale,
) -> gtk4::Box {
    let buttons = gtk4::Box::builder().spacing(2).valign(gtk4::Align::Center).build();
    buttons.append(&round_button("media-skip-backward-symbolic", &[], previous));
    buttons.append(play);
    buttons.append(&round_button("media-skip-forward-symbolic", &[], next));

    let sound = gtk4::Box::builder().spacing(4).halign(gtk4::Align::End).valign(gtk4::Align::Center).hexpand(true).build();
    sound.append(volume_icon);
    sound.append(volume);

    let row = gtk4::Box::builder().css_classes(["island-controls"]).build();
    row.append(&buttons);
    row.append(&sound);
    row
}
