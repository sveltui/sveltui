//! Pixels to the bytes each terminal graphics protocol understands. Every
//! encoder takes an opaque RGBA picture that is already the size it will
//! be shown at.

#[cfg(feature = "iterm2")]
pub(crate) mod iterm2;
#[cfg(feature = "kitty")]
pub(crate) mod kitty;
#[cfg(feature = "sixel")]
pub(crate) mod sixel;
