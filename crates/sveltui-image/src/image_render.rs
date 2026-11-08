//! Turns a decoded image into terminal cells, two pixels to a cell: the
//! upper half block `▀` carries the top pixel in its foreground and the
//! bottom one in its background.

use image::{RgbaImage, imageops};

use crate::image_types::ImageFit;

/// One pixel of the sampled grid; `None` is left empty.
pub(crate) type Pixel = Option<[u8; 3]>;

/// A run of identical cells in one row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Run {
    pub text: String,
    pub fg: Option<[u8; 3]>,
    pub bg: Option<[u8; 3]>,
}

/// The cells an image of `width` x `height` pixels gets when no size is
/// given or only one side is, keeping its aspect ratio. `cell` is how many
/// pixels one cell holds across and down: `(1, 2)` for half blocks, the
/// terminal's real cell size for graphics protocols.
pub(crate) fn cell_size(
    (width, height): (u32, u32),
    cols: Option<u16>,
    rows: Option<u16>,
    (cw, ch): (u32, u32),
) -> (u16, u16) {
    let (width, height) = (width.max(1), height.max(1));
    let (cw, ch) = (cw.max(1), ch.max(1));
    let to_rows = |cols: u32| (height * cols * cw).div_ceil(width * ch).max(1);
    let to_cols = |rows: u32| (width * rows * ch).div_ceil(height * cw).max(1);
    let clamp = |v: u32| v.min(u32::from(u16::MAX)) as u16;
    match (cols, rows) {
        (Some(c), Some(r)) => (c, r),
        (Some(c), None) => (c, clamp(to_rows(u32::from(c)))),
        (None, Some(r)) => (clamp(to_cols(u32::from(r))), r),
        (None, None) => {
            let c = width.div_ceil(cw).min(60);
            (clamp(c), clamp(to_rows(c)))
        }
    }
}

fn blend(px: &image::Rgba<u8>, background: [u8; 3]) -> [u8; 3] {
    let a = u32::from(px[3]);
    let mix = |fg: u8, bg: u8| ((u32::from(fg) * a + u32::from(bg) * (255 - a) + 127) / 255) as u8;
    [
        mix(px[0], background[0]),
        mix(px[1], background[1]),
        mix(px[2], background[2]),
    ]
}

/// An image scaled for a box, and where its top-left corner sits in it.
/// The offset is negative when the image overflows and gets cropped.
pub(crate) struct Fitted {
    pub image: RgbaImage,
    pub off_x: i64,
    pub off_y: i64,
}

/// Scales `img` into a `width` x `height` pixel box the way `fit` says.
pub(crate) fn fit_to_box(img: &RgbaImage, width: u32, height: u32, fit: ImageFit) -> Fitted {
    let (w, h) = (img.width().max(1), img.height().max(1));
    let scale = |s: f64| {
        (
            ((f64::from(w) * s).round() as u32).max(1),
            ((f64::from(h) * s).round() as u32).max(1),
        )
    };
    let fit_scale = (f64::from(width) / f64::from(w)).min(f64::from(height) / f64::from(h));
    let cover_scale = (f64::from(width) / f64::from(w)).max(f64::from(height) / f64::from(h));
    let (tw, th) = match fit {
        ImageFit::Contain => scale(fit_scale),
        ImageFit::Cover => scale(cover_scale),
        ImageFit::Fill => (width.max(1), height.max(1)),
        ImageFit::None => (w, h),
    };
    let image = if (tw, th) == (img.width(), img.height()) {
        img.clone()
    } else {
        imageops::resize(img, tw, th, imageops::FilterType::Triangle)
    };
    Fitted {
        image,
        off_x: (i64::from(width) - i64::from(tw)) / 2,
        off_y: (i64::from(height) - i64::from(th)) / 2,
    }
}

#[cfg(any(feature = "kitty", feature = "iterm2", feature = "sixel"))]
/// An opaque `width` x `height` picture: `img` fitted and blended over
/// `background`, which also fills any empty margin.
pub(crate) fn canvas(
    img: &RgbaImage,
    width: u32,
    height: u32,
    fit: ImageFit,
    background: [u8; 3],
) -> RgbaImage {
    let fitted = fit_to_box(img, width, height, fit);
    let mut out = RgbaImage::from_pixel(
        width.max(1),
        height.max(1),
        image::Rgba([background[0], background[1], background[2], 255]),
    );
    for (x, y, px) in out.enumerate_pixels_mut() {
        let (sx, sy) = (i64::from(x) - fitted.off_x, i64::from(y) - fitted.off_y);
        if sx >= 0
            && sy >= 0
            && sx < i64::from(fitted.image.width())
            && sy < i64::from(fitted.image.height())
        {
            let [r, g, b] = blend(fitted.image.get_pixel(sx as u32, sy as u32), background);
            *px = image::Rgba([r, g, b, 255]);
        }
    }
    out
}

/// Samples `img` into a grid `cols` wide and `rows * 2` pixels tall.
pub(crate) fn sample(
    img: &RgbaImage,
    cols: u16,
    rows: u16,
    fit: ImageFit,
    background: [u8; 3],
) -> Vec<Vec<Pixel>> {
    let (cols, rows) = (u32::from(cols), u32::from(rows) * 2);
    let mut grid = vec![vec![None; cols as usize]; rows as usize];
    if cols == 0 || rows == 0 || img.width() == 0 || img.height() == 0 {
        return grid;
    }
    let fitted = fit_to_box(img, cols, rows, fit);
    let (tw, th) = (
        i64::from(fitted.image.width()),
        i64::from(fitted.image.height()),
    );
    for (y, row) in grid.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let (sx, sy) = (x as i64 - fitted.off_x, y as i64 - fitted.off_y);
            if sx >= 0 && sy >= 0 && sx < tw && sy < th {
                *cell = Some(blend(
                    fitted.image.get_pixel(sx as u32, sy as u32),
                    background,
                ));
            }
        }
    }
    grid
}

/// Groups a sampled grid into rows of cell runs.
pub(crate) fn runs(grid: &[Vec<Pixel>]) -> Vec<Vec<Run>> {
    grid.chunks(2)
        .map(|pair| {
            let top = &pair[0];
            let bottom = pair.get(1);
            let mut row: Vec<Run> = Vec::new();
            for x in 0..top.len() {
                let (upper, lower) = (top[x], bottom.and_then(|b| b[x]));
                let (glyph, fg, bg) = match (upper, lower) {
                    (Some(u), Some(l)) if u == l => ('█', Some(u), None),
                    (Some(u), Some(l)) => ('▀', Some(u), Some(l)),
                    (Some(u), None) => ('▀', Some(u), None),
                    (None, Some(l)) => ('▄', Some(l), None),
                    (None, None) => (' ', None, None),
                };
                match row.last_mut() {
                    Some(last) if last.fg == fg && last.bg == bg && last.text.ends_with(glyph) => {
                        last.text.push(glyph);
                    }
                    _ => row.push(Run {
                        text: glyph.to_string(),
                        fg,
                        bg,
                    }),
                }
            }
            row
        })
        .collect()
}
