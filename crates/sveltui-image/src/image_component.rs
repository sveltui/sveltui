use std::{cell::RefCell, rc::Rc};
use sveltui::prelude::*;

use crate::image_render::{cell_size, runs, sample};
use crate::image_types::*;
#[cfg(any(feature = "kitty", feature = "iterm2", feature = "sixel"))]
use crate::paint::GraphicsPaint;
#[cfg(any(feature = "kitty", feature = "iterm2", feature = "sixel"))]
use crate::protocol::ImageProtocol;

type Loaded = Result<Rc<image::RgbaImage>, String>;

fn decode(props: &ImageProps) -> Option<Loaded> {
    let bytes: Vec<u8> = match (&props.bytes, &props.src) {
        (Some(bytes), _) => bytes.0.to_vec(),
        (None, Some(path)) => match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => return Some(Err(format!("{path}: {error}"))),
        },
        (None, None) => return None,
    };
    Some(
        image::load_from_memory(&bytes)
            .map(|img| Rc::new(img.to_rgba8()))
            .map_err(|error| error.to_string()),
    )
}

/// Which image the props name, so a decoded one is kept until that changes.
fn key(props: &ImageProps) -> String {
    match (&props.bytes, &props.src) {
        (Some(bytes), _) => format!("bytes:{:p}:{}", Rc::as_ptr(&bytes.0), bytes.0.len()),
        (None, Some(path)) => format!("file:{path}"),
        (None, None) => String::new(),
    }
}

fn rgb([r, g, b]: [u8; 3]) -> Color {
    Color::Rgb { r, g, b }
}

fn rgb_of(color: Color) -> [u8; 3] {
    match color {
        Color::Rgb { r, g, b } => [r, g, b],
        Color::White => [255, 255, 255],
        Color::Grey => [192, 192, 192],
        Color::DarkGrey => [128, 128, 128],
        _ => [0, 0, 0],
    }
}

/// Which protocol to draw with and the cell size it needs: a graphics
/// protocol only when the terminal has told us how big a cell is.
#[cfg(any(feature = "kitty", feature = "iterm2", feature = "sixel"))]
fn graphics(protocol: ImageProtocol) -> Option<(ImageProtocol, (u16, u16))> {
    match protocol.resolve() {
        ImageProtocol::HalfBlock | ImageProtocol::Auto => None,
        resolved => sveltui::terminal::cell_pixels().map(|px| (resolved, px)),
    }
}

/// A raster image. It is drawn with the terminal's graphics protocol when
/// the build has one and the terminal speaks it, and with truecolor
/// half-block cells otherwise. Decoding happens once per source and is kept
/// across frames. When the image can't be loaded, `alt` is shown instead.
#[component]
pub fn Image(props: ImageProps) -> VNode {
    let cache = use_hook(|| Rc::new(RefCell::new(None::<(String, Option<Loaded>)>)));
    let wanted = key(&props);
    {
        let mut cached = cache.borrow_mut();
        if cached.as_ref().is_none_or(|(have, _)| *have != wanted) {
            *cached = Some((wanted, decode(&props)));
        }
    }
    let loaded = cache
        .borrow()
        .as_ref()
        .and_then(|(_, loaded)| loaded.clone());
    let alt = props.alt.clone().unwrap_or_default();
    let semantics = Semantics::new(Role::Image, alt.clone());

    let body = match loaded {
        Some(Ok(img)) => picture(&props, img),
        _ => placeholder(alt),
    };
    PressableElement::new()
        .semantics(semantics)
        .child(body)
        .into_node()
}

/// The text shown where an image could not be.
pub(crate) fn placeholder(alt: String) -> ViewElement {
    ViewElement::new().child(
        TextElement::new(if alt.is_empty() {
            String::from("[image]")
        } else {
            alt
        })
        .fg(Color::DarkGrey),
    )
}

/// A decoded image as a view: one picture for a graphics protocol, or rows
/// of half-block cells.
pub(crate) fn picture(props: &ImageProps, img: Rc<image::RgbaImage>) -> ViewElement {
    let dims = (img.width(), img.height());
    #[cfg(any(feature = "kitty", feature = "iterm2", feature = "sixel"))]
    if let Some((protocol, (cw, ch))) = graphics(props.protocol) {
        let (cols, rows) = cell_size(
            dims,
            props.width,
            props.height,
            (u32::from(cw), u32::from(ch)),
        );
        let paint = GraphicsPaint {
            protocol,
            source: img,
            fit: props.fit,
            background: rgb_of(props.background),
            cells: (cols, rows),
            cell_px: (cw, ch),
        };
        return ViewElement::new()
            .width(Dimension::Length(f32::from(cols)))
            .height(Dimension::Length(f32::from(rows)))
            .flex_shrink(0.0)
            .image(Rc::new(paint));
    }
    let (cols, rows) = cell_size(dims, props.width, props.height, (1, 2));
    let grid = sample(&img, cols, rows, props.fit, rgb_of(props.background));
    let lines = runs(&grid).into_iter().map(|row| {
        let spans = row.into_iter().map(|run| {
            TextElement::new(run.text)
                .fg(run.fg.map(rgb))
                .bg(run.bg.map(rgb).or(Some(props.background)))
        });
        ViewElement::new()
            .flex_direction(FlexDirection::Row)
            .height(Dimension::Length(1.0))
            .children(spans)
    });
    ViewElement::new()
        .width(Dimension::Length(f32::from(cols)))
        .height(Dimension::Length(f32::from(rows)))
        .overflow(Overflow::Hidden)
        .flex_shrink(0.0)
        .children(lines)
}
