use std::rc::Rc;
use sveltui::prelude::*;

use crate::date::{Date, Weekday};
use crate::date_field::DateOrder;

#[derive(Props)]
pub struct DatePickerProps {
    /// The picked date. Use `bind:value` to share it, or pass a plain value
    /// and follow `on_value_change`.
    #[prop(bindable)]
    pub value: Bindable<Option<Date>>,
    #[prop(optional)]
    pub default_value: Option<Date>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(Option<Date>)>>,
    /// The earliest date: the calendar stops there and the field marks
    /// anything earlier invalid.
    #[prop(optional)]
    pub min: Option<Date>,
    /// The latest date.
    #[prop(optional)]
    pub max: Option<Date>,
    /// Dates that can't be picked.
    #[prop(optional)]
    pub is_date_unavailable: Option<Rc<dyn Fn(Date) -> bool>>,
    #[prop(default)]
    pub order: DateOrder,
    #[prop(default = Weekday::Monday)]
    pub week_starts_on: Weekday,
    /// What the date is for, for assistive tech.
    #[prop(default = "Date".to_string())]
    pub label: String,
    /// The field's first segment.
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    /// Whether the calendar is showing. Use `bind:open` to share it.
    #[prop(bindable)]
    pub open: Bindable<bool>,
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
}
