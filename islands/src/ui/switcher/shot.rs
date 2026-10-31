//! A window's picture behind its card: as tall as the card, against its right side, melting into the
//! card from the picture's left edge to past its middle, whatever its shape.

use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, graphene, gsk};

/// How much of the shown picture, from its left edge, the fade takes.
const FADE_SHARE: f32 = 0.6;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Shot {
        pub picture: RefCell<Option<gdk::Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Shot {
        const NAME: &'static str = "BrasaWindowShot";
        type Type = super::Shot;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Shot {}

    impl WidgetImpl for Shot {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let Some(picture) = self.picture.borrow().clone() else { return };
            let widget = self.obj();
            paint_faded(snapshot, &picture, widget.width() as f32, widget.height() as f32);
        }
    }
}

glib::wrapper! {
    pub struct Shot(ObjectSubclass<imp::Shot>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for Shot {
    fn default() -> Self {
        let shot: Shot = glib::Object::new();
        shot.add_css_class("switcher-shot");
        shot
    }
}

impl Shot {
    pub fn show(&self, picture: &gdk::Texture) {
        self.imp().picture.replace(Some(picture.clone()));
        self.queue_draw();
    }
}

fn paint_faded(snapshot: &gtk4::Snapshot, picture: &gdk::Texture, width: f32, height: f32) {
    let picture_width = height * picture.width() as f32 / picture.height().max(1) as f32;
    let placed = graphene::Rect::new(width - picture_width, 0.0, picture_width, height);
    let shown_left = placed.x().max(0.0);
    let card = graphene::Rect::new(0.0, 0.0, width, height);
    let fade_end = shown_left + FADE_SHARE * (width - shown_left);

    snapshot.push_mask(gsk::MaskMode::Alpha);
    let stops = [
        gsk::ColorStop::new(0.0, gdk::RGBA::new(1.0, 1.0, 1.0, 0.0)),
        gsk::ColorStop::new(1.0, gdk::RGBA::WHITE),
    ];
    let from = graphene::Point::new(shown_left, 0.0);
    let to = graphene::Point::new(fade_end, 0.0);
    snapshot.append_linear_gradient(&card, &from, &to, &stops);
    snapshot.pop();
    snapshot.push_clip(&card);
    snapshot.append_scaled_texture(picture, gsk::ScalingFilter::Linear, &placed);
    snapshot.pop();
    snapshot.pop();
}
