//! A captured window made small: each pixel of the result is the average of the block it covers.

pub const BYTES_PER_PIXEL: usize = 4;

pub struct Image {
    pub width: usize,
    pub height: usize,
    /// Rows of `width` pixels, 4 bytes each, with no padding between them.
    pub pixels: Vec<u8>,
}

/// `pixels` holds rows `stride` bytes apart. The result is at most `max_height` tall, its width
/// keeping the proportions; an image already that small is only copied.
pub fn shrink(pixels: &[u8], width: usize, height: usize, stride: usize, max_height: usize) -> Image {
    let new_height = height.min(max_height).max(1);
    let new_width = (width * new_height / height.max(1)).max(1);
    let mut shrunk = Vec::with_capacity(new_width * new_height * BYTES_PER_PIXEL);
    for row in 0..new_height {
        let (top, bottom) = block(row, new_height, height);
        for column in 0..new_width {
            let (left, right) = block(column, new_width, width);
            shrunk.extend(average(pixels, stride, left..right, top..bottom));
        }
    }
    Image { width: new_width, height: new_height, pixels: shrunk }
}

/// The source pixels a result pixel covers along one side: never empty.
fn block(index: usize, new_length: usize, length: usize) -> (usize, usize) {
    let start = index * length / new_length;
    let end = ((index + 1) * length / new_length).max(start + 1).min(length);
    (start, end)
}

fn average(pixels: &[u8], stride: usize, columns: std::ops::Range<usize>, rows: std::ops::Range<usize>) -> [u8; 4] {
    let mut sums = [0u32; BYTES_PER_PIXEL];
    let count = (columns.len() * rows.len()) as u32;
    for row in rows {
        let row_start = row * stride;
        for column in columns.clone() {
            let at = row_start + column * BYTES_PER_PIXEL;
            for (channel, sum) in sums.iter_mut().enumerate() {
                *sum += u32::from(pixels[at + channel]);
            }
        }
    }
    sums.map(|sum| (sum / count) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_an_image_by_averaging_blocks() {
        #[rustfmt::skip]
        let pixels = [
            0, 0, 0, 0,   100, 100, 100, 100,   9, 9, 9, 9,
            200, 200, 200, 200,   100, 100, 100, 100,   9, 9, 9, 9,
        ];
        let image = shrink(&pixels, 2, 2, 12, 1);
        assert_eq!((image.width, image.height), (1, 1));
        assert_eq!(image.pixels, [100, 100, 100, 100]);
    }

    #[test]
    fn keeps_the_proportions() {
        let pixels = vec![0u8; 400 * 100 * 4];
        let image = shrink(&pixels, 400, 100, 1600, 50);
        assert_eq!((image.width, image.height), (200, 50));
        assert_eq!(image.pixels.len(), 200 * 50 * 4);
    }

    #[test]
    fn a_small_image_keeps_its_size() {
        let pixels = vec![7u8; 3 * 2 * 4];
        let image = shrink(&pixels, 3, 2, 12, 300);
        assert_eq!((image.width, image.height), (3, 2));
        assert_eq!(image.pixels, pixels);
    }
}
