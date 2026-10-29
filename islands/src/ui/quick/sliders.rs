//! Volume and, with a backlight, screen brightness. While the panel is open they follow changes made
//! elsewhere (the keys, another program), except right after you moved them.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::glib;
use gtk4::prelude::*;

use super::glyphs;
use crate::runtime::workers;
use crate::services::{audio, hardware};

const KEY_STEP: f64 = 0.05;
const USER_HOLD: Duration = Duration::from_secs(1);

pub struct Slider {
    pub root: gtk4::Box,
    pub scale: gtk4::Scale,
    read: fn() -> f64,
    moved_at: Rc<Cell<Option<Instant>>>,
}

impl Slider {
    fn new(icon: &str, read: fn() -> f64, write: fn(f64)) -> Slider {
        let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 1.0, 0.01);
        scale.set_hexpand(true);
        scale.set_draw_value(false);
        scale.adjustment().set_step_increment(KEY_STEP);
        scale.adjustment().set_page_increment(KEY_STEP * 2.0);

        let moved_at = Rc::new(Cell::new(None));
        let remember_move = Rc::clone(&moved_at);
        scale.connect_change_value(move |_, _, value| {
            remember_move.set(Some(Instant::now()));
            write(value.clamp(0.0, 1.0));
            glib::Propagation::Proceed
        });

        let root = gtk4::Box::builder().spacing(10).css_classes(["quick-slider"]).build();
        root.append(&gtk4::Label::builder().label(icon).css_classes(["quick-slider-icon"]).build());
        root.append(&scale);
        Slider { root, scale, read, moved_at }
    }

    /// Reads on a worker and moves the scale when the value arrives, unless you moved it meanwhile.
    pub fn refresh(&self) {
        let (sender, receiver) = async_channel::bounded(1);
        let read = self.read;
        workers::run(move || {
            let _ = sender.send_blocking(read());
        });
        let scale = self.scale.clone();
        let moved_at = Rc::clone(&self.moved_at);
        glib::spawn_future_local(async move {
            let Ok(value) = receiver.recv().await else { return };
            let moved_lately = moved_at.get().is_some_and(|at| at.elapsed() < USER_HOLD);
            if !moved_lately {
                scale.set_value(value);
            }
        });
    }
}

pub fn sliders(with_backlight: bool) -> Vec<Slider> {
    let mut found = vec![Slider::new(glyphs::VOLUME, audio::volume, audio::set_volume)];
    if with_backlight {
        found.push(Slider::new(glyphs::BRIGHTNESS, hardware::screen_brightness, hardware::set_screen_brightness));
    }
    found
}
