//! The rounded cover: the picture, or a music icon while there is none.

use gtk4::gdk;
use gtk4::prelude::*;

const PLACEHOLDER_ICON: &str = "audio-x-generic-symbolic";

pub struct CoverView {
    frame: gtk4::Box,
    image: gtk4::Image,
    side: i32,
}

impl CoverView {
    pub fn new(side: i32) -> CoverView {
        let frame = gtk4::Box::builder()
            .css_classes(["island-cover"])
            .overflow(gtk4::Overflow::Hidden)
            .width_request(side)
            .height_request(side)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Center)
            .hexpand(false)
            .vexpand(false)
            .build();
        let image = gtk4::Image::builder().hexpand(true).halign(gtk4::Align::Center).valign(gtk4::Align::Center).build();
        frame.append(&image);
        let cover = CoverView { frame, image, side };
        cover.show(None);
        cover
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.frame
    }

    pub fn show(&self, picture: Option<&gdk::Texture>) {
        match picture {
            Some(texture) => {
                self.image.set_paintable(Some(texture));
                self.image.set_pixel_size(self.side);
            }
            None => {
                self.image.set_icon_name(Some(PLACEHOLDER_ICON));
                self.image.set_pixel_size(self.side / 2);
            }
        }
    }
}
