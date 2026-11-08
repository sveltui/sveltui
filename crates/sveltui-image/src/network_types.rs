use std::rc::Rc;
use sveltui::prelude::*;

use crate::{ImageFit, ImageProtocol};

/// Where a network image is in loading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageStatus {
    /// Nothing has been asked for yet.
    #[default]
    Idle,
    Loading,
    Loaded,
    /// Every source failed.
    Error,
}

#[derive(Props)]
pub struct RootProps {
    /// The box the image and its fallback fill, in cells.
    #[prop(optional)]
    pub width: Option<u16>,
    #[prop(optional)]
    pub height: Option<u16>,
    #[prop(default = ImageFit::Contain)]
    pub fit: ImageFit,
    #[prop(default = ImageProtocol::Auto)]
    pub protocol: ImageProtocol,
    /// What transparent pixels blend into.
    #[prop(default = Color::Black)]
    pub background: Color,
    /// Where loading has got to. Pass it as `bind:status={state}` to read
    /// it from the parent.
    #[prop(bindable)]
    pub status: Bindable<ImageStatus>,
    #[prop(optional)]
    pub on_loading_status_change: Option<Rc<dyn Fn(ImageStatus)>>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ImageProps {
    /// An `http://` or `https://` URL, or a path to a file, which makes a
    /// good last resort in a chain of sources.
    pub src: String,
    #[prop(optional)]
    pub alt: Option<String>,
    /// How long a request may take, in milliseconds.
    #[prop(default = 10_000)]
    pub timeout_ms: u64,
    /// The most bytes a response may have.
    #[prop(default = 8_000_000)]
    pub max_bytes: usize,
}

#[derive(Props)]
pub struct FallbackProps {
    /// How long to wait before showing the fallback while the image is
    /// still loading, in milliseconds, so that a quick load doesn't flash
    /// it. A failed load shows it at once.
    #[prop(default = 0)]
    pub delay_ms: u64,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
