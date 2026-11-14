use std::rc::Rc;
use sveltui::prelude::*;

use super::date_field_types::*;
use crate::calendar::month_name;
use crate::controllable::use_controllable;
use crate::date::{self, Date};
use crate::segmented_field::{
    Segment, SegmentedField, reset_typing, segmented_field, use_segment_ids, use_typed,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Year,
    Month,
    Day,
}

impl DateOrder {
    fn parts(self) -> [Part; 3] {
        match self {
            DateOrder::Ymd => [Part::Year, Part::Month, Part::Day],
            DateOrder::Mdy => [Part::Month, Part::Day, Part::Year],
            DateOrder::Dmy => [Part::Day, Part::Month, Part::Year],
        }
    }

    fn separator(self) -> &'static str {
        match self {
            DateOrder::Ymd => "-",
            DateOrder::Mdy | DateOrder::Dmy => "/",
        }
    }
}

/// What's in each segment so far. A half-typed date lives here, since the
/// value only holds whole dates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Parts {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

impl Parts {
    fn of(value: Option<Date>) -> Self {
        value.map_or_else(Self::default, |d| Self {
            year: Some(i32::from(d.year())),
            month: Some(i32::from(d.month())),
            day: Some(i32::from(d.day())),
        })
    }

    fn get(self, part: Part) -> Option<i32> {
        match part {
            Part::Year => self.year,
            Part::Month => self.month,
            Part::Day => self.day,
        }
    }

    fn with(mut self, part: Part, value: Option<i32>) -> Self {
        match part {
            Part::Year => self.year = value,
            Part::Month => self.month = value,
            Part::Day => self.day = value,
        }
        // A day past the end of the month moves back to its last day, so
        // Jan 31 becomes Feb 28 when the month changes.
        if let Some(day) = self.day {
            self.day = Some(day.min(self.days_in_month()));
        }
        self
    }

    /// The month's length, or 31 while the month (or year) isn't known.
    fn days_in_month(self) -> i32 {
        let year = self.year.unwrap_or(2000);
        self.month
            .and_then(|m| date_of(year, m, 1))
            .map_or(31, |d| i32::from(d.days_in_month()))
    }

    fn date(self) -> Option<Date> {
        date_of(self.year?, self.month?, self.day?)
    }

    /// The segment for one part, as it stands.
    fn segment(self, part: Part) -> Segment {
        let value = self.get(part);
        let today = Parts::of(Some(date::today()));
        let (name, max, page_step, digits, placeholder) = match part {
            Part::Year => ("year", 9999, 5, 4, "yyyy"),
            Part::Month => ("month", 12, 2, 2, "mm"),
            Part::Day => ("day", self.days_in_month(), 7, 2, "dd"),
        };
        let text = value.map_or_else(|| placeholder.to_owned(), |v| format!("{v:0digits$}"));
        let value_text = match (part, value) {
            (_, None) => "Empty".to_owned(),
            (Part::Month, Some(m)) => {
                date_of(2000, m, 1).map_or_else(|| m.to_string(), |d| month_name(d).to_owned())
            }
            (_, Some(v)) => v.to_string(),
        };
        Segment {
            name,
            value,
            start: today.get(part).unwrap_or(1),
            min: 1,
            max,
            step: 1,
            page_step,
            digits,
            text,
            value_text,
            letters: None,
        }
    }
}

fn date_of(year: i32, month: i32, day: i32) -> Option<Date> {
    Date::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
    )
    .ok()
}

/// A date typed in three parts: year, month and day, each its own tab stop.
/// Up/Down step a part (wrapping round), Page Up/Down take bigger steps,
/// Home/End go to its ends, digits type into it and move on once it's
/// full, Backspace clears it, Left/Right move between parts and Alt+Down
/// calls `on_open`. Scrolling over a part steps it too.
#[component]
pub fn DateField(props: DateFieldProps) -> VNode {
    // The ids follow the on-screen order, so `id` is always the first.
    let ids = use_segment_ids::<3>(props.id);
    let order = props.order.parts();

    let value = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value,
        props.on_value_change.clone(),
    );
    let parts = state_with(|| Parts::of(value.get()));
    let seen = state_with(|| value.get());
    let typed = use_typed(&ids);
    // A new date from outside replaces whatever was being typed.
    if seen.get() != value.get() {
        seen.set(value.get());
        parts.set(Parts::of(value.get()));
        reset_typing(typed);
    }

    // Every edit goes through here: the parts change, and the value follows
    // them, a date when they're all filled in and `None` when they're not.
    let edit: Rc<dyn Fn(usize, Option<i32>)> = {
        let value = value.clone();
        Rc::new(move |index, next| {
            let next = parts.get().with(order[index], next);
            parts.set(next);
            let date = next.date();
            seen.set(date);
            value.set(date);
        })
    };

    let current = parts.get();
    let invalid = current.date().is_some_and(|d| {
        props.min.is_some_and(|m| d < m)
            || props.max.is_some_and(|m| d > m)
            || props.is_date_unavailable.as_ref().is_some_and(|f| f(d))
    });
    let separator = props.order.separator();
    segmented_field(SegmentedField {
        label: props.label,
        ids: ids.to_vec(),
        segments: order.iter().map(|&part| current.segment(part)).collect(),
        separators: vec![separator; 2],
        invalid,
        disabled: props.disabled,
        typed,
        edit,
        on_open: props.on_open,
    })
}
