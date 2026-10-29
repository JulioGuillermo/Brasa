//! The sizes an island takes for what it shows.

use crate::config::Size;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Hidden,
    /// One line: the peek, or the music beside notifications.
    Line,
    /// Music and notifications, open.
    Card,
    Launcher(Size),
}

pub struct Dimensions {
    pub body_width: f64,
    pub max_height: f64,
    pub shoulder: f64,
    pub radius: f64,
    /// From the edge to the content, and from the sides to the content.
    pub padding: (f64, f64),
}

impl Shape {
    pub fn dimensions(self) -> Dimensions {
        let (body_width, max_height, shoulder, radius, padding) = match self {
            Shape::Hidden => (150.0, 0.0, 0.0, 0.0, (16.0, 24.0)),
            Shape::Line => (352.0, 44.0, 64.0, 22.0, (8.0, 18.0)),
            Shape::Card => (470.0, 620.0, 120.0, 38.0, (16.0, 24.0)),
            Shape::Launcher(Size::Small) => (400.0, 320.0, 90.0, 32.0, (14.0, 18.0)),
            Shape::Launcher(Size::Medium) => (560.0, 460.0, 110.0, 38.0, (16.0, 22.0)),
            Shape::Launcher(Size::Large) => (780.0, 640.0, 130.0, 42.0, (18.0, 26.0)),
        };
        Dimensions { body_width, max_height, shoulder, radius, padding }
    }

    pub fn content_width(self) -> f64 {
        let dimensions = self.dimensions();
        dimensions.body_width - 2.0 * dimensions.padding.1
    }

    /// Its whole width, shoulders included.
    pub fn full_width(self) -> f64 {
        let dimensions = self.dimensions();
        dimensions.body_width + 2.0 * dimensions.shoulder
    }
}
