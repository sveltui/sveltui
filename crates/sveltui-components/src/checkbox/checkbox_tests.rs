use crate::*;
use sveltui::element::render_node;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn row(node: &VNode) -> String {
    let mut buffer = Buffer::new(20, 1);
    render_node(node, Rect::new(0, 0, 20, 1), &mut buffer);
    let text: String = (0..20).map(|x| buffer.get(x, 0).unwrap().ch).collect();
    text.trim_end().to_owned()
}

#[test]
fn checkbox_renders_its_binding_and_label() {
    let checked = state(true);
    let node = view! { <Checkbox bind:checked=checked label="Accept" /> };
    assert_eq!(row(&node), "  [x] Accept");

    checked.set(false);
    let node = view! { <Checkbox bind:checked=checked label="Accept" /> };
    assert_eq!(row(&node), "  [ ] Accept");
}

#[test]
fn checkbox_without_binding_renders_unchecked() {
    let node = view! { <Checkbox label="Remember me" /> };
    assert_eq!(row(&node), "  [ ] Remember me");
}

mod bindings {
    use crate::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::{cell::RefCell, rc::Rc};
    use sveltui::element::{render_node_interactive, reset_focus_counter};
    use sveltui::layout::Rect;
    use sveltui::prelude::*;
    use sveltui::terminal::Buffer;

    const ID: FocusId = FocusId(100_101);

    /// Renders until focus settles and returns the first row.
    fn frame(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> String {
        for _ in 0..5 {
            reset_focus_counter();
            begin_frame();
            let node = manager.with_context(build);
            end_frame();
            let mut buffer = Buffer::new(30, 1);
            let mut hits = HitMap::new();
            manager.with_context(|| {
                render_node_interactive(&node, Rect::new(0, 0, 30, 1), &mut buffer, &mut hits)
            });
            if !manager.reconcile(hits.interactions) {
                let row: String = (0..30).map(|x| buffer.get(x, 0).unwrap().ch).collect();
                return row.trim_end().to_owned();
            }
        }
        panic!("checkbox focus did not settle");
    }

    fn toggle(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> String {
        manager.dispatch(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        frame(manager, build)
    }

    #[test]
    fn a_read_only_value_reports_toggles_but_stays_put() {
        let reported = Rc::new(RefCell::new(Vec::new()));
        let build = {
            let reported = reported.clone();
            move || {
                let reported = reported.clone();
                view! {
                    <Checkbox id=ID checked={false} label="Accept"
                        on_checked_change={move |next: bool| reported.borrow_mut().push(next)} />
                }
            }
        };
        let mut manager = FocusManager::new();
        frame(&mut manager, &build);
        manager.focus(ID);
        assert!(toggle(&mut manager, &build).contains("[ ] Accept"));
        assert_eq!(*reported.borrow(), vec![true]);
    }

    #[test]
    fn bind_checked_writes_the_parents_state() {
        let accepted = state(false);
        let build = move || view! { <Checkbox id=ID bind:checked={accepted} label="Accept" /> };
        let mut manager = FocusManager::new();
        frame(&mut manager, &build);
        manager.focus(ID);
        assert!(toggle(&mut manager, &build).contains("[x] Accept"));
        assert!(accepted.get());
    }

    #[test]
    fn default_checked_starts_checked_and_toggles_on_its_own() {
        let build = || view! { <Checkbox id=ID default_checked=true label="Accept" /> };
        let mut manager = FocusManager::new();
        assert!(frame(&mut manager, &build).contains("[x] Accept"));
        manager.focus(ID);
        assert!(toggle(&mut manager, &build).contains("[ ] Accept"));
    }
}
