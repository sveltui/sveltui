use super::*;
use crate::clock::TEST_TIME;
use crate::date::Time;
use crate::date_field::date_field_tests::Harness;
use crossterm::event::KeyCode;
use jiff::civil::time;
use sveltui::prelude::*;

fn t(hour: i8, minute: i8) -> Time {
    time(hour, minute, 0, 0)
}

/// A field with the clock pinned to 09:41.
fn open(view: impl Fn() -> VNode + 'static) -> Harness {
    TEST_TIME.with(|now| now.set(Some(t(9, 41))));
    let mut f = Harness::new(view);
    f.key(KeyCode::Tab);
    f
}

fn h24(value: State<Option<Time>>) -> impl Fn() -> VNode {
    move || {
        view! { <TimeField bind:value={value} label="Alarm" max={Some(t(22, 0))} /> }.into_node()
    }
}

fn h12(value: State<Option<Time>>) -> impl Fn() -> VNode {
    move || {
        view! { <TimeField bind:value={value} label="Meeting" hour_cycle=HourCycle::H12 /> }
            .into_node()
    }
}

#[test]
fn typing_fills_hours_then_minutes() {
    let value = state(None);
    let mut f = open(h24(value));
    assert_eq!(f.line(), "--:--");
    f.typing("07");
    assert_eq!(f.focused().label, "minute");
    f.typing("30");
    assert_eq!(
        (f.line(), value.get()),
        ("07:30".to_owned(), Some(t(7, 30)))
    );
}

#[test]
fn an_hour_that_cannot_grow_moves_on_at_once() {
    let value = state(None);
    let mut f = open(h24(value));
    f.typing("1");
    assert_eq!(f.focused().label, "hour", "1 could still be 10 to 19");
    f.key(KeyCode::Backspace);
    f.typing("3");
    assert_eq!(f.focused().label, "minute");
}

#[test]
fn steps_snap_and_page_and_home_end_jump() {
    let value = state(Some(t(7, 7)));
    let mut f = open(move || view! { <TimeField bind:value={value} minute_step=5 /> }.into_node());
    f.key(KeyCode::End);
    assert_eq!(value.get(), Some(t(23, 7)));
    f.key(KeyCode::Home);
    assert_eq!(value.get(), Some(t(0, 7)));
    f.key(KeyCode::Right);
    f.key(KeyCode::Up);
    assert_eq!(value.get(), Some(t(0, 10)), "onto the next multiple of 5");
    f.key(KeyCode::PageDown);
    assert_eq!(value.get(), Some(t(0, 55)), "15 back, round the hour");
}

#[test]
fn an_empty_part_starts_at_the_current_time() {
    let value = state(None);
    let mut f = open(h24(value));
    f.key(KeyCode::Up);
    f.key(KeyCode::Right);
    f.key(KeyCode::Down);
    assert_eq!(value.get(), Some(t(9, 41)));
}

#[test]
fn a_twelve_hour_clock_fills_am_and_takes_a_and_p() {
    let value = state(None);
    let mut f = open(h12(value));
    f.typing("9");
    f.typing("30");
    assert_eq!(f.line(), "09:30 AM");
    assert_eq!(value.get(), Some(t(9, 30)));
    assert_eq!(f.focused().label, "AM/PM");
    f.typing("p");
    assert_eq!(value.get(), Some(t(21, 30)));
    f.key(KeyCode::Up);
    assert_eq!(value.get(), Some(t(9, 30)), "Up toggles");
}

#[test]
fn twelve_is_midnight_or_noon() {
    let value = state(Some(t(0, 0)));
    let mut f = open(h12(value));
    assert_eq!(f.line(), "12:00 AM");
    value.set(Some(t(12, 0)));
    f.render();
    assert_eq!(f.line(), "12:00 PM");
    assert_eq!(f.focused().value_text.as_deref(), Some("12 PM"));
}

#[test]
fn granularity_sets_the_parts() {
    let value = state(Some(time(7, 30, 15, 0)));
    let f = open(move || {
        view! { <TimeField bind:value={value} granularity=Granularity::Second /> }.into_node()
    });
    assert_eq!(f.line(), "07:30:15");
}

#[test]
fn an_hour_field_has_one_part() {
    let value = state(None);
    let mut f = open(move || {
        view! { <TimeField bind:value={value} granularity=Granularity::Hour /> }.into_node()
    });
    f.typing("8");
    assert_eq!((f.line(), value.get()), ("08".to_owned(), Some(t(8, 0))));
}

#[test]
fn backspace_clears_and_a_new_value_replaces_the_parts() {
    let value = state(Some(t(7, 30)));
    let mut f = open(h24(value));
    f.key(KeyCode::Backspace);
    assert_eq!((f.line(), value.get()), ("--:30".to_owned(), None));
    value.set(Some(t(18, 5)));
    f.render();
    assert_eq!(f.line(), "18:05");
}

#[test]
fn a_time_past_max_is_marked_invalid() {
    let value = state(Some(t(23, 0)));
    let f = open(h24(value));
    assert_eq!(f.line(), "23:00 !");
    let nodes = f.tree.nodes();
    let group = nodes
        .iter()
        .find(|n| n.interaction.semantics.role == Role::Group)
        .map(|n| n.interaction.semantics.clone())
        .expect("a group");
    assert_eq!((group.label.as_str(), group.invalid), ("Alarm", Some(true)));
}

#[test]
fn each_part_is_a_spin_button() {
    let value = state(Some(t(21, 5)));
    let mut f = open(h12(value));
    let hour = f.focused();
    assert_eq!(hour.role, Role::SpinButton);
    assert_eq!(
        (hour.value, hour.value_min, hour.value_max),
        (Some(9.0), Some(1.0), Some(12.0))
    );
    assert_eq!(hour.value_text.as_deref(), Some("9 PM"));
    f.key(KeyCode::Tab);
    f.key(KeyCode::Tab);
    let period = f.focused();
    assert_eq!(
        (period.label.as_str(), period.value_text.as_deref()),
        ("AM/PM", Some("PM"))
    );
}
