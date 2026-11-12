use super::*;
use crate::AlertDialog;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::Runtime;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const WIDTH: u16 = 70;
const HEIGHT: u16 = 20;

/// A trigger and a dialog with a title, a description, Cancel and Action.
/// `log` records which answer ran.
fn dialog(
    open: State<bool>,
    close_on_escape: bool,
    outside_press: OutsidePress,
    log: State<Vec<&'static str>>,
) -> impl Fn() -> VNode {
    move || {
        view! {
            <AlertDialog::Root bind:open={open} close_on_escape=close_on_escape
                outside_press=outside_press>
                <AlertDialog::Trigger label="Delete branch" />
                <AlertDialog::Content>
                    <AlertDialog::Title>"Delete feature/toast?"</AlertDialog::Title>
                    <AlertDialog::Description>"Its 2 unmerged commits will be lost."</AlertDialog::Description>
                    <View flex_direction=FlexDirection::Row gap=1>
                        <AlertDialog::Cancel label="Keep it"
                            on_click={move || log.update(|l| l.push("cancel"))} />
                        <AlertDialog::Action label="Delete"
                            on_click={move || log.update(|l| l.push("delete"))} />
                    </View>
                </AlertDialog::Content>
            </AlertDialog::Root>
        }
        .into_node()
    }
}

struct Harness {
    focus: FocusManager,
    view: Box<dyn Fn() -> VNode>,
    hits: HitMap,
    tree: InteractionTree,
    screen: String,
}

impl Harness {
    fn new(view: impl Fn() -> VNode + 'static) -> Self {
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

    fn render(&mut self) {
        for _ in 0..10 {
            // Cleared so this thread's writes don't keep other tests'
            // harnesses waiting; we settle on what we see, not on the flag.
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
            // Settled once focus holds still and two frames look the same.
            // The dirty flag is shared by every test thread, so other tests
            // keep setting it; we don't wait on it.
            if !changed && screen == self.screen {
                return;
            }
            self.screen = screen;
        }
        panic!("the dialog did not settle");
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.focus.dispatch(KeyEvent::new(code, modifiers));
        self.render();
    }

    fn press(&mut self, code: KeyCode) {
        self.key(code, KeyModifiers::NONE);
    }

    /// Clicks the top-left cell, which is backdrop while the dialog is open.
    fn click_backdrop(&mut self) {
        let target = self.hits.hit_test(0, 0).expect("the backdrop").clone();
        self.focus.pointer_down(&target, 0, 0);
        self.render();
    }

    /// The label of the focused button: the lowest one drawn with a double
    /// border. (The trigger keeps its double border while its dialog is
    /// open, and the dialog sits below it.)
    fn focused_label(&self) -> Option<String> {
        self.screen.lines().rev().find_map(|line| {
            let start = line.find('║')?;
            let rest = &line[start + '║'.len_utf8()..];
            let end = rest.find('║')?;
            Some(rest[..end].trim().to_owned())
        })
    }
}

/// Opens the dialog from its trigger, the way a user would.
fn opened(
    close_on_escape: bool,
    outside_press: OutsidePress,
) -> (Harness, State<bool>, State<Vec<&'static str>>) {
    let open = state(false);
    let log = state(Vec::new());
    let mut harness = Harness::new(dialog(open, close_on_escape, outside_press, log));
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert!(open.get());
    (harness, open, log)
}

#[test]
fn opening_focuses_cancel_and_tab_stays_inside() {
    let (mut harness, _, _) = opened(true, OutsidePress::Ignore);
    assert!(
        harness.screen.contains("Delete feature/toast?"),
        "{}",
        harness.screen
    );
    assert_eq!(harness.focused_label().as_deref(), Some("Keep it"));

    harness.press(KeyCode::Tab);
    assert_eq!(harness.focused_label().as_deref(), Some("Delete"));
    harness.press(KeyCode::Tab);
    assert_eq!(harness.focused_label().as_deref(), Some("Keep it"));
    harness.key(KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(harness.focused_label().as_deref(), Some("Delete"));
}

#[test]
fn escape_cancels_and_focus_goes_back_to_the_trigger() {
    let (mut harness, open, log) = opened(true, OutsidePress::Ignore);
    harness.press(KeyCode::Esc);
    assert!(!open.get());
    assert_eq!(log.get(), ["cancel"]);
    assert_eq!(harness.focused_label().as_deref(), Some("Delete branch"));
}

#[test]
fn escape_can_be_turned_off() {
    let (mut harness, open, log) = opened(false, OutsidePress::Ignore);
    harness.press(KeyCode::Esc);
    assert!(open.get());
    assert!(log.get().is_empty());
}

#[test]
fn the_action_runs_then_closes() {
    let (mut harness, open, log) = opened(true, OutsidePress::Ignore);
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert!(!open.get());
    assert_eq!(log.get(), ["delete"]);
}

#[test]
fn a_press_outside_does_what_it_is_set_to() {
    let (mut harness, open, log) = opened(true, OutsidePress::Ignore);
    harness.click_backdrop();
    assert!(open.get());

    let (mut harness, open, _) = opened(true, OutsidePress::Bell);
    Runtime::take_bell();
    harness.click_backdrop();
    assert!(open.get());
    assert!(Runtime::take_bell());

    let (mut harness, open, log_close) = opened(true, OutsidePress::Close);
    harness.click_backdrop();
    assert!(!open.get());
    assert_eq!(log_close.get(), ["cancel"]);
    assert!(log.get().is_empty());
}

#[test]
fn the_dialog_is_labelled_and_described_by_its_parts() {
    let (harness, _, _) = opened(true, OutsidePress::Ignore);
    let nodes = harness.tree.nodes();
    let dialog = nodes
        .iter()
        .find(|n| n.interaction.semantics.role == Role::AlertDialog)
        .expect("an alertdialog");
    let semantics = &dialog.interaction.semantics;
    assert_eq!(semantics.modal, Some(true));
    let title = semantics.labelled_by.expect("labelled");
    let description = semantics.described_by.expect("described");
    let role_of = |id| {
        nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.interaction.semantics.role)
    };
    assert_eq!(role_of(title), Some(Role::Heading));
    assert_eq!(role_of(description), Some(Role::Generic));
}
