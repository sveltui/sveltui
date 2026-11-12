use crate::clock::TEST_NOW;
use crate::{Button, ButtonProps, HoverCard, Tooltip};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use sveltui::Runtime;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

const WIDTH: u16 = 80;
const HEIGHT: u16 = 20;
const OTHER: FocusId = FocusId(820_001);
const SAVE: FocusId = FocusId(820_002);

/// A page with a plain button, a button with a tooltip, and a name with a
/// hover card, rendered against a clock the test moves by hand.
pub(crate) struct Harness {
    pub(crate) focus: FocusManager,
    view: Box<dyn Fn() -> VNode>,
    pub(crate) hits: HitMap,
    pub(crate) tree: InteractionTree,
    pub(crate) screen: String,
}

pub(crate) fn page(clicks: State<u32>) -> impl Fn() -> VNode {
    move || {
        view! {
            <View padding=4 gap=1>
                <View flex_direction=FlexDirection::Row gap=2>
                    <Button id=OTHER label="Other" on_click={move || clicks.update(|n| *n += 1)} />
                    <Tooltip::Root>
                        <Tooltip::Trigger>
                            <Button id=SAVE label="Save" />
                        </Tooltip::Trigger>
                        <Tooltip::Content>"Write to disk"</Tooltip::Content>
                    </Tooltip::Root>
                </View>
                <HoverCard::Root>
                    <HoverCard::Trigger tab_stop=true>
                        <TextSpan>"@ada"</TextSpan>
                    </HoverCard::Trigger>
                    <HoverCard::Content>
                        <TextSpan>"Ada Lovelace"</TextSpan>
                    </HoverCard::Content>
                </HoverCard::Root>
            </View>
        }
        .into_node()
    }
}

impl Harness {
    pub(crate) fn new(view: impl Fn() -> VNode + 'static) -> Self {
        TEST_NOW.with(|now| now.set(Some(Instant::now())));
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
        panic!("the page did not settle");
    }

    pub(crate) fn wait(&mut self, by: Duration) {
        TEST_NOW.with(|now| now.set(now.get().map(|t| t + by)));
        self.render();
    }

    pub(crate) fn key(&mut self, code: KeyCode) {
        self.focus.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
        self.render();
    }

    /// Where `text` is on screen.
    pub(crate) fn find(&self, text: &str) -> (u16, u16) {
        self.screen
            .lines()
            .enumerate()
            .find_map(|(y, line)| {
                line.find(text)
                    .map(|byte| (line[..byte].chars().count() as u16, y as u16))
            })
            .unwrap_or_else(|| panic!("{text:?} is not on screen:\n{}", self.screen))
    }

    pub(crate) fn hover(&mut self, text: &str) {
        let (x, y) = self.find(text);
        self.hover_at(x, y);
    }

    pub(crate) fn hover_at(&mut self, x: u16, y: u16) {
        self.focus.pointer_move(&self.hits, x, y);
        self.render();
    }

    pub(crate) fn click(&mut self, text: &str) {
        let (x, y) = self.find(text);
        let target = self.hits.hit_test(x, y).expect("a target").clone();
        self.focus.pointer_down(&target, x, y);
        self.render();
    }

    pub(crate) fn shows(&self, text: &str) -> bool {
        self.screen.contains(text)
    }
}

const TIP: &str = "Write to disk";
const CARD: &str = "Ada Lovelace";

#[test]
fn hovering_shows_the_tooltip_after_the_delay_and_leaving_hides_it() {
    let mut page = Harness::new(page(state(0)));
    page.hover("Save");
    page.wait(Duration::from_millis(599));
    assert!(!page.shows(TIP), "{}", page.screen);
    page.wait(Duration::from_millis(1));
    assert!(page.shows(TIP), "{}", page.screen);

    page.hover_at(WIDTH - 1, HEIGHT - 1);
    assert!(!page.shows(TIP), "{}", page.screen);
}

#[test]
fn the_pointer_can_move_onto_a_hover_card() {
    let mut page = Harness::new(page(state(0)));
    page.hover("@ada");
    page.wait(Duration::from_millis(600));
    assert!(page.shows(CARD), "{}", page.screen);

    // Off the trigger and onto the card, inside the close delay.
    page.hover_at(WIDTH - 1, HEIGHT - 1);
    page.wait(Duration::from_millis(200));
    page.hover(CARD);
    page.wait(Duration::from_secs(5));
    assert!(page.shows(CARD), "the card stays while it's hovered");

    page.hover_at(WIDTH - 1, HEIGHT - 1);
    page.wait(Duration::from_millis(299));
    assert!(page.shows(CARD));
    page.wait(Duration::from_millis(1));
    assert!(!page.shows(CARD), "{}", page.screen);
}

#[test]
fn focus_shows_the_tooltip_and_keeps_focus_on_the_trigger() {
    let mut page = Harness::new(page(state(0)));
    page.key(KeyCode::Tab);
    page.key(KeyCode::Tab);
    assert_eq!(page.focus.focused(), Some(SAVE));
    page.wait(Duration::from_millis(600));
    assert!(page.shows(TIP), "{}", page.screen);
    assert_eq!(
        page.focus.focused(),
        Some(SAVE),
        "the tooltip never takes focus"
    );

    page.key(KeyCode::Tab);
    assert!(!page.shows(TIP), "{}", page.screen);
}

#[test]
fn escape_hides_it_until_focus_leaves_and_comes_back() {
    let mut page = Harness::new(page(state(0)));
    page.key(KeyCode::Tab);
    page.key(KeyCode::Tab);
    page.wait(Duration::from_millis(600));
    assert!(page.shows(TIP));

    page.key(KeyCode::Esc);
    assert!(!page.shows(TIP));
    assert_eq!(page.focus.focused(), Some(SAVE));
    page.wait(Duration::from_secs(10));
    assert!(!page.shows(TIP), "it stays hidden while focus stays");

    page.key(KeyCode::Tab);
    page.key(KeyCode::BackTab);
    page.wait(Duration::from_millis(600));
    assert!(page.shows(TIP), "{}", page.screen);
}

#[test]
fn clicks_reach_the_page_while_a_tooltip_shows() {
    let clicks = state(0);
    let mut page = Harness::new(page(clicks));
    page.hover("Save");
    page.wait(Duration::from_millis(600));
    assert!(page.shows(TIP));
    page.click("Other");
    assert_eq!(clicks.get(), 1);
}

/// Two buttons with tooltips side by side.
fn toolbar() -> VNode {
    view! {
        <View padding=4 flex_direction=FlexDirection::Row gap=1>
            <Tooltip::Root>
                <Tooltip::Trigger><Button label="Save" /></Tooltip::Trigger>
                <Tooltip::Content>"Write to disk"</Tooltip::Content>
            </Tooltip::Root>
            <Tooltip::Root>
                <Tooltip::Trigger><Button label="Undo" /></Tooltip::Trigger>
                <Tooltip::Content>"Undo it"</Tooltip::Content>
            </Tooltip::Root>
        </View>
    }
    .into_node()
}

#[test]
fn a_clicked_button_keeps_no_tooltip_once_the_pointer_leaves() {
    let mut page = Harness::new(toolbar);
    page.hover("Save");
    page.click("Save");
    page.wait(Duration::from_millis(600));
    assert!(page.shows(TIP), "the pointer is still on it");

    // The click focused Save, but that's not keyboard focus, so moving on
    // to Undo closes Save's tooltip and opens only Undo's.
    page.hover("Undo");
    assert!(!page.shows(TIP), "{}", page.screen);
    page.wait(Duration::from_millis(600));
    assert!(page.shows("Undo it"), "{}", page.screen);
    assert!(!page.shows(TIP), "{}", page.screen);

    page.hover_at(WIDTH - 1, HEIGHT - 1);
    assert!(!page.shows("Undo it"), "{}", page.screen);
}

#[test]
fn tab_after_a_click_moves_the_tooltip_along() {
    let mut page = Harness::new(toolbar);
    page.hover("Save");
    page.click("Save");
    page.hover_at(WIDTH - 1, HEIGHT - 1);
    page.key(KeyCode::Tab);
    page.wait(Duration::from_millis(600));
    assert!(
        page.shows("Undo it"),
        "keyboard focus opens it: {}",
        page.screen
    );
    assert!(!page.shows(TIP), "{}", page.screen);
}

#[test]
fn only_one_tooltip_shows_at_a_time() {
    let mut page = Harness::new(toolbar);
    // Keyboard focus keeps Save's tooltip wanted the whole time.
    page.key(KeyCode::Tab);
    page.wait(Duration::from_millis(600));
    assert!(page.shows(TIP));

    page.hover("Undo");
    page.wait(Duration::from_millis(599));
    assert!(page.shows(TIP), "Save's stays until Undo's is due");
    page.wait(Duration::from_millis(1));
    assert!(page.shows("Undo it"), "{}", page.screen);
    assert!(!page.shows(TIP), "{}", page.screen);

    // Save still has focus, but it doesn't take the slot back.
    page.wait(Duration::from_secs(5));
    assert!(page.shows("Undo it") && !page.shows(TIP), "{}", page.screen);

    // Once the pointer leaves Undo, nothing shows until focus moves.
    page.hover_at(WIDTH - 1, HEIGHT - 1);
    page.wait(Duration::from_secs(5));
    assert!(
        !page.shows("Undo it") && !page.shows(TIP),
        "{}",
        page.screen
    );
    page.key(KeyCode::Tab);
    page.wait(Duration::from_millis(600));
    assert!(page.shows("Undo it") && !page.shows(TIP), "{}", page.screen);
}

#[test]
fn opening_one_from_code_closes_the_other() {
    let save = state(true);
    let undo = state(false);
    let mut page = Harness::new(move || {
        view! {
            <View padding=4 flex_direction=FlexDirection::Row gap=1>
                <Tooltip::Root bind:open={save}>
                    <Tooltip::Trigger><Button label="Save" /></Tooltip::Trigger>
                    <Tooltip::Content>"Write to disk"</Tooltip::Content>
                </Tooltip::Root>
                <Tooltip::Root bind:open={undo}>
                    <Tooltip::Trigger><Button label="Undo" /></Tooltip::Trigger>
                    <Tooltip::Content>"Undo it"</Tooltip::Content>
                </Tooltip::Root>
            </View>
        }
        .into_node()
    });
    assert!(page.shows(TIP));
    undo.set(true);
    page.render();
    assert!(page.shows("Undo it"), "{}", page.screen);
    assert!(!page.shows(TIP) && !save.get(), "{}", page.screen);
}
