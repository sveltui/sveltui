use std::rc::Rc;
use sveltui::prelude::*;

use crate::date::Date;

/// The order the year, month and day segments come in. Year first
/// (`2026-02-13`) is the default; the other two use slashes
/// (`02/13/2026`, `13/02/2026`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateOrder {
    #[default]
    Ymd,
    Mdy,
    Dmy,
}

#[derive(Props)]
pub struct DateFieldProps {
    /// The date, once every segment is filled in; `None` while any is
    /// empty. Use `bind:value` to share it, or pass a plain value and
    /// follow `on_value_change`.
    #[prop(bindable)]
    pub value: Bindable<Option<Date>>,
    #[prop(optional)]
    pub default_value: Option<Date>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(Option<Date>)>>,
    /// Dates before this are marked invalid.
    #[prop(optional)]
    pub min: Option<Date>,
    /// Dates after this are marked invalid.
    #[prop(optional)]
    pub max: Option<Date>,
    /// Dates that are marked invalid when typed in.
    #[prop(optional)]
    pub is_date_unavailable: Option<Rc<dyn Fn(Date) -> bool>>,
    #[prop(default)]
    pub order: DateOrder,
    /// What the date is for, for assistive tech.
    #[prop(default = "Date".to_string())]
    pub label: String,
    /// The first segment's id.
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    /// Called on Alt+Down; a date picker opens its calendar on it.
    #[prop(optional)]
    pub on_open: Option<Rc<dyn Fn()>>,
}
