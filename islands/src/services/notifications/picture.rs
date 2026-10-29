//! A picture an app sends as raw pixels (the image-data hint of a notification).

use std::path::Path;

use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::prelude::*;
use gtk4::{gdk, glib};

const BITS_PER_SAMPLE: i32 = 8;

/// The hint's type: width, height, rowstride, has alpha, bits per sample, channels, pixels.
pub const IMAGE_DATA_TYPE: &str = "(iiibiiay)";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    width: i32,
    height: i32,
    rowstride: i32,
    has_alpha: bool,
    pixels: Vec<u8>,
}

impl Picture {
    /// None when the hint does not describe 8-bit RGB or RGBA pixels that fit in its bytes.
    pub fn from_hint(hint: &glib::Variant) -> Option<Picture> {
        let (width, height, rowstride, has_alpha, bits, channels, pixels) =
            hint.get::<(i32, i32, i32, bool, i32, i32, Vec<u8>)>()?;
        let channels_fit = channels == if has_alpha { 4 } else { 3 };
        if bits != BITS_PER_SAMPLE || !channels_fit || width <= 0 || height <= 0 {
            return None;
        }
        let row_bytes = width as usize * channels as usize;
        let rowstride_bytes = usize::try_from(rowstride).ok()?;
        let last_row_end = (height as usize - 1) * rowstride_bytes + row_bytes;
        if rowstride_bytes < row_bytes || pixels.len() < last_row_end {
            return None;
        }
        Some(Picture { width, height, rowstride, has_alpha, pixels })
    }

    pub fn to_texture(&self) -> gdk::Texture {
        let format = if self.has_alpha { gdk::MemoryFormat::R8g8b8a8 } else { gdk::MemoryFormat::R8g8b8 };
        let bytes = glib::Bytes::from(&self.pixels);
        gdk::MemoryTexture::new(self.width, self.height, format, &bytes, self.rowstride as usize).upcast()
    }

    /// A name that follows the pixels, so the same picture is written once.
    pub fn content_name(&self) -> String {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.pixels.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Needs no display, so it may run on a worker thread.
    pub fn save_png(&self, path: &Path) -> Result<(), glib::Error> {
        let bytes = glib::Bytes::from(&self.pixels);
        let pixbuf =
            Pixbuf::from_bytes(&bytes, Colorspace::Rgb, self.has_alpha, BITS_PER_SAMPLE, self.width, self.height, self.rowstride);
        pixbuf.savev(path, "png", &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint(width: i32, height: i32, rowstride: i32, has_alpha: bool, bits: i32, channels: i32, bytes: usize) -> glib::Variant {
        (width, height, rowstride, has_alpha, bits, channels, vec![0u8; bytes]).to_variant()
    }

    #[test]
    fn reads_a_well_formed_rgba_picture() {
        assert!(Picture::from_hint(&hint(2, 2, 8, true, 8, 4, 16)).is_some());
    }

    #[test]
    fn a_last_row_without_padding_is_enough() {
        assert!(Picture::from_hint(&hint(2, 2, 8, false, 8, 3, 14)).is_some());
    }

    #[test]
    fn rejects_pixels_that_are_too_few() {
        assert!(Picture::from_hint(&hint(2, 2, 8, true, 8, 4, 15)).is_none());
    }

    #[test]
    fn rejects_other_bit_depths_and_channel_counts() {
        assert!(Picture::from_hint(&hint(1, 1, 4, true, 16, 4, 8)).is_none());
        assert!(Picture::from_hint(&hint(1, 1, 4, true, 8, 3, 4)).is_none());
    }

    #[test]
    fn the_same_pixels_get_the_same_name() {
        let first = Picture::from_hint(&hint(1, 1, 4, true, 8, 4, 4)).unwrap();
        let second = first.clone();
        assert_eq!(first.content_name(), second.content_name());
    }
}
