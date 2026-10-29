//! How big the launcher's parts are for each size setting.

use crate::config::Size;

pub struct Metrics {
    pub list_height: i32,
    pub columns: u32,
    pub grid_icon: i32,
    pub list_icon: i32,
}

pub fn metrics_for(size: Size) -> Metrics {
    match size {
        Size::Small => Metrics { list_height: 220, columns: 4, grid_icon: 40, list_icon: 24 },
        Size::Medium => Metrics { list_height: 340, columns: 5, grid_icon: 48, list_icon: 28 },
        Size::Large => Metrics { list_height: 500, columns: 7, grid_icon: 56, list_icon: 32 },
    }
}
