//! Small pictures of open windows, for the window switcher's cards. Blocking: call it from a worker.

mod capture;
mod shrink;

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use wayland_client::protocol::wl_shm::Format;

use capture::Copied;

/// A full size copy of a big window takes tens of megabytes: so only a few are held at a time.
const WINDOWS_AT_A_TIME: usize = 4;

/// One picture per window that could be copied, each at most `max_height` pixels tall, given to
/// `on_shot` with the window's capture id, in the order asked for.
pub fn take(capture_ids: &[String], max_height: usize, mut on_shot: impl FnMut(String, gdk::Texture)) {
    for some in capture_ids.chunks(WINDOWS_AT_A_TIME) {
        for copied in capture::copy_windows(some) {
            if let Some(picture) = small_picture(&copied, max_height) {
                on_shot(copied.capture_id, picture);
            }
        }
    }
}

fn small_picture(copied: &Copied, max_height: usize) -> Option<gdk::Texture> {
    let format = memory_format(copied.format)?;
    let image = shrink::shrink(&copied.pixels, copied.width, copied.height, copied.stride, max_height);
    let stride = image.width * shrink::BYTES_PER_PIXEL;
    let bytes = glib::Bytes::from_owned(image.pixels);
    let texture = gdk::MemoryTexture::new(image.width as i32, image.height as i32, format, &bytes, stride);
    Some(texture.upcast())
}

/// wl_shm formats name a 32-bit pixel, little-endian in memory: Argb8888 is B, G, R, A byte by byte.
fn memory_format(format: Format) -> Option<gdk::MemoryFormat> {
    let memory_format = match format {
        Format::Argb8888 => gdk::MemoryFormat::B8g8r8a8Premultiplied,
        Format::Xrgb8888 => gdk::MemoryFormat::B8g8r8x8,
        Format::Abgr8888 => gdk::MemoryFormat::R8g8b8a8Premultiplied,
        Format::Xbgr8888 => gdk::MemoryFormat::R8g8b8x8,
        _ => return None,
    };
    Some(memory_format)
}

