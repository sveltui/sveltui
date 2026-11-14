//! A segmented field: a row of numeric segments, each its own tab stop,
//! that type, step and wrap. Callers (the date and time fields) say what
//! each segment holds; this handles the keys, scrolling, focus and
//! semantics.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

/// One segment, as the field sees it right now.
#[derive(Clone)]
pub(crate) struct Segment {
    /// What assistive tech calls it: "year", "hour", "AM/PM".
    pub(crate) name: &'static str,
    pub(crate) value: Option<i32>,
    /// Where stepping an empty segment starts: today's day, the current hour.
    pub(crate) start: i32,
    pub(crate) min: i32,
    pub(crate) max: i32,
    pub(crate) step: i32,
    /// Page Up/Down's step.
    pub(crate) page_step: i32,
    /// How many digits fill it; 0 for one that isn't typed in digits.
    pub(crate) digits: usize,
    /// What's drawn: "07", "mm", "PM".
    pub(crate) text: String,
    /// The value in words, for assistive tech: "February", "9 PM", "Empty".
    pub(crate) value_text: String,
    /// Letters that set the value directly, like `a` and `p` for AM/PM.
    pub(crate) letters: Option<fn(char) -> Option<i32>>,
}

/// Where typing into a segment has got to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Typed {
    segment: Option<usize>,
    digits: String,
}

/// Everything a segmented field needs to draw itself.
pub(crate) struct SegmentedField {
    pub(crate) label: String,
    pub(crate) ids: Vec<FocusId>,
    pub(crate) segments: Vec<Segment>,
    /// What goes between each pair of segments.
    pub(crate) separators: Vec<&'static str>,
    pub(crate) invalid: bool,
    pub(crate) disabled: bool,
    pub(crate) typed: State<Typed>,
    /// Called with a segment's index and its new value.
    pub(crate) edit: Rc<dyn Fn(usize, Option<i32>)>,
    /// Called on Alt+Down.
    pub(crate) on_open: Option<Rc<dyn Fn()>>,
}

/// `N` stable segment ids; `first` replaces the first one when given.
pub(crate) fn use_segment_ids<const N: usize>(first: Option<FocusId>) -> [FocusId; N] {
    let mut ids = [FocusId::default(); N];
    for id in &mut ids {
        *id = use_hook(FocusId::new);
    }
    if let Some(first) = first {
        ids[0] = first;
    }
    ids
}

/// The typing state, dropped once focus leaves the field so coming back to
/// a segment starts it fresh.
pub(crate) fn use_typed(ids: &[FocusId]) -> State<Typed> {
    let typed = state(Typed::default());
    let within = ids.iter().any(|&id| use_focus(id).is_focused());
    if !within && typed.get() != Typed::default() {
        typed.set(Typed::default());
    }
    typed
}

/// Forgets half-typed digits, for when a new value arrives from outside.
pub(crate) fn reset_typing(typed: State<Typed>) {
    if typed.get() != Typed::default() {
        typed.set(Typed::default());
    }
}

/// Draws the field: a labelled group of segments, with a trailing `!` when
/// the value is invalid.
pub(crate) fn segmented_field(field: SegmentedField) -> VNode {
    let count = field.segments.len();
    let mut row: Vec<VNode> = Vec::with_capacity(count * 2 + 1);
    for (index, segment) in field.segments.into_iter().enumerate() {
        if index > 0 {
            let separator = field.separators.get(index - 1).copied().unwrap_or(" ");
            row.push(
                view! { <TextSpan foreground_color=Color::DarkGrey>{separator}</TextSpan> }
                    .into_node(),
            );
        }
        let moves = Moves {
            previous: index.checked_sub(1).map(|i| field.ids[i]),
            next: (index + 1 < count).then(|| field.ids[index + 1]),
        };
        let edit = Rc::clone(&field.edit);
        row.push(segment_view(
            index,
            segment,
            field.ids[index],
            field.disabled,
            moves,
            Rc::new(move |value| edit(index, value)),
            field.typed,
            field.on_open.clone(),
        ));
    }
    if field.invalid {
        row.push(view! { <TextSpan foreground_color=Color::DarkGrey>" !"</TextSpan> }.into_node());
    }
    let semantics = Semantics::new(Role::Group, field.label).invalid(field.invalid);
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

/// Moves a value by `by` (negative goes down), wrapping round past either
/// end, then lands on a multiple of the segment's step counted from `min`:
/// minute 07 with a step of 5 goes up to 10 and down to 05.
fn step(segment: &Segment, by: i32) -> i32 {
    let Segment { min, max, .. } = *segment;
    let Some(value) = segment.value else {
        return segment.start.clamp(min, max);
    };
    let grid = segment.step.max(1);
    let moved = min + (value + by - min).rem_euclid(max - min + 1);
    let offset = moved - min;
    let snapped = if offset % grid == 0 {
        moved
    } else if by > 0 {
        min + offset / grid * grid
    } else {
        min + (offset / grid + 1) * grid
    };
    // Past the last multiple that fits, use that one.
    snapped.min(min + (max - min) / grid * grid)
}

#[allow(clippy::too_many_arguments)]
fn segment_view(
    index: usize,
    segment: Segment,
    id: FocusId,
    disabled: bool,
    moves: Moves,
    set: Rc<dyn Fn(Option<i32>)>,
    typed: State<Typed>,
    on_open: Option<Rc<dyn Fn()>>,
) -> VNode {
    let focused = use_focus(id).is_focused() && !disabled;
    let empty = segment.value.is_none();
    let mut semantics =
        Semantics::new(Role::SpinButton, segment.name).value_text(segment.value_text.clone());
    semantics.value = segment.value.map(f64::from);
    semantics.value_min = Some(f64::from(segment.min));
    semantics.value_max = Some(f64::from(segment.max));
    let text = segment.text.clone();

    let on_scroll: Rc<dyn Fn(i16, i16)> = {
        let segment = segment.clone();
        let set = Rc::clone(&set);
        Rc::new(move |_dx: i16, dy: i16| {
            if disabled || dy == 0 {
                return;
            }
            typed.set(Typed::default());
            let by = if dy < 0 { segment.step } else { -segment.step };
            set(Some(step(&segment, by)));
        })
    };

    let on_key = move |key: KeyEvent, ctx: &mut EventContext| -> EventResult {
        if key.modifiers.contains(KeyModifiers::ALT) {
            if key.code == KeyCode::Down
                && let Some(open) = &on_open
            {
                open();
                return EventResult::Handled;
            }
            return EventResult::Ignored;
        }
        let jump = |value: i32| {
            typed.set(Typed::default());
            set(Some(value));
        };
        match key.code {
            KeyCode::Up => jump(step(&segment, segment.step)),
            KeyCode::Down => jump(step(&segment, -segment.step)),
            KeyCode::PageUp => jump(step(&segment, segment.page_step)),
            KeyCode::PageDown => jump(step(&segment, -segment.page_step)),
            KeyCode::Home => jump(segment.min),
            KeyCode::End => jump(segment.max),
            KeyCode::Backspace | KeyCode::Delete => {
                typed.set(Typed::default());
                set(None);
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
            KeyCode::Char(ch) if ch.is_ascii_digit() && segment.digits > 0 => {
                let (number, full) = type_digit(index, &segment, typed, ch);
                set(Some(number.max(segment.min)));
                if full {
                    typed.set(Typed::default());
                    if let Some(next) = moves.next {
                        ctx.focus(next);
                    }
                }
            }
            KeyCode::Char(ch) => {
                let Some(value) = segment.letters.and_then(|letter| letter(ch)) else {
                    return EventResult::Ignored;
                };
                jump(value);
                if let Some(next) = moves.next {
                    ctx.focus(next);
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Handled
    };

    let (fg, bg) = if focused {
        (Some(Color::Black), Some(Color::White))
    } else if empty || disabled {
        (Some(Color::DarkGrey), None)
    } else {
        (None, None)
    };
    view! {
        <Pressable focus_id=id tab_stop={!disabled} disabled=disabled semantics=semantics
            on_key=on_key>
            <View foreground_color=fg background_color=bg overflow_y=Overflow::Scroll
                scrollbar_visibility=ScrollbarVisibility::Never scroll_id=id on_scroll=on_scroll>
                <TextSpan>{text}</TextSpan>
            </View>
        </Pressable>
    }
    .into_node()
}

/// Adds a typed digit to the segment, returning the number it now reads
/// and whether it's full: no further digit could keep it in range, so
/// typing moves on. A digit that would push it out of range starts over.
fn type_digit(index: usize, segment: &Segment, typed: State<Typed>, ch: char) -> (i32, bool) {
    let mut now = typed.get();
    if now.segment != Some(index) {
        now = Typed {
            segment: Some(index),
            digits: String::new(),
        };
    }
    now.digits.push(ch);
    let mut number: i32 = now.digits.parse().unwrap_or(0);
    if number > segment.max {
        now.digits = ch.to_string();
        number = ch.to_digit(10).map_or(0, |d| d as i32);
    }
    let full = now.digits.len() >= segment.digits || number * 10 > segment.max;
    typed.set(now);
    (number, full)
}

#[cfg(test)]
mod segmented_field_tests;
