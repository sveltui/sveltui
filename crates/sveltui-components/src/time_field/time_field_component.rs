use std::rc::Rc;
use sveltui::prelude::*;

use super::time_field_types::*;
use crate::controllable::use_controllable;
use crate::date::{self, Time};
use crate::segmented_field::{
    Segment, SegmentedField, reset_typing, segmented_field, use_segment_ids, use_typed,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Hour,
    Minute,
    Second,
    Period,
}

/// What's in each segment so far. On a 12-hour clock `hour` holds 1 to 12,
/// as shown, and `pm` says which half of the day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Parts {
    hour: Option<i32>,
    minute: Option<i32>,
    second: Option<i32>,
    pm: Option<bool>,
}

/// The shape of the field: which clock, and down to which part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Shape {
    cycle: HourCycle,
    granularity: Granularity,
}

impl Shape {
    fn parts(self) -> Vec<Part> {
        let mut parts = vec![Part::Hour];
        if self.granularity >= Granularity::Minute {
            parts.push(Part::Minute);
        }
        if self.granularity >= Granularity::Second {
            parts.push(Part::Second);
        }
        if self.cycle == HourCycle::H12 {
            parts.push(Part::Period);
        }
        parts
    }
}

/// 0 to 23 as a 12-hour clock shows it: 12 AM, 1 AM … 12 PM, 1 PM.
fn to_12(hour: i32) -> (i32, bool) {
    let shown = match hour % 12 {
        0 => 12,
        h => h,
    };
    (shown, hour >= 12)
}

fn to_24(shown: i32, pm: bool) -> i32 {
    shown % 12 + if pm { 12 } else { 0 }
}

impl Parts {
    fn of(value: Option<Time>, shape: Shape) -> Self {
        let Some(t) = value else {
            return Self::default();
        };
        let hour = i32::from(t.hour());
        let (hour, pm) = match shape.cycle {
            HourCycle::H24 => (hour, None),
            HourCycle::H12 => {
                let (shown, pm) = to_12(hour);
                (shown, Some(pm))
            }
        };
        Self {
            hour: Some(hour),
            minute: Some(i32::from(t.minute())),
            second: Some(i32::from(t.second())),
            pm,
        }
    }

    fn get(self, part: Part) -> Option<i32> {
        match part {
            Part::Hour => self.hour,
            Part::Minute => self.minute,
            Part::Second => self.second,
            Part::Period => self.pm.map(i32::from),
        }
    }

    fn with(mut self, part: Part, value: Option<i32>, shape: Shape) -> Self {
        match part {
            Part::Hour => {
                self.hour = value;
                // Typing an hour on a 12-hour clock fills in AM, so the time
                // is whole without a trip to the last segment.
                if shape.cycle == HourCycle::H12 && value.is_some() && self.pm.is_none() {
                    self.pm = Some(false);
                }
            }
            Part::Minute => self.minute = value,
            Part::Second => self.second = value,
            Part::Period => self.pm = value.map(|v| v == 1),
        }
        self
    }

    /// The time, when every part the field shows is filled in.
    fn time(self, shape: Shape) -> Option<Time> {
        let hour = match shape.cycle {
            HourCycle::H24 => self.hour?,
            HourCycle::H12 => to_24(self.hour?, self.pm?),
        };
        let minute = match shape.granularity {
            Granularity::Hour => 0,
            _ => self.minute?,
        };
        let second = match shape.granularity {
            Granularity::Second => self.second?,
            _ => 0,
        };
        Time::new(
            i8::try_from(hour).ok()?,
            i8::try_from(minute).ok()?,
            i8::try_from(second).ok()?,
            0,
        )
        .ok()
    }

    fn segment(self, part: Part, shape: Shape, steps: [i32; 3]) -> Segment {
        let value = self.get(part);
        let now = Parts::of(Some(date::now_time()), shape);
        let start = now.get(part).unwrap_or(0);
        let [hour_step, minute_step, second_step] = steps;
        let twelve = shape.cycle == HourCycle::H12;
        let (name, min, max, step, page_step) = match part {
            Part::Hour if twelve => ("hour", 1, 12, hour_step, 2),
            Part::Hour => ("hour", 0, 23, hour_step, 2),
            Part::Minute => ("minute", 0, 59, minute_step, 15),
            Part::Second => ("second", 0, 59, second_step, 15),
            Part::Period => ("AM/PM", 0, 1, 1, 1),
        };
        let period = |pm: bool| if pm { "PM" } else { "AM" };
        let (text, value_text) = match (part, value) {
            (Part::Period, None) => ("--".to_owned(), "Empty".to_owned()),
            (Part::Period, Some(v)) => (period(v == 1).to_owned(), period(v == 1).to_owned()),
            (_, None) => ("--".to_owned(), "Empty".to_owned()),
            // "9 PM" says more than 9 does.
            (Part::Hour, Some(h)) if twelve => match self.pm {
                Some(pm) => (format!("{h:02}"), format!("{h} {}", period(pm))),
                None => (format!("{h:02}"), h.to_string()),
            },
            (_, Some(v)) => (format!("{v:02}"), v.to_string()),
        };
        Segment {
            name,
            value,
            start,
            min,
            max,
            step,
            page_step,
            digits: if part == Part::Period { 0 } else { 2 },
            text,
            value_text,
            letters: (part == Part::Period).then_some(period_letter as fn(char) -> Option<i32>),
        }
    }
}

fn period_letter(ch: char) -> Option<i32> {
    match ch.to_ascii_lowercase() {
        'a' => Some(0),
        'p' => Some(1),
        _ => None,
    }
}

/// A time of day typed in parts: hour, minute, an optional second, and
/// AM/PM on a 12-hour clock, each its own tab stop. Up/Down step a part by
/// its step (wrapping round), Page Up/Down take bigger steps, Home/End go to
/// its ends, digits type into it and move on once it's full, `a` and `p`
/// set AM/PM, Backspace clears it and Left/Right move between parts.
/// Scrolling over a part steps it too.
#[component]
pub fn TimeField(props: TimeFieldProps) -> VNode {
    // Room for the most parts there can be; the shape decides how many show.
    let ids = use_segment_ids::<4>(props.id);
    let shape = Shape {
        cycle: props.hour_cycle,
        granularity: props.granularity,
    };
    let order = shape.parts();
    let ids = ids[..order.len()].to_vec();

    let value = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value,
        props.on_value_change.clone(),
    );
    let parts = state_with(|| Parts::of(value.get(), shape));
    let seen = state_with(|| (value.get(), shape));
    let typed = use_typed(&ids);
    // A new time from outside, or a new clock, replaces what was typed.
    if seen.get() != (value.get(), shape) {
        seen.set((value.get(), shape));
        parts.set(Parts::of(value.get(), shape));
        reset_typing(typed);
    }

    let edit: Rc<dyn Fn(usize, Option<i32>)> = {
        let value = value.clone();
        let order = order.clone();
        Rc::new(move |index, next| {
            let next = parts.get().with(order[index], next, shape);
            parts.set(next);
            let time = next.time(shape);
            seen.set((time, shape));
            value.set(time);
        })
    };

    let current = parts.get();
    let invalid = current
        .time(shape)
        .is_some_and(|t| props.min.is_some_and(|m| t < m) || props.max.is_some_and(|m| t > m));
    let steps =
        [props.hour_step, props.minute_step, props.second_step].map(|s| i32::from(s.max(1)));
    let separators = order
        .iter()
        .skip(1)
        .map(|&part| if part == Part::Period { " " } else { ":" })
        .collect();
    segmented_field(SegmentedField {
        label: props.label,
        ids,
        segments: order
            .iter()
            .map(|&part| current.segment(part, shape, steps))
            .collect(),
        separators,
        invalid,
        disabled: props.disabled,
        typed,
        edit,
        on_open: None,
    })
}
