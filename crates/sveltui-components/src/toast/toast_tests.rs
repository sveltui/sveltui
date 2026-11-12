use super::*;
use crate::Toast;
use crate::{Button, ButtonProps};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use sveltui::Runtime;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const WIDTH: u16 = 60;
const HEIGHT: u16 = 16;
const BUTTON: FocusId = FocusId(810_001);

/// A clock the test moves by hand.
fn clock(manager: &ToastManager) -> Rc<Cell<Instant>> {
    let now = Rc::new(Cell::new(Instant::now()));
    let read = now.clone();
    manager.set_clock(move || read.get());
    now
}

fn advance(now: &Cell<Instant>, by: Duration) {
    now.set(now.get() + by);
}

fn closed_log(options: ToastOptions, log: &Rc<RefCell<Vec<String>>>) -> ToastOptions {
    let log = log.clone();
    let title = options.title.clone();
    options.on_close(move || log.borrow_mut().push(title.clone()))
}

#[test]
fn the_list_is_newest_first_and_the_limit_closes_the_oldest() {
    let manager = ToastManager::new();
    manager.configure(Duration::from_secs(5), 2);
    let log = Rc::new(RefCell::new(Vec::new()));
    manager.add(closed_log(ToastOptions::new("one"), &log));
    let two = manager.add(closed_log(ToastOptions::new("two"), &log));
    manager.add(closed_log(ToastOptions::new("three"), &log));
    let titles: Vec<String> = manager.list().iter().map(|t| t.title().into()).collect();
    assert_eq!(titles, ["three", "two"]);
    assert_eq!(*log.borrow(), ["one"]);

    manager.update(two, ToastOptions::new("two, again"));
    assert_eq!(manager.list()[1].title(), "two, again");
    manager.close(two);
    manager.close(two);
    assert_eq!(manager.list().len(), 1);
}

#[test]
fn a_toast_closes_when_its_time_is_up_and_asks_for_that_frame() {
    let manager = ToastManager::new();
    let now = clock(&manager);
    let log = Rc::new(RefCell::new(Vec::new()));
    manager.add(closed_log(ToastOptions::new("saved"), &log));
    manager.add(ToastOptions::new("sticky").timeout(Duration::ZERO));
    manager.tick();
    assert_eq!(
        Runtime::next_wake().map(|at| at <= now.get() + Duration::from_secs(5)),
        Some(true)
    );

    advance(&now, Duration::from_millis(4_999));
    manager.tick();
    assert_eq!(manager.list().len(), 2);
    advance(&now, Duration::from_millis(1));
    manager.tick();
    assert_eq!(*log.borrow(), ["saved"]);

    advance(&now, Duration::from_secs(3600));
    manager.tick();
    assert_eq!(manager.list()[0].title(), "sticky");
}

#[test]
fn pausing_keeps_the_time_that_was_left() {
    let manager = ToastManager::new();
    let now = clock(&manager);
    manager.add(ToastOptions::new("saved"));
    manager.tick();
    advance(&now, Duration::from_secs(2));
    manager.set_paused(true);
    advance(&now, Duration::from_secs(60));
    manager.tick();
    assert_eq!(manager.list().len(), 1, "paused toasts don't time out");

    manager.set_paused(false);
    advance(&now, Duration::from_millis(2_999));
    manager.tick();
    assert_eq!(manager.list().len(), 1);
    advance(&now, Duration::from_millis(1));
    manager.tick();
    assert!(manager.is_empty());
}

/// Renders a provider holding `manager`, with a button to start focus on,
/// until focus settles, like `App` does.
struct Harness {
    focus: FocusManager,
    view: Box<dyn Fn() -> VNode>,
    hits: HitMap,
    rows: Vec<String>,
}

impl Harness {
    fn new(manager: &ToastManager) -> Self {
        let manager = manager.clone();
        let mut harness = Self {
            focus: FocusManager::new(),
            view: Box::new(move || {
                view! {
                    <Toast::Provider manager={manager.clone()}>
                        <Button id=BUTTON label="Save" />
                        <Toast::Viewport placement=Placement::BottomEnd width=30>
                            {#each use_toast_manager().list() as toast}
                                <Toast::Root toast={toast.clone()} />
                            {/each}
                        </Toast::Viewport>
                    </Toast::Provider>
                }
                .into_node()
            }),
            hits: HitMap::new(),
            rows: Vec::new(),
        };
        harness.render();
        harness.key(KeyCode::Tab);
        harness
    }

    fn render(&mut self) -> InteractionTree {
        for _ in 0..10 {
            Runtime::set_dirty(false);
            reset_focus_counter();
            begin_frame();
            let node = self.focus.with_context(|| (self.view)());
            end_frame();
            let tree = InteractionTree::from_node(&node);
            let mut buffer = Buffer::new(WIDTH, HEIGHT);
            self.hits = HitMap::new();
            let hits = &mut self.hits;
            self.focus.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, WIDTH, HEIGHT), &mut buffer, hits)
            });
            let changed = self.focus.reconcile(self.hits.interactions.clone());
            if !changed && !Runtime::is_dirty() {
                self.rows = (0..HEIGHT)
                    .map(|y| (0..WIDTH).map(|x| buffer.get(x, y).unwrap().ch).collect())
                    .collect();
                return tree;
            }
        }
        panic!("the toasts did not settle");
    }

    fn key(&mut self, code: KeyCode) -> InteractionTree {
        self.focus.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
        self.render()
    }

    fn screen(&self) -> String {
        self.rows.join("\n")
    }
}

/// The role, label, live setting and atomic flag of each toast on screen.
fn toasts(tree: &InteractionTree) -> Vec<(Role, String, Option<Live>, Option<bool>)> {
    tree.nodes()
        .iter()
        .map(|n| &n.interaction.semantics)
        .filter(|s| matches!(s.role, Role::Status | Role::Alert))
        .map(|s| (s.role, s.label.clone(), s.live, s.atomic))
        .collect()
}

#[test]
fn toasts_show_in_the_corner_and_announce_by_priority() {
    let manager = ToastManager::new();
    let mut harness = Harness::new(&manager);
    manager.add(ToastOptions::new("Saved").description("main.rs"));
    manager.add(ToastOptions::new("Disk full").priority(ToastPriority::High));
    let tree = harness.render();

    assert_eq!(
        toasts(&tree),
        [
            (
                Role::Alert,
                "Disk full".to_owned(),
                Some(Live::Assertive),
                Some(true)
            ),
            (
                Role::Status,
                "Saved. main.rs".to_owned(),
                Some(Live::Polite),
                Some(true)
            ),
        ]
    );
    let screen = harness.screen();
    assert!(screen.contains("Saved"), "{screen}");
    assert!(screen.contains("Disk full"), "{screen}");
    // Bottom-right: the newest sits on the bottom edge, at the right.
    let last = harness.rows[usize::from(HEIGHT) - 2].clone();
    assert!(last.contains("Disk full"), "{screen}");
    assert!(last.ends_with('│'), "{screen}");
}

#[test]
fn the_hotkey_focuses_the_newest_toast_and_escape_goes_back() {
    let manager = ToastManager::new();
    let mut harness = Harness::new(&manager);
    manager.add(ToastOptions::new("older"));
    let newest = manager.add(ToastOptions::new("newest"));
    harness.render();
    assert_eq!(harness.focus.focused(), Some(BUTTON));

    harness.key(KeyCode::F(8));
    let focused = manager.list()[0].focus_id;
    assert_eq!(manager.list()[0].id, newest);
    assert_eq!(harness.focus.focused(), Some(focused));

    harness.key(KeyCode::Esc);
    let titles: Vec<String> = manager.list().iter().map(|t| t.title().into()).collect();
    assert_eq!(titles, ["older"]);
    assert_eq!(harness.focus.focused(), Some(BUTTON));
}

#[test]
fn the_close_button_and_the_action_close_the_toast() {
    let manager = ToastManager::new();
    let mut harness = Harness::new(&manager);
    let undone = Rc::new(Cell::new(false));
    let flag = undone.clone();
    manager.add(ToastOptions::new("Deleted").action("Undo", move || flag.set(true)));
    harness.render();
    assert!(harness.screen().contains("[Undo]"), "{}", harness.screen());

    // Into the toast, then Tab to its action.
    harness.key(KeyCode::F(8));
    harness.key(KeyCode::Tab);
    harness.key(KeyCode::Enter);
    assert!(undone.get());
    assert!(manager.is_empty());
    assert_eq!(harness.focus.focused(), Some(BUTTON));

    // Without an action, the close button is the next tab stop.
    manager.add(ToastOptions::new("Saved"));
    harness.render();
    harness.key(KeyCode::F(8));
    harness.key(KeyCode::Tab);
    harness.key(KeyCode::Char(' '));
    assert!(manager.is_empty());
}

#[test]
fn hovering_the_viewport_pauses_the_countdown() {
    let manager = ToastManager::new();
    let now = clock(&manager);
    let mut harness = Harness::new(&manager);
    manager.add(ToastOptions::new("Saved"));
    harness.render();

    let row = harness
        .rows
        .iter()
        .position(|r| r.contains("Saved"))
        .unwrap() as u16;
    let column = harness.rows[usize::from(row)].find("Saved").unwrap() as u16;
    harness.focus.pointer_move(&harness.hits, column, row);
    harness.render();
    advance(&now, Duration::from_secs(60));
    harness.render();
    assert!(!manager.is_empty(), "hovered toasts stay up");

    harness.focus.pointer_move(&harness.hits, 0, HEIGHT - 1);
    harness.render();
    advance(&now, Duration::from_secs(5));
    harness.render();
    assert!(manager.is_empty());
}
