//! What tooltips and hover cards share: when to open (after a delay, on
//! hover or focus), when to close, and the trigger and content parts.

mod hover_popup_component;
mod hover_popup_types;

pub(crate) use hover_popup_component::*;
pub(crate) use hover_popup_types::*;

#[cfg(test)]
pub(crate) mod hover_popup_tests;
