use super::*;
use crate::clock::TEST_TODAY;
use crate::date::Date;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use jiff::civil::date;
use sveltui::Runtime;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const WIDTH: u16 = 40;
const HEIGHT: u16 = 16;
/// A Friday.
pub(crate) const TODAY: Date = date(2026, 2, 13);

/// Renders a view against a pinned today and drives it by key and click.
pub(crate) struct Harness {
    pub(crate) focus: FocusManager,
    view: Box<dyn Fn() -> VNode>,
    pub(crate) hits: HitMap,
    pub(crate) tree: InteractionTree,
    pub(crate) screen: String,
}

impl Harness {
    pub(crate) fn new(view: impl Fn() -> VNode + 'static) -> Self {
        TEST_TODAY.with(|t| t.set(Some(TODAY)));
        let mut harness = Self {
            focus: FocusManager::new(),
            view: Box::new(view),
            hits: HitMap::new(),
            tree: InteractionTree::default(),
            screen: String::new(),
        };
        harness.render();
        harness
    }

    /// Renders until focus holds still and two frames look the same.
    pub(crate) fn render(&mut self) {
        for _ in 0..10 {
            Runtime::set_dirty(false);
            reset_focus_counter();
            begin_frame();
            let node = self.focus.with_context(|| (self.view)());
            end_frame();
            self.tree = InteractionTree::from_node(&node);
            let mut buffer = Buffer::new(WIDTH, HEIGHT);
            self.hits = HitMap::new();
            let hits = &mut self.hits;
            self.focus.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, WIDTH, HEIGHT), &mut buffer, hits)
            });
            let changed = self.focus.reconcile(self.hits.interactions.clone());
            let screen = (0..HEIGHT)
                .map(|y| (0..WIDTH).map(|x| buffer.get(x, y).unwrap().ch).collect())
                .collect::<Vec<String>>()
                .join("\n");
            if !changed && screen == self.screen {
                return;
            }
            self.screen = screen;
        }
        panic!("the view did not settle:\n{}", self.screen);
    }

    pub(crate) fn key_with(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.focus.dispatch(KeyEvent::new(code, modifiers));
        self.render();
    }

    pub(crate) fn key(&mut self, code: KeyCode) {
        self.key_with(code, KeyModifiers::NONE);
    }

    pub(crate) fn typing(&mut self, text: &str) {
        for ch in text.chars() {
            self.key(KeyCode::Char(ch));
        }
    }

    /// The first line, trimmed: where the field is.
    pub(crate) fn line(&self) -> String {
        self.screen.lines().next().unwrap().trim().to_owned()
    }

    /// The semantics of the focused element.
    pub(crate) fn focused(&self) -> Semantics {
        let id = self.focus.focused().expect("something focused");
        self.tree
            .node(id)
            .expect("the focused node")
            .interaction
            .semantics
            .clone()
    }

    pub(crate) fn find(&self, text: &str, from_row: usize) -> (u16, u16) {
        self.screen
            .lines()
            .enumerate()
            .skip(from_row)
            .find_map(|(y, l)| {
                l.find(text)
                    .map(|b| (l[..b].chars().count() as u16, y as u16))
            })
            .unwrap_or_else(|| panic!("{text:?} not on screen:\n{}", self.screen))
    }

    pub(crate) fn click(&mut self, x: u16, y: u16) {
        let target = self.hits.hit_test(x, y).expect("a target").clone();
        self.focus.pointer_down(&target, x, y);
        self.render();
    }
}

fn field(value: State<Option<Date>>, order: DateOrder) -> impl Fn() -> VNode {
    move || {
        view! {
            <DateField bind:value={value} order=order label="Due" max={Some(date(2026, 12, 31))} />
        }
        .into_node()
    }
}

#[test]
fn typing_fills_each_segment_in_turn() {
    let value = state(None);
    let mut f = Harness::new(field(value, DateOrder::Ymd));
    assert_eq!(f.line(), "yyyy-mm-dd");
    f.key(KeyCode::Tab);
    f.typing("2026");
    assert_eq!(f.focused().label, "month", "four year digits move on");
    // No month starts with 2, so it's finished at once.
    f.typing("2");
    assert_eq!(f.focused().label, "day");
    assert_eq!(value.get(), None, "still missing the day");
    f.typing("13");
    assert_eq!(f.line(), "2026-02-13");
    assert_eq!(value.get(), Some(TODAY));
}

#[test]
fn a_digit_that_does_not_fit_starts_the_segment_over() {
    let value = state(Some(date(2026, 1, 5)));
    let mut f = Harness::new(field(value, DateOrder::Ymd));
    f.key(KeyCode::Tab);
    f.key(KeyCode::Right);
    // 1 could still become 10 to 12; 15 can't, so the 5 starts again.
    f.typing("1");
    assert_eq!(f.focused().label, "month");
    f.typing("5");
    assert_eq!(f.line(), "2026-05-05");
    assert_eq!(f.focused().label, "day");
}

#[test]
fn arrows_step_and_wrap_and_an_empty_segment_starts_at_today() {
    let value = state(Some(date(2026, 1, 31)));
    let mut f = Harness::new(field(value, DateOrder::Ymd));
    f.key(KeyCode::Tab);
    f.key(KeyCode::Right);
    f.key(KeyCode::Up);
    assert_eq!(value.get(), Some(date(2026, 2, 28)), "the day follows");
    f.key(KeyCode::Down);
    f.key(KeyCode::Down);
    assert_eq!(value.get(), Some(date(2026, 12, 28)), "wraps round");

    f.key(KeyCode::Right);
    f.key(KeyCode::Backspace);
    assert_eq!(f.line(), "2026-12-dd");
    assert_eq!(value.get(), None);
    f.key(KeyCode::Up);
    assert_eq!(value.get(), Some(date(2026, 12, 13)), "today's day");
}

#[test]
fn other_orders_rearrange_the_segments() {
    let value = state(Some(TODAY));
    let mut f = Harness::new(field(value, DateOrder::Dmy));
    assert_eq!(f.line(), "13/02/2026");
    f.key(KeyCode::Tab);
    assert_eq!(f.focused().label, "day");
    f.key(KeyCode::Right);
    f.key(KeyCode::Right);
    assert_eq!(f.focused().label, "year");
    f.key(KeyCode::Left);
    assert_eq!(f.focused().label, "month");
}

#[test]
fn a_new_value_from_outside_replaces_the_segments() {
    let value = state(None);
    let mut f = Harness::new(field(value, DateOrder::Ymd));
    f.key(KeyCode::Tab);
    f.typing("20");
    value.set(Some(date(2025, 7, 4)));
    f.render();
    assert_eq!(f.line(), "2025-07-04");
}

#[test]
fn a_date_past_max_is_marked_invalid() {
    let value = state(Some(date(2027, 1, 1)));
    let f = Harness::new(field(value, DateOrder::Ymd));
    assert_eq!(f.line(), "2027-01-01 !");
    let nodes = f.tree.nodes();
    let group = nodes
        .iter()
        .find(|n| n.interaction.semantics.role == Role::Group)
        .expect("a group");
    assert_eq!(group.interaction.semantics.label, "Due");
    assert_eq!(group.interaction.semantics.invalid, Some(true));
}

#[test]
fn each_segment_is_a_spin_button() {
    let value = state(Some(TODAY));
    let mut f = Harness::new(field(value, DateOrder::Ymd));
    f.key(KeyCode::Tab);
    f.key(KeyCode::Tab);
    let month = f.focused();
    assert_eq!(month.role, Role::SpinButton);
    assert_eq!(month.value, Some(2.0));
    assert_eq!((month.value_min, month.value_max), (Some(1.0), Some(12.0)));
    assert_eq!(month.value_text.as_deref(), Some("February"));
    f.key(KeyCode::Tab);
    assert_eq!(f.focused().value_max, Some(28.0), "February's length");
}

#[test]
fn alt_down_asks_to_open() {
    let opened = state(0);
    let mut f = Harness::new(move || {
        view! { <DateField on_open={move || opened.update(|n| *n += 1)} /> }.into_node()
    });
    f.key(KeyCode::Tab);
    f.key_with(KeyCode::Down, KeyModifiers::ALT);
    assert_eq!(opened.get(), 1);
}
