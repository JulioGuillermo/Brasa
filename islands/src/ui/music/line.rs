//! The one-line view: cover, title, artist and play state.

use gtk4::gdk;
use gtk4::prelude::*;

use super::cover::CoverView;
use super::label::label;

const COVER_SIDE: i32 = 28;

pub struct Line {
    root: gtk4::Box,
    cover: CoverView,
    title: gtk4::Label,
    artist: gtk4::Label,
    state: gtk4::Image,
}

impl Line {
    pub fn new() -> Line {
        let cover = CoverView::new(COVER_SIDE);
        let title = label(&["island-title"], 0.0, 40);
        title.set_hexpand(true);
        let artist = label(&["island-artist"], 0.0, 12);
        let state = gtk4::Image::builder().css_classes(["island-state"]).build();

        let texts = gtk4::Box::builder().spacing(8).hexpand(true).build();
        texts.append(&title);
        texts.append(&artist);
        let root = gtk4::Box::builder().spacing(10).css_classes(["island-peek"]).build();
        root.append(cover.widget());
        root.append(&texts);
        root.append(&state);
        Line { root, cover, title, artist, state }
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.root
    }

    pub fn show(&self, title: &str, artist: &str, state_icon: &str) {
        self.title.set_text(title);
        self.artist.set_text(artist);
        self.state.set_icon_name(Some(state_icon));
    }

    pub fn show_cover(&self, picture: Option<&gdk::Texture>) {
        self.cover.show(picture);
    }
}
