//! iTerm2 inline images (iTerm2, WezTerm): a PNG in one OSC 1337 sequence.

use image::RgbaImage;
use std::io::Cursor;

use crate::base64;

/// An escape that draws `img` at the cursor over `cols` x `rows` cells.
pub(crate) fn encode(img: &RgbaImage, cols: u16, rows: u16) -> Vec<u8> {
    let mut png = Cursor::new(Vec::new());
    if img.write_to(&mut png, image::ImageFormat::Png).is_err() {
        return Vec::new();
    }
    let png = png.into_inner();
    let mut out = format!(
        "\x1b]1337;File=inline=1;width={cols};height={rows};preserveAspectRatio=0;size={}:",
        png.len()
    )
    .into_bytes();
    out.extend_from_slice(base64::encode(&png).as_bytes());
    out.push(0x07);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn the_sequence_names_its_size_and_carries_a_png() {
        let img = RgbaImage::from_pixel(3, 3, Rgba([200, 10, 10, 255]));
        let out = String::from_utf8(encode(&img, 6, 2)).unwrap();
        assert!(
            out.starts_with("\x1b]1337;File=inline=1;width=6;height=2;preserveAspectRatio=0;size=")
        );
        assert!(out.ends_with('\x07'));
        // "iVBORw0KGgo" is the base64 of a PNG signature.
        assert!(out.contains(":iVBORw0KGgo"), "{out}");
    }
}
