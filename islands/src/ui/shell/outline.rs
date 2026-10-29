//! An island's outline: a body hanging from the screen edge (y = 0), joined to it by long, smooth
//! shoulders, so it looks like the edge itself bending into the screen. Black at the edge, the
//! island's colour at the far end, with a thin red rim.

use gtk4::prelude::*;
use gtk4::{gdk, graphene, gsk};

const SHOULDER_FLATNESS: f32 = 0.5;
const SHOULDER_DROP: f32 = 0.5;
const CIRCLE_HANDLE: f32 = 0.552;
const RIM_WIDTH: f32 = 1.2;
const RIM_FADE: f32 = 10.0;
const RIM_ALPHA: f32 = 0.55;

#[derive(Clone, Copy, Debug)]
pub struct Outline {
    pub center: f32,
    pub width: f32,
    pub height: f32,
    pub shoulder: f32,
    pub radius: f32,
}

impl Outline {
    pub fn left(&self) -> f32 {
        self.center - self.width / 2.0 - self.shoulder
    }

    pub fn full_width(&self) -> f32 {
        self.width + 2.0 * self.shoulder
    }

    pub fn body(&self) -> graphene::Rect {
        graphene::Rect::new(self.center - self.width / 2.0, 0.0, self.width, self.height)
    }

    /// Drawn with y growing away from the edge; the caller flips it for an island on the bottom edge.
    pub fn paint(&self, snapshot: &gtk4::Snapshot, color: &gdk::RGBA) {
        if self.height < 0.5 {
            return;
        }
        let path = self.path();
        let bounds = graphene::Rect::new(self.left() - 2.0, -2.0, self.full_width() + 4.0, self.height + 4.0);

        snapshot.push_fill(&path, gsk::FillRule::Winding);
        let fill = [
            gsk::ColorStop::new(0.0, gdk::RGBA::BLACK),
            gsk::ColorStop::new(1.0, *color),
        ];
        snapshot.append_linear_gradient(&bounds, &point(0.0, 0.0), &point(0.0, self.height), &fill);
        snapshot.pop();

        snapshot.push_stroke(&path, &gsk::Stroke::new(RIM_WIDTH));
        let rim = [
            gsk::ColorStop::new(0.0, gdk::RGBA::new(1.0, 0.0, 0.0, 0.0)),
            gsk::ColorStop::new(1.0, gdk::RGBA::new(1.0, 0.0, 0.0, RIM_ALPHA)),
        ];
        snapshot.append_linear_gradient(&bounds, &point(0.0, 0.0), &point(0.0, RIM_FADE), &rim);
        snapshot.pop();
    }

    fn path(&self) -> gsk::Path {
        let height = self.height.max(0.0);
        let radius = self.radius.min(height / 2.0).min(self.width / 2.0);
        let shoulder = self.shoulder.max(0.0);
        let drop = (shoulder * 0.75).min(height - radius);
        let left = self.center - self.width / 2.0;
        let right = self.center + self.width / 2.0;
        let handle = radius * CIRCLE_HANDLE;

        let path = gsk::PathBuilder::new();
        path.move_to(left - shoulder, -1.0);
        path.line_to(left - shoulder, 0.0);
        path.cubic_to(left - shoulder * (1.0 - SHOULDER_FLATNESS), 0.0, left, drop * SHOULDER_DROP, left, drop);
        path.line_to(left, height - radius);
        path.cubic_to(left, height - radius + handle, left + radius - handle, height, left + radius, height);
        path.line_to(right - radius, height);
        path.cubic_to(right - radius + handle, height, right, height - radius + handle, right, height - radius);
        path.line_to(right, drop);
        path.cubic_to(right, drop * SHOULDER_DROP, right + shoulder * (1.0 - SHOULDER_FLATNESS), 0.0, right + shoulder, 0.0);
        path.line_to(right + shoulder, -1.0);
        path.close();
        path.to_path()
    }
}

fn point(x: f32, y: f32) -> graphene::Point {
    graphene::Point::new(x, y)
}
