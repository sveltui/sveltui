use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::date_field_types::*;
use crate::calendar::month_name;
use crate::controllable::use_controllable;
use crate::date::{self, Date};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Segment {
    Year,
    Month,
    Day,
}

impl DateOrder {
    fn segments(self) -> [Segment; 3] {
        match self {
            DateOrder::Ymd => [Segment::Year, Segment::Month, Segment::Day],
            DateOrder::Mdy => [Segment::Month, Segment::Day, Segment::Year],
            DateOrder::Dmy => [Segment::Day, Segment::Month, Segment::Year],
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

    fn get(self, segment: Segment) -> Option<i32> {
        match segment {
            Segment::Year => self.year,
            Segment::Month => self.month,
            Segment::Day => self.day,
        }
    }

    fn with(mut self, segment: Segment, value: Option<i32>) -> Self {
        match segment {
            Segment::Year => self.year = value,
            Segment::Month => self.month = value,
            Segment::Day => self.day = value,
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

    fn range(self, segment: Segment) -> (i32, i32) {
        match segment {
            Segment::Year => (1, 9999),
            Segment::Month => (1, 12),
            Segment::Day => (1, self.days_in_month()),
        }
    }

    fn date(self) -> Option<Date> {
        date_of(self.year?, self.month?, self.day?)
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

/// Where typing into a segment has got to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Typed {
    segment: Option<Segment>,
    digits: String,
}

/// A date typed in three parts: year, month and day, each its own tab stop.
/// Up/Down step a part (wrapping round), digits type into it and move on
/// once it's full, Backspace clears it, Left/Right move between parts and
/// Alt+Down calls `on_open`.
#[component]
pub fn DateField(props: DateFieldProps) -> VNode {
    let own = [
        use_hook(FocusId::new),
        use_hook(FocusId::new),
        use_hook(FocusId::new),
    ];
    let segments = props.order.segments();
    // The ids follow the on-screen order, so `id` is always the first.
    let ids = [props.id.unwrap_or(own[0]), own[1], own[2]];

    let value = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value,
        props.on_value_change.clone(),
    );
    let parts = state_with(|| Parts::of(value.get()));
    let seen = state_with(|| value.get());
    let typed = state(Typed::default());
    // A new date from outside replaces whatever was being typed.
    if seen.get() != value.get() {
        seen.set(value.get());
        parts.set(Parts::of(value.get()));
        typed.set(Typed::default());
    }

    // Every edit goes through here: the parts change, and the value follows
    // them, a date when they're all filled in and `None` when they're not.
    let edit: Rc<dyn Fn(Parts)> = {
        let value = value.clone();
        Rc::new(move |next: Parts| {
            parts.set(next);
            let date = next.date();
            seen.set(date);
            value.set(date);
        })
    };

    // Half-typed digits are dropped once focus leaves the field, so coming
    // back to a segment starts it fresh.
    let within = ids.iter().any(|&id| use_focus(id).is_focused());
    if !within && typed.get() != Typed::default() {
        typed.set(Typed::default());
    }

    let current = parts.get();
    let invalid = current.date().is_some_and(|d| {
        props.min.is_some_and(|m| d < m)
            || props.max.is_some_and(|m| d > m)
            || props.is_date_unavailable.as_ref().is_some_and(|f| f(d))
    });

    let mut row: Vec<VNode> = Vec::with_capacity(6);
    for (index, segment) in segments.into_iter().enumerate() {
        if index > 0 {
            row.push(
                view! { <TextSpan foreground_color=Color::DarkGrey>{props.order.separator()}</TextSpan> }
                    .into_node(),
            );
        }
        let id = ids[index];
        let previous = index.checked_sub(1).map(|i| ids[i]);
        let next = ids.get(index + 1).copied();
        row.push(segment_view(
            segment,
            id,
            current,
            props.disabled,
            Moves { previous, next },
            Rc::clone(&edit),
            typed,
            props.on_open.clone(),
        ));
    }
    if invalid {
        row.push(view! { <TextSpan foreground_color=Color::DarkGrey>" !"</TextSpan> }.into_node());
    }

    let semantics = Semantics::new(Role::Group, props.label).invalid(invalid);
    view! {
        <Pressable semantics=semantics>
            <View flex_direction=FlexDirection::Row>{row}</View>
        </Pressable>
    }
    .into_node()
}

/// The segments either side of one, for Left/Right and for moving on once
/// a segment is full.
#[derive(Clone, Copy)]
struct Moves {
    previous: Option<FocusId>,
    next: Option<FocusId>,
}

#[allow(clippy::too_many_arguments)]
fn segment_view(
    segment: Segment,
    id: FocusId,
    parts: Parts,
    disabled: bool,
    moves: Moves,
    edit: Rc<dyn Fn(Parts)>,
    typed: State<Typed>,
    on_open: Option<Rc<dyn Fn()>>,
) -> VNode {
    let focused = use_focus(id).is_focused() && !disabled;
    let value = parts.get(segment);
    let (min, max) = parts.range(segment);

    let on_key = move |key: KeyEvent, ctx: &mut EventContext| -> EventResult {
        let parts_now = parts;
        if key.modifiers.contains(KeyModifiers::ALT) {
            if key.code == KeyCode::Down
                && let Some(open) = &on_open
            {
                open();
                return EventResult::Handled;
            }
            return EventResult::Ignored;
        }
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                let step = if key.code == KeyCode::Up { 1 } else { -1 };
                let next = match value {
                    // An empty segment starts from today.
                    None => Parts::of(Some(date::today())).get(segment).unwrap_or(min),
                    Some(v) => (v - min + step).rem_euclid(max - min + 1) + min,
                };
                typed.set(Typed::default());
                edit(parts_now.with(segment, Some(next)));
            }
            KeyCode::Backspace | KeyCode::Delete => {
                typed.set(Typed::default());
                edit(parts_now.with(segment, None));
            }
            KeyCode::Left => {
                typed.set(Typed::default());
                if let Some(previous) = moves.previous {
                    ctx.focus(previous);
                }
            }
            KeyCode::Right => {
                typed.set(Typed::default());
                if let Some(next) = moves.next {
                    ctx.focus(next);
                }
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let (number, full) = type_digit(segment, typed, ch, max);
                edit(parts_now.with(segment, Some(number.max(min))));
                if full {
                    typed.set(Typed::default());
                    if let Some(next) = moves.next {
                        ctx.focus(next);
                    }
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Handled
    };

    let (name, width) = match segment {
        Segment::Year => ("year", 4),
        Segment::Month => ("month", 2),
        Segment::Day => ("day", 2),
    };
    let text = match value {
        Some(v) => format!("{v:0width$}"),
        None => match segment {
            Segment::Year => "yyyy".to_owned(),
            Segment::Month => "mm".to_owned(),
            Segment::Day => "dd".to_owned(),
        },
    };
    let value_text = match (segment, value) {
        (_, None) => "Empty".to_owned(),
        (Segment::Month, Some(m)) => {
            date_of(2000, m, 1).map_or_else(|| m.to_string(), |d| month_name(d).to_owned())
        }
        (_, Some(v)) => v.to_string(),
    };
    let mut semantics = Semantics::new(Role::SpinButton, name).value_text(value_text);
    semantics.value = value.map(f64::from);
    semantics.value_min = Some(f64::from(min));
    semantics.value_max = Some(f64::from(max));

    let (fg, bg) = if focused {
        (Some(Color::Black), Some(Color::White))
    } else if value.is_none() || disabled {
        (Some(Color::DarkGrey), None)
    } else {
        (None, None)
    };
    view! {
        <Pressable focus_id=id tab_stop={!disabled} disabled=disabled semantics=semantics
            on_key=on_key>
            <View foreground_color=fg background_color=bg>
                <TextSpan>{text}</TextSpan>
            </View>
        </Pressable>
    }
    .into_node()
}

/// Adds a typed digit to the segment, returning the number it now reads
/// and whether it's full: no further digit could keep it in range, so
/// typing moves on. A digit that would push it out of range starts over.
fn type_digit(segment: Segment, typed: State<Typed>, ch: char, max: i32) -> (i32, bool) {
    let mut now = typed.get();
    if now.segment != Some(segment) {
        now = Typed {
            segment: Some(segment),
            digits: String::new(),
        };
    }
    let longest = if segment == Segment::Year { 4 } else { 2 };
    now.digits.push(ch);
    let mut number: i32 = now.digits.parse().unwrap_or(0);
    if number > max {
        now.digits = ch.to_string();
        number = ch.to_digit(10).map_or(0, |d| d as i32);
    }
    let full = now.digits.len() >= longest || number * 10 > max;
    typed.set(now);
    (number, full)
}
