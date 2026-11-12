//! State a component can own itself, take from its parent, or share with
//! its parent through a `bind_*` prop.
mod controllable_types;

pub(crate) use controllable_types::{Controllable, use_controllable};

#[cfg(test)]
mod controllable_tests;
