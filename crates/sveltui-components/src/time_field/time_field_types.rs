use std::rc::Rc;
use sveltui::prelude::*;

use crate::date::Time;

/// A 24-hour clock (`21:30`, the default) or a 12-hour one with AM/PM
/// (`09:30 PM`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HourCycle {
    H12,
    #[default]
    H24,
}

/// The smallest part the field asks for. Parts below it are zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Granularity {
    Hour,
    #[default]
    Minute,
    Second,
}

#[derive(Props)]
pub struct TimeFieldProps {
    /// The time, once every part is filled in; `None` while any is empty.
    /// Use `bind:value` to share it, or pass a plain value and follow
    /// `on_value_change`.
    #[prop(bindable)]
    pub value: Bindable<Option<Time>>,
    #[prop(optional)]
    pub default_value: Option<Time>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(Option<Time>)>>,
    /// Times before this are marked invalid.
    #[prop(optional)]
    pub min: Option<Time>,
    /// Times after this are marked invalid.
    #[prop(optional)]
    pub max: Option<Time>,
    #[prop(default)]
    pub hour_cycle: HourCycle,
    #[prop(default)]
    pub granularity: Granularity,
    /// How far Up/Down move the hour.
    #[prop(default = 1)]
    pub hour_step: u8,
    /// How far Up/Down move the minutes; they land on multiples of it.
    #[prop(default = 1)]
    pub minute_step: u8,
    #[prop(default = 1)]
    pub second_step: u8,
    /// What the time is for, for assistive tech.
    #[prop(default = "Time".to_string())]
    pub label: String,
    /// The first part's id.
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
}
