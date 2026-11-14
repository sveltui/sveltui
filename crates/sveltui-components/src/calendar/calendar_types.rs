use std::rc::Rc;
use sveltui::prelude::*;

use crate::date::{Date, Weekday};

#[derive(Props)]
pub struct CalendarProps {
    /// The picked date. Use `bind:value` to share it, or pass a plain value
    /// and follow `on_value_change`.
    #[prop(bindable)]
    pub value: Bindable<Option<Date>>,
    #[prop(optional)]
    pub default_value: Option<Date>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(Option<Date>)>>,
    /// The earliest date the cursor can reach.
    #[prop(optional)]
    pub min: Option<Date>,
    /// The latest date the cursor can reach.
    #[prop(optional)]
    pub max: Option<Date>,
    /// Dates that can be looked at but not picked.
    #[prop(optional)]
    pub is_date_unavailable: Option<Rc<dyn Fn(Date) -> bool>>,
    #[prop(default = Weekday::Monday)]
    pub week_starts_on: Weekday,
    /// The grid's id: it's the calendar's one tab stop.
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// What the calendar is for, for assistive tech.
    #[prop(default = "Calendar".to_string())]
    pub label: String,
    /// Called after a date is picked.
    #[prop(optional)]
    pub on_select: Option<Rc<dyn Fn(Date)>>,
}
