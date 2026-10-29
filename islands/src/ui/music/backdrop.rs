//! The music card's row, with the cover painted behind it: from the island's left side it reaches
//! under the texts, melting into the island, a long fade to the right and short ones elsewhere.

use std::cell::{Cell, RefCell};

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, graphene, gsk};

/// How much wider than the row is tall the cover is painted, so it reaches under the texts.
const REACH: f32 = 1.45;
/// Parts of the cover's width: the fade in from the left edge, and where the long fade to the right starts.
const LEFT_FADE: f32 = 0.04;
const RIGHT_FADE_START: f32 = 0.42;
const TOP_BOTTOM_FADE_PIXELS: f32 = 16.0;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Backdrop {
        pub picture: RefCell<Option<gdk::Texture>>,
        pub bleed_left: Cell<f32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Backdrop {
        const NAME: &'static str = "BrasaMusicBackdrop";
        type Type = super::Backdrop;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for Backdrop {}

    impl WidgetImpl for Backdrop {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            if let Some(picture) = self.picture.borrow().as_ref() {
                let widget = self.obj();
                let bleed = self.bleed_left.get();
                paint_faded(snapshot, picture, bleed, widget.width() as f32, widget.height() as f32);
            }
            self.parent_snapshot(snapshot);
        }
    }

    impl BoxImpl for Backdrop {}
}

glib::wrapper! {
    pub struct Backdrop(ObjectSubclass<imp::Backdrop>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Backdrop {
    /// bleed_left: how far past its left edge the cover is painted, to reach the island's side.
    pub fn new(spacing: i32, bleed_left: f32) -> Self {
        let backdrop: Backdrop = glib::Object::builder().property("spacing", spacing).build();
        backdrop.imp().bleed_left.set(bleed_left);
        backdrop
    }

    pub fn show(&self, picture: Option<&gdk::Texture>) {
        self.imp().picture.replace(picture.cloned());
        self.queue_draw();
    }
}

/// A square of the picture as wide as REACH times the row's height, cropped to the row, under two
/// masks: one across (fading in at the left, out to the right) and one down (fading at top and bottom).
fn paint_faded(snapshot: &gtk4::Snapshot, picture: &gdk::Texture, bleed: f32, width: f32, height: f32) {
    let side = height * REACH;
    let square = graphene::Rect::new(-bleed, (height - side) / 2.0, side, side);
    let row = graphene::Rect::new(-bleed, 0.0, width + bleed, height);

    snapshot.push_clip(&row);
    snapshot.push_mask(gsk::MaskMode::Alpha);
    snapshot.append_linear_gradient(&row, &point(-bleed, 0.0), &point(side - bleed, 0.0), &across_stops());
    snapshot.pop();

    snapshot.push_mask(gsk::MaskMode::Alpha);
    snapshot.append_linear_gradient(&row, &point(-bleed, 0.0), &point(-bleed, height), &down_stops(height));
    snapshot.pop();

    snapshot.append_scaled_texture(picture, gsk::ScalingFilter::Linear, &square);
    snapshot.pop();
    snapshot.pop();
    snapshot.pop();
}

fn across_stops() -> [gsk::ColorStop; 4] {
    [
        gsk::ColorStop::new(0.0, transparent()),
        gsk::ColorStop::new(LEFT_FADE, opaque()),
        gsk::ColorStop::new(RIGHT_FADE_START, opaque()),
        gsk::ColorStop::new(1.0, transparent()),
    ]
}

fn down_stops(height: f32) -> [gsk::ColorStop; 4] {
    let fade = (TOP_BOTTOM_FADE_PIXELS / height).min(0.5);
    [
        gsk::ColorStop::new(0.0, transparent()),
        gsk::ColorStop::new(fade, opaque()),
        gsk::ColorStop::new(1.0 - fade, opaque()),
        gsk::ColorStop::new(1.0, transparent()),
    ]
}

fn opaque() -> gdk::RGBA {
    gdk::RGBA::WHITE
}

fn transparent() -> gdk::RGBA {
    gdk::RGBA::new(1.0, 1.0, 1.0, 0.0)
}

fn point(x: f32, y: f32) -> graphene::Point {
    graphene::Point::new(x, y)
}
