use std::rc::Rc;
use sveltui::prelude::*;

use crate::protocol::ImageProtocol;

/// How an image fits the box it is given.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    /// The whole image, scaled to fit and centered; the rest of the box
    /// stays empty.
    #[default]
    Contain,
    /// Scaled to cover the box, with what overflows cropped.
    Cover,
    /// Stretched to the box, ignoring the aspect ratio.
    Fill,
    /// Unscaled, one pixel per half cell, centered and cropped.
    None,
}

/// The encoded bytes of an image (PNG, JPEG, GIF or WebP), shared cheaply
/// between frames.
#[derive(Clone, Debug)]
pub struct ImageBytes(pub(crate) Rc<[u8]>);

impl ImageBytes {
    pub fn new(bytes: impl Into<Rc<[u8]>>) -> Self {
        Self(bytes.into())
    }
}

impl From<Vec<u8>> for ImageBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl IntoPropOption<ImageBytes> for ImageBytes {
    fn into_prop_option(self) -> Option<ImageBytes> {
        Some(self)
    }
}

#[derive(Props)]
pub struct ImageProps {
    /// The path of an image file (PNG, JPEG, GIF or WebP).
    #[prop(optional)]
    pub src: Option<String>,
    /// The encoded bytes of an image, instead of a file.
    #[prop(optional)]
    pub bytes: Option<ImageBytes>,
    /// What the image shows, for assistive tech. It is also drawn when the
    /// image can't be loaded.
    #[prop(optional)]
    pub alt: Option<String>,
    /// The width in cells. Left out, it follows `height`, or the image's
    /// own width up to 60 cells.
    #[prop(optional)]
    pub width: Option<u16>,
    /// The height in cells. Left out, it follows the width and the aspect
    /// ratio.
    #[prop(optional)]
    pub height: Option<u16>,
    #[prop(default = ImageFit::Contain)]
    pub fit: ImageFit,
    /// How the image reaches the terminal. `Auto` uses a graphics protocol
    /// when the build and the terminal have one, else half-block cells.
    #[prop(default = ImageProtocol::Auto)]
    pub protocol: ImageProtocol,
    /// What transparent pixels blend into.
    #[prop(default = Color::Black)]
    pub background: Color,
}
