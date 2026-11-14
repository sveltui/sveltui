use super::*;
use crate::clock::TEST_TODAY;
use crate::date::{Date, Weekday};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use jiff::civil::date;
use std::collections::HashSet;
use sveltui::Runtime;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const WIDTH: u16 = 30;
const HEIGHT: u16 = 10;
/// A Friday.
const TODAY: Date = date(2026, 2, 13);

struct Harness {
    focus: FocusManager,
    view: Box<dyn Fn() -> VNode>,
    hits: HitMap,
    tree: InteractionTree,
    buffer: Buffer,
    screen: String,
}

impl Harness {
    fn new(view: impl Fn() -> VNode + 'static) -> Self {
        TEST_TODAY.with(|t| t.set(Some(TODAY)));
        let mut harness = Self {
            focus: FocusManager::new(),
            view: Box::new(view),
            hits: HitMap::new(),
            tree: InteractionTree::default(),
            buffer: Buffer::new(WIDTH, HEIGHT),
            screen: String::new(),
        };
        harness.render();
        harness
    }

    /// Renders until focus holds still and two frames look the same.
    fn render(&mut self) {
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
            self.buffer = buffer;
            if !changed && screen == self.screen {
                return;
            }
            self.screen = screen;
        }
        panic!("the calendar did not settle");
    }

    fn key_with(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.focus.dispatch(KeyEvent::new(code, modifiers));
        self.render();
    }

    fn key(&mut self, code: KeyCode) {
        self.key_with(code, KeyModifiers::NONE);
    }

    /// Tabs past the previous-month button onto the grid.
    fn focus_grid(&mut self) {
        while !self.cursor_shown() {
            self.key(KeyCode::Tab);
        }
    }

    fn cursor_shown(&self) -> bool {
        self.buffer
            .cells
            .iter()
            .any(|c| c.bg == Some(Color::White) && c.ch.is_ascii_digit())
    }

    /// The day number under the cursor (drawn inverted).
    fn cursor_day(&self) -> u32 {
        let text: String = self
            .buffer
            .cells
            .iter()
            .filter(|c| c.bg == Some(Color::White))
            .map(|c| c.ch)
            .collect();
        text.trim()
            .parse()
            .unwrap_or_else(|_| panic!("no cursor:\n{}", self.screen))
    }

    fn heading(&self) -> String {
        self.screen
            .lines()
            .next()
            .unwrap()
            .replace(['[', ']', '‹', '›'], "")
            .trim()
            .to_owned()
    }

    fn cells(&self) -> Vec<Semantics> {
        self.tree
            .nodes()
            .iter()
            .map(|n| n.interaction.semantics.clone())
            .filter(|s| s.role == Role::Cell)
            .collect()
    }

    fn click(&mut self, x: u16, y: u16) {
        let target = self.hits.hit_test(x, y).expect("a target").clone();
        self.focus.pointer_down(&target, x, y);
        self.render();
    }

    /// Where `text` is on screen, searching rows from `from_row`.
    fn find(&self, text: &str, from_row: usize) -> (u16, u16) {
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
}

fn calendar(value: State<Option<Date>>, picked: State<Vec<Date>>) -> impl Fn() -> VNode {
    move || {
        view! {
            <Calendar bind:value={value} on_select={move |d: Date| picked.update(|p| p.push(d))}
                is_date_unavailable={|d: Date| d == date(2026, 2, 20)} />
        }
        .into_node()
    }
}

fn opened() -> (Harness, State<Option<Date>>, State<Vec<Date>>) {
    let value = state(None);
    let picked = state(Vec::new());
    let mut harness = Harness::new(calendar(value, picked));
    harness.focus_grid();
    (harness, value, picked)
}

#[test]
fn it_opens_on_today_and_arrows_move_by_day_and_week() {
    let (mut c, _, _) = opened();
    assert_eq!(c.heading(), "February 2026");
    assert_eq!(c.cursor_day(), 13);
    c.key(KeyCode::Right);
    assert_eq!(c.cursor_day(), 14);
    c.key(KeyCode::Down);
    assert_eq!(c.cursor_day(), 21);
    c.key(KeyCode::Up);
    c.key(KeyCode::Up);
    c.key(KeyCode::Left);
    assert_eq!(c.cursor_day(), 6);
}

#[test]
fn pages_move_months_and_years_and_cross_into_them() {
    let value = state(Some(date(2026, 1, 31)));
    let mut c = Harness::new(calendar(value, state(Vec::new())));
    c.focus_grid();
    assert_eq!(c.heading(), "January 2026");
    c.key(KeyCode::PageDown);
    assert_eq!(
        (c.heading().as_str(), c.cursor_day()),
        ("February 2026", 28)
    );
    c.key_with(KeyCode::PageDown, KeyModifiers::SHIFT);
    assert_eq!(c.heading(), "February 2027");
    c.key_with(KeyCode::PageUp, KeyModifiers::SHIFT);
    c.key(KeyCode::PageUp);
    assert_eq!((c.heading().as_str(), c.cursor_day()), ("January 2026", 28));

    // Walking off the end of the month turns the page.
    c.key(KeyCode::Down);
    c.key(KeyCode::Down);
    assert_eq!(
        (c.heading().as_str(), c.cursor_day()),
        ("February 2026", 11)
    );
}

#[test]
fn home_and_end_follow_the_start_of_the_week() {
    let (mut c, _, _) = opened();
    c.key(KeyCode::Home);
    assert_eq!(c.cursor_day(), 9, "Monday");
    c.key(KeyCode::End);
    assert_eq!(c.cursor_day(), 15, "Sunday");
}

// Its own test, so its own thread: hook state is kept per thread, and a
// second calendar on the same thread would start where the first left off.
#[test]
fn weeks_can_start_on_sunday() {
    let mut c = Harness::new(|| view! { <Calendar week_starts_on=Weekday::Sunday /> }.into_node());
    c.focus_grid();
    c.key(KeyCode::Home);
    assert_eq!(c.cursor_day(), 8, "Sunday");
    assert!(
        c.screen
            .lines()
            .nth(1)
            .unwrap()
            .trim_start()
            .starts_with("Su")
    );
}

#[test]
fn min_and_max_stop_the_cursor() {
    let mut c = Harness::new(|| {
        view! { <Calendar min={Some(date(2026, 2, 10))} max={Some(date(2026, 2, 16))} /> }
            .into_node()
    });
    c.focus_grid();
    c.key(KeyCode::Up);
    assert_eq!(c.cursor_day(), 10);
    c.key(KeyCode::PageDown);
    assert_eq!(c.cursor_day(), 16);
}

#[test]
fn enter_and_space_pick_but_not_an_unavailable_day() {
    let (mut c, value, picked) = opened();
    c.key(KeyCode::Enter);
    assert_eq!(value.get(), Some(TODAY));
    c.key(KeyCode::Right);
    c.key(KeyCode::Char(' '));
    assert_eq!(value.get(), Some(date(2026, 2, 14)));
    assert_eq!(picked.get(), [TODAY, date(2026, 2, 14)]);

    // The 20th is unavailable: the cursor reaches it, picking does nothing.
    c.key(KeyCode::Down);
    c.key(KeyCode::Left);
    assert_eq!(c.cursor_day(), 20);
    c.key(KeyCode::Enter);
    assert_eq!(value.get(), Some(date(2026, 2, 14)));
}

#[test]
fn clicks_pick_days_and_turn_pages() {
    let (mut c, value, _) = opened();
    let (x, y) = c.find("25", 2);
    c.click(x, y);
    assert_eq!(value.get(), Some(date(2026, 2, 25)));

    // The first row starts with the end of January.
    let (x, y) = c.find("26", 2);
    c.click(x, y);
    assert_eq!(value.get(), Some(date(2026, 1, 26)));
    assert_eq!(c.heading(), "January 2026");

    let (x, y) = c.find("›", 0);
    c.click(x, y);
    assert_eq!(c.heading(), "February 2026");
    let (x, y) = c.find("‹", 0);
    c.click(x, y);
    c.click(x, y);
    assert_eq!(c.heading(), "December 2025");
}

#[test]
fn the_grid_says_what_each_day_is() {
    let (mut c, _, _) = opened();
    c.key(KeyCode::Enter);
    let nodes = c.tree.nodes();
    let grid = nodes
        .iter()
        .find(|n| n.interaction.semantics.role == Role::Grid)
        .expect("a grid");
    let heading = grid.interaction.semantics.labelled_by.expect("labelled");
    let heading = nodes.iter().find(|n| n.id == heading).expect("the heading");
    assert_eq!(heading.interaction.semantics.label, "February 2026");
    assert_eq!(heading.interaction.semantics.live, Some(Live::Polite));

    let headers = nodes
        .iter()
        .filter(|n| n.interaction.semantics.role == Role::ColumnHeader)
        .count();
    assert_eq!(headers, 7);
    let cells = c.cells();
    assert_eq!(cells.len(), 42);
    let today: Vec<_> = cells.iter().filter(|s| s.current == Some(true)).collect();
    assert_eq!(today.len(), 1);
    assert_eq!(today[0].label, "Friday, February 13, 2026");
    assert_eq!(today[0].selected, Some(true));
    assert_eq!((today[0].row, today[0].column), (Some(3), Some(4)));

    let disabled: HashSet<_> = nodes
        .iter()
        .filter(|n| n.interaction.disabled && n.interaction.semantics.role == Role::Cell)
        .map(|n| n.interaction.semantics.label.clone())
        .collect();
    assert_eq!(
        disabled,
        HashSet::from(["Friday, February 20, 2026".to_owned()])
    );
}
