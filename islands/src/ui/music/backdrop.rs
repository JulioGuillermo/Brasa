//! The music card's row, with the cover painted behind it: from the island's left side it reaches
//! under the texts. Where the cover sits it is fully opaque; around it, it melts into the island: soft
//! fades at the left and right, shorter ones at the top and bottom.

use std::cell::{Cell, RefCell};

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, graphene, gsk};

const LEFT_FADE_PIXELS: f32 = 44.0;
const RIGHT_FADE_PIXELS: f32 = 120.0;
const TOP_BOTTOM_FADE_PIXELS: f32 = 22.0;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Backdrop {
        pub picture: RefCell<Option<gdk::Texture>>,
        pub bleed_left: Cell<f32>,
        pub cover_width: Cell<f32>,
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
                let area = Area {
                    bleed: self.bleed_left.get(),
                    cover_width: self.cover_width.get(),
                    width: widget.width() as f32,
                    height: widget.height() as f32,
                };
                paint_faded(snapshot, picture, &area);
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
    /// cover_width: the cover's place at the row's start, where it is fully opaque.
    pub fn new(spacing: i32, bleed_left: f32, cover_width: f32) -> Self {
        let backdrop: Backdrop = glib::Object::builder().property("spacing", spacing).build();
        backdrop.imp().bleed_left.set(bleed_left);
        backdrop.imp().cover_width.set(cover_width);
        backdrop
    }

    pub fn show(&self, picture: Option<&gdk::Texture>) {
        self.imp().picture.replace(picture.cloned());
        self.queue_draw();
    }
}

/// The row in its own coordinates, and how far past its left edge the cover is painted.
struct Area {
    bleed: f32,
    cover_width: f32,
    width: f32,
    height: f32,
}

/// A square of the picture from the island's side to the end of the right fade, centred on the row and
/// cropped to it, under two masks: one across and one down.
fn paint_faded(snapshot: &gtk4::Snapshot, picture: &gdk::Texture, area: &Area) {
    let left = -area.bleed;
    let right = area.cover_width + RIGHT_FADE_PIXELS;
    let side = right - left;
    let square = graphene::Rect::new(left, (area.height - side) / 2.0, side, side);
    let row = graphene::Rect::new(left, 0.0, area.width - left, area.height);

    snapshot.push_clip(&row);
    snapshot.push_mask(gsk::MaskMode::Alpha);
    let across = [(0.0, transparent()), (LEFT_FADE_PIXELS, opaque()), (side - RIGHT_FADE_PIXELS, opaque()), (side, transparent())];
    snapshot.append_linear_gradient(&row, &point(left, 0.0), &point(right, 0.0), &stops(&across, side));
    snapshot.pop();

    snapshot.push_mask(gsk::MaskMode::Alpha);
    let fade = TOP_BOTTOM_FADE_PIXELS.min(area.height / 2.0);
    let height = area.height;
    let down = [(0.0, transparent()), (fade, opaque()), (height - fade, opaque()), (height, transparent())];
    snapshot.append_linear_gradient(&row, &point(left, 0.0), &point(left, height), &stops(&down, height));
    snapshot.pop();

    snapshot.append_scaled_texture(picture, gsk::ScalingFilter::Linear, &square);
    snapshot.pop();
    snapshot.pop();
    snapshot.pop();
}

/// Stops given in pixels along a gradient of that length.
fn stops(at_pixels: &[(f32, gdk::RGBA)], length: f32) -> Vec<gsk::ColorStop> {
    at_pixels.iter().map(|(pixels, color)| gsk::ColorStop::new(pixels / length, *color)).collect()
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
