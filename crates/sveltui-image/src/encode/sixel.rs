//! Sixel output (foot, xterm, mlterm and others): the picture is cut into
//! bands six pixels tall, one pass per color, using a fixed 6x6x6 palette.

use image::RgbaImage;
use std::fmt::Write;

const LEVELS: u32 = 6;

fn index(r: u8, g: u8, b: u8) -> usize {
    let level = |v: u8| (u32::from(v) * (LEVELS - 1) + 127) / 255;
    (level(r) * LEVELS * LEVELS + level(g) * LEVELS + level(b)) as usize
}

/// The palette entry's color in sixel's 0-100 percent scale.
fn percent(index: usize) -> (u32, u32, u32) {
    let i = index as u32;
    let scale = |l: u32| l * 100 / (LEVELS - 1);
    (
        scale(i / (LEVELS * LEVELS)),
        scale(i / LEVELS % LEVELS),
        scale(i % LEVELS),
    )
}

/// A sixel image of `img`, drawn at the cursor at 1:1 pixel aspect.
pub(crate) fn encode(img: &RgbaImage) -> Vec<u8> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut indices = vec![0u8; w * h];
    let mut used = [false; 216];
    for (i, px) in img.pixels().enumerate() {
        let c = index(px[0], px[1], px[2]);
        used[c] = true;
        indices[i] = c as u8;
    }
    let mut out = String::new();
    let _ = write!(out, "\x1bP0;0;0q\"1;1;{w};{h}");
    for (c, _) in used.iter().enumerate().filter(|(_, used)| **used) {
        let (r, g, b) = percent(c);
        let _ = write!(out, "#{c};2;{r};{g};{b}");
    }
    for band in (0..h).step_by(6) {
        let rows = (h - band).min(6);
        let mut colors = [false; 216];
        for y in 0..rows {
            for x in 0..w {
                colors[indices[(band + y) * w + x] as usize] = true;
            }
        }
        let mut first = true;
        for (c, _) in colors.iter().enumerate().filter(|(_, present)| **present) {
            if !first {
                out.push('$');
            }
            first = false;
            let _ = write!(out, "#{c}");
            let mut run: Option<(char, usize)> = None;
            for x in 0..w {
                let mut bits = 0u8;
                for y in 0..rows {
                    if indices[(band + y) * w + x] as usize == c {
                        bits |= 1 << y;
                    }
                }
                let ch = (63 + bits) as char;
                run = match run {
                    Some((prev, n)) if prev == ch => Some((prev, n + 1)),
                    Some((prev, n)) => {
                        flush(&mut out, prev, n);
                        Some((ch, 1))
                    }
                    None => Some((ch, 1)),
                };
            }
            if let Some((ch, n)) = run {
                flush(&mut out, ch, n);
            }
        }
        out.push('-');
    }
    out.push_str("\x1b\\");
    out.into_bytes()
}

fn flush(out: &mut String, ch: char, n: usize) {
    if n > 3 {
        let _ = write!(out, "!{n}{ch}");
    } else {
        for _ in 0..n {
            out.push(ch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn a_solid_picture_is_one_color_and_one_run_per_band() {
        let img = RgbaImage::from_pixel(8, 6, Rgba([255, 0, 0, 255]));
        let out = String::from_utf8(encode(&img)).unwrap();
        assert!(out.starts_with("\x1bP0;0;0q\"1;1;8;6"));
        // Pure red is palette entry 180 (5 * 36), at 100% red.
        assert!(out.contains("#180;2;100;0;0"));
        // All six rows set (63 + 63 = '~'), eight times over.
        assert!(out.contains("#180!8~"), "{out}");
        assert!(out.ends_with("-\x1b\\"));
    }

    #[test]
    fn colors_in_a_band_are_drawn_in_separate_passes() {
        let img = RgbaImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        });
        let out = String::from_utf8(encode(&img)).unwrap();
        // One row: bit 0 only is set, '@' (63 + 1), one cell per color.
        assert!(out.contains("#5@") || out.contains("#5?@"), "{out}");
        assert_eq!(out.matches('$').count(), 1);
    }

    #[test]
    fn a_partial_last_band_only_sets_the_rows_it_has() {
        let img = RgbaImage::from_pixel(1, 7, Rgba([0, 0, 0, 255]));
        let out = String::from_utf8(encode(&img)).unwrap();
        // Two bands: six rows ('~'), then a single row ('@').
        assert!(out.contains("#0~-#0@-"), "{out}");
    }
}
