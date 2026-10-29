//! The panel's outline: a body sitting in a bottom corner of the screen, joined to the side edge and to
//! the bottom edge by long, smooth shoulders, like the island but on two edges. Drawn in the corner's own
//! frame: x grows away from the side edge, y grows up from the bottom edge. Black along both edges, the
//! panel's colour towards the free corner, a thin red rim that fades out along the edges.

use gtk4::prelude::*;
use gtk4::{gdk, graphene, gsk};

/// The window reaches this far past the bottom and side edges, so its own border, which a fractional
/// scale can leave half drawn, stays off screen.
pub const OVERHANG: f64 = 2.0;

const SHOULDER_FLATNESS: f32 = 0.5;
const SHOULDER_DROP: f32 = 0.5;
const CIRCLE_HANDLE: f32 = 0.552;
const EDGE_SHADE: f32 = 0.6;
const RIM_WIDTH: f32 = 1.2;
const RIM_FADE: f32 = 10.0;
const RIM_ALPHA: f32 = 0.55;

#[derive(Clone, Copy, Debug)]
pub struct Corner {
    pub width: f32,
    pub height: f32,
    pub shoulder: f32,
    pub radius: f32,
}

impl Corner {
    pub fn closed() -> Corner {
        Corner { width: 0.0, height: 0.0, shoulder: 0.0, radius: 0.0 }
    }

    /// Whether a point of the corner's frame is on the body.
    pub fn holds(&self, x: f64, y: f64) -> bool {
        (0.0..=self.width as f64).contains(&x) && (0.0..=self.height as f64).contains(&y)
    }

    pub fn paint(&self, snapshot: &gtk4::Snapshot, color: &gdk::RGBA) {
        if self.width < 0.5 || self.height < 0.5 {
            return;
        }
        let path = self.path();
        let bounds = graphene::Rect::new(-2.0, -2.0, self.width + self.shoulder + 4.0, self.height + self.shoulder + 4.0);
        self.paint_fill(snapshot, &path, &bounds, color);
        paint_rim(snapshot, &path, &bounds);
    }

    fn paint_fill(&self, snapshot: &gtk4::Snapshot, path: &gsk::Path, bounds: &graphene::Rect, color: &gdk::RGBA) {
        snapshot.push_fill(path, gsk::FillRule::Winding);
        snapshot.append_color(color, bounds);
        let from_side = fade_out(1.0);
        snapshot.append_linear_gradient(bounds, &point(0.0, 0.0), &point(self.width * EDGE_SHADE, 0.0), &from_side);
        snapshot.append_linear_gradient(bounds, &point(0.0, 0.0), &point(0.0, self.height * EDGE_SHADE), &from_side);
        snapshot.pop();
    }

    fn path(&self) -> gsk::Path {
        let width = self.width.max(0.0);
        let height = self.height.max(0.0);
        let radius = self.radius.min(width / 2.0).min(height / 2.0);
        let shoulder = self.shoulder.max(0.0);
        let side_drop = (shoulder * 0.75).min(width - radius);
        let bottom_drop = (shoulder * 0.75).min(height - radius);
        let handle = radius * CIRCLE_HANDLE;
        let flat = 1.0 - SHOULDER_FLATNESS;

        let path = gsk::PathBuilder::new();
        path.move_to(-1.0, height + shoulder);
        path.line_to(0.0, height + shoulder);
        path.cubic_to(0.0, height + shoulder * flat, side_drop * SHOULDER_DROP, height, side_drop, height);
        path.line_to(width - radius, height);
        path.cubic_to(width - radius + handle, height, width, height - radius + handle, width, height - radius);
        path.line_to(width, bottom_drop);
        path.cubic_to(width, bottom_drop * SHOULDER_DROP, width + shoulder * flat, 0.0, width + shoulder, 0.0);
        path.line_to(width + shoulder, -1.0);
        path.line_to(-1.0, -1.0);
        path.close();
        path.to_path()
    }
}

/// The red line along the outline, invisible at the two edges and full strength RIM_FADE away from them.
fn paint_rim(snapshot: &gtk4::Snapshot, path: &gsk::Path, bounds: &graphene::Rect) {
    let fade_in = fade_out(0.0);
    snapshot.push_mask(gsk::MaskMode::Alpha);
    snapshot.append_linear_gradient(bounds, &point(0.0, 0.0), &point(RIM_FADE, 0.0), &fade_in);
    snapshot.pop();
    snapshot.push_mask(gsk::MaskMode::Alpha);
    snapshot.append_linear_gradient(bounds, &point(0.0, 0.0), &point(0.0, RIM_FADE), &fade_in);
    snapshot.pop();
    snapshot.push_stroke(path, &gsk::Stroke::new(RIM_WIDTH));
    snapshot.append_color(&gdk::RGBA::new(1.0, 0.0, 0.0, RIM_ALPHA), bounds);
    snapshot.pop();
    snapshot.pop();
    snapshot.pop();
}

/// Black at offset 0 with `alpha_at_start`, to black with the opposite alpha at offset 1.
fn fade_out(alpha_at_start: f32) -> [gsk::ColorStop; 2] {
    [
        gsk::ColorStop::new(0.0, gdk::RGBA::new(0.0, 0.0, 0.0, alpha_at_start)),
        gsk::ColorStop::new(1.0, gdk::RGBA::new(0.0, 0.0, 0.0, 1.0 - alpha_at_start)),
    ]
}

fn point(x: f32, y: f32) -> graphene::Point {
    graphene::Point::new(x, y)
}

/// Where the screen's corner is in the window, and which side it is on.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub x: f64,
    pub y: f64,
    pub on_right: bool,
}

impl Placement {
    pub fn in_window(window_width: f64, window_height: f64, on_right: bool) -> Placement {
        let x = if on_right { window_width - OVERHANG } else { OVERHANG };
        Placement { x, y: window_height - OVERHANG, on_right }
    }

    /// A window point in the corner's frame.
    pub fn to_corner(&self, x: f64, y: f64) -> (f64, f64) {
        let corner_x = if self.on_right { self.x - x } else { x - self.x };
        (corner_x, self.y - y)
    }

    pub fn enter_corner_frame(&self, snapshot: &gtk4::Snapshot) {
        snapshot.translate(&graphene::Point::new(self.x as f32, self.y as f32));
        snapshot.scale(if self.on_right { -1.0 } else { 1.0 }, -1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_point_maps_into_the_corner_frame_on_either_side() {
        let right = Placement::in_window(1000.0, 800.0, true);
        assert_eq!(right.to_corner(900.0, 700.0), (98.0, 98.0));
        let left = Placement::in_window(1000.0, 800.0, false);
        assert_eq!(left.to_corner(100.0, 700.0), (98.0, 98.0));
    }

    #[test]
    fn the_body_holds_its_own_area_only() {
        let corner = Corner { width: 100.0, height: 50.0, shoulder: 10.0, radius: 5.0 };
        assert!(corner.holds(100.0, 50.0));
        assert!(!corner.holds(101.0, 10.0));
        assert!(!corner.holds(-1.0, 10.0));
    }
}
