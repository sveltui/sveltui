use super::*;
use crate::date::Date;
use crate::date_field::date_field_tests::{Harness, TODAY};
use crossterm::event::{KeyCode, KeyModifiers};
use jiff::civil::date;
use sveltui::prelude::*;

fn picker(value: State<Option<Date>>) -> impl Fn() -> VNode {
    move || {
        view! {
            <View padding=1>
                <DatePicker bind:value={value} label="Due"
                    is_date_unavailable={|d: Date| d == date(2026, 2, 20)} />
            </View>
        }
        .into_node()
    }
}

fn is_open(p: &Harness) -> bool {
    p.screen.contains("February 2026") || p.screen.contains("March 2026")
}

/// Tabs past the three segments onto the button.
fn on_trigger(p: &mut Harness) {
    for _ in 0..4 {
        p.key(KeyCode::Tab);
    }
    assert_eq!(p.focused().label, "Choose date");
}

#[test]
fn enter_on_a_day_picks_it_closes_and_returns_focus() {
    let value = state(None);
    let mut p = Harness::new(picker(value));
    on_trigger(&mut p);
    p.key(KeyCode::Enter);
    assert!(is_open(&p), "{}", p.screen);
    assert_eq!(p.focused().role, Role::Grid, "focus starts on the grid");

    p.key(KeyCode::Right);
    p.key(KeyCode::Enter);
    assert_eq!(value.get(), Some(date(2026, 2, 14)));
    assert!(!is_open(&p), "{}", p.screen);
    assert_eq!(p.focused().label, "Choose date");
    assert!(p.screen.contains("2026-02-14 [▾]"), "{}", p.screen);
}

#[test]
fn escape_closes_without_a_change_and_reopening_starts_fresh() {
    let value = state(None);
    let mut p = Harness::new(picker(value));
    on_trigger(&mut p);
    p.key(KeyCode::Enter);
    p.key(KeyCode::Right);
    p.key(KeyCode::Right);
    p.key(KeyCode::Esc);
    assert!(!is_open(&p));
    assert_eq!(value.get(), None);
    assert_eq!(p.focused().label, "Choose date");

    // The cursor went back to today, not where it was left.
    p.key(KeyCode::Enter);
    p.key(KeyCode::Enter);
    assert_eq!(value.get(), Some(TODAY));
}

#[test]
fn reopening_starts_on_the_picked_date() {
    let value = state(Some(date(2026, 3, 9)));
    let mut p = Harness::new(picker(value));
    on_trigger(&mut p);
    p.key(KeyCode::Enter);
    assert!(p.screen.contains("March 2026"), "{}", p.screen);
    p.key(KeyCode::Right);
    p.key(KeyCode::Enter);
    assert_eq!(value.get(), Some(date(2026, 3, 10)));
}

#[test]
fn tab_stays_inside_while_open() {
    let mut p = Harness::new(picker(state(None)));
    on_trigger(&mut p);
    p.key(KeyCode::Enter);
    for _ in 0..5 {
        p.key(KeyCode::Tab);
        assert!(is_open(&p));
        let label = p.focused().label;
        assert!(
            ["Previous month", "Next month", "Due"].contains(&label.as_str()),
            "focus left the popover: {label}"
        );
    }
}

#[test]
fn an_unavailable_day_does_not_close_it() {
    let value = state(None);
    let mut p = Harness::new(picker(value));
    on_trigger(&mut p);
    p.key(KeyCode::Enter);
    p.key(KeyCode::Down);
    p.key(KeyCode::Enter);
    assert_eq!(value.get(), None);
    assert!(is_open(&p));
}

#[test]
fn clicks_open_pick_and_close_outside() {
    let value = state(None);
    let mut p = Harness::new(picker(value));
    let (x, y) = p.find("▾", 0);
    p.click(x, y);
    assert!(is_open(&p), "{}", p.screen);
    p.click(39, 15);
    assert!(!is_open(&p), "an outside press closes it");
    assert_eq!(value.get(), None);

    p.click(x, y);
    let (dx, dy) = p.find("25", 3);
    p.click(dx, dy);
    assert_eq!(value.get(), Some(date(2026, 2, 25)));
    assert!(!is_open(&p));
}

#[test]
fn alt_down_in_the_field_opens_it() {
    let mut p = Harness::new(picker(state(None)));
    p.key(KeyCode::Tab);
    p.key_with(KeyCode::Down, KeyModifiers::ALT);
    assert!(is_open(&p), "{}", p.screen);
    assert_eq!(p.focused().role, Role::Grid);
}

#[test]
fn the_button_says_what_it_opens() {
    let mut p = Harness::new(picker(state(None)));
    on_trigger(&mut p);
    let closed = p.focused();
    assert_eq!(
        (closed.has_popup, closed.expanded),
        (Some(Role::Dialog), Some(false))
    );
    p.key(KeyCode::Enter);
    let dialog = p
        .tree
        .nodes()
        .iter()
        .find(|n| n.interaction.semantics.role == Role::Dialog)
        .map(|n| n.interaction.semantics.clone())
        .expect("a dialog");
    assert_eq!(dialog.modal, Some(true));
    p.key(KeyCode::Esc);
    assert_eq!(p.focused().expanded, Some(false));
}
