//! Images for sveltui. `Image` decodes a file or bytes and draws it with the
//! terminal's graphics protocol when there is one (`kitty`, `iterm2` and
//! `sixel` features), and with half-block cells anywhere else.

mod base64;
mod encode;
mod image_component;
mod image_render;
mod image_types;
#[cfg(any(feature = "kitty", feature = "iterm2", feature = "sixel"))]
mod paint;
mod protocol;

#[cfg(feature = "network")]
mod controllable;
#[cfg(feature = "network")]
mod network_component;
#[cfg(feature = "network")]
mod network_loader;
#[cfg(feature = "network")]
mod network_types;

pub use image_component::*;
pub use image_types::*;
pub use protocol::ImageProtocol;

#[cfg(feature = "network")]
pub use network_loader::{clear_cache, load_error};
#[cfg(feature = "network")]
pub use network_types::ImageStatus;

/// An image loaded over the network, with fallbacks like Base UI's Avatar:
/// `NetworkImage::Root` holds the box and the loading status, each
/// `NetworkImage::Image` is a source to try in order, and
/// `NetworkImage::Fallback` shows while no image has loaded.
#[cfg(feature = "network")]
#[allow(non_snake_case)]
pub mod NetworkImage {
    pub use crate::network_component::{Fallback, Image, Root};
    pub use crate::network_types::{FallbackProps, ImageProps, ImageStatus, RootProps};
}

#[cfg(test)]
mod image_tests;
#[cfg(all(test, feature = "network"))]
mod network_tests;

pub mod prelude {
    pub use crate::{Image, ImageBytes, ImageFit, ImageProps, ImageProtocol};
}
