//! The kitty graphics protocol (kitty, Ghostty, WezTerm): raw RGBA sent in
//! base64 chunks, drawn at the cursor and scaled to a box of cells.

use image::RgbaImage;

use crate::base64;

const CHUNK: usize = 4096;

/// Transmits and places `img` at the cursor, `cols` x `rows` cells big,
/// under image id `id`. Placing the same id again replaces the picture.
pub(crate) fn encode(img: &RgbaImage, cols: u16, rows: u16, id: u32) -> Vec<u8> {
    let payload = base64::encode(img.as_raw());
    let mut out = Vec::with_capacity(payload.len() + 128);
    let chunks: Vec<&[u8]> = payload.as_bytes().chunks(CHUNK).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let more = u8::from(i + 1 < chunks.len());
        out.extend_from_slice(b"\x1b_G");
        if i == 0 {
            out.extend_from_slice(
                format!(
                    "a=T,f=32,s={},v={},i={id},c={cols},r={rows},C=1,q=2,m={more}",
                    img.width(),
                    img.height()
                )
                .as_bytes(),
            );
        } else {
            out.extend_from_slice(format!("m={more}").as_bytes());
        }
        out.push(b';');
        out.extend_from_slice(chunk);
        out.extend_from_slice(b"\x1b\\");
    }
    out
}

/// Removes the picture placed under `id`.
pub(crate) fn erase(id: u32) -> Vec<u8> {
    format!("\x1b_Ga=d,d=I,i={id},q=2\x1b\\").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn a_small_image_is_one_chunk_with_its_geometry() {
        let img = RgbaImage::from_pixel(2, 1, Rgba([1, 2, 3, 255]));
        let out = String::from_utf8(encode(&img, 4, 3, 9)).unwrap();
        assert!(out.starts_with("\x1b_Ga=T,f=32,s=2,v=1,i=9,c=4,r=3,C=1,q=2,m=0;"));
        assert!(out.ends_with("\x1b\\"));
        assert_eq!(out.matches("\x1b_G").count(), 1);
    }

    #[test]
    fn a_large_image_is_split_into_chunks_that_say_more_follows() {
        let img = RgbaImage::from_pixel(64, 64, Rgba([9, 9, 9, 255]));
        let out = String::from_utf8(encode(&img, 8, 4, 1)).unwrap();
        let chunks = out.matches("\x1b_G").count();
        assert!(chunks > 1);
        assert_eq!(out.matches("m=1").count(), chunks - 1);
        assert_eq!(out.matches("m=0").count(), 1);
        // Only the first chunk carries the geometry.
        assert_eq!(out.matches("f=32").count(), 1);
    }

    #[test]
    fn erasing_names_the_image() {
        assert_eq!(erase(5), b"\x1b_Ga=d,d=I,i=5,q=2\x1b\\");
    }
}
