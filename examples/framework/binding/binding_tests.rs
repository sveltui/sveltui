use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

/// Renders until focus settles and returns the screen as text.
fn render(manager: &mut FocusManager) -> String {
    for _ in 0..8 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <BindingDemo /> });
        end_frame();
        let mut buffer = Buffer::new(80, 40);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 80, 40), &mut buffer, &mut hits)
        });
        if !manager.reconcile(hits.interactions) {
            return (0..buffer.height)
                .map(|y| {
                    (0..buffer.width)
                        .map(|x| buffer.get(x, y).unwrap().ch)
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
    panic!("binding example focus did not settle");
}

fn press(manager: &mut FocusManager, code: KeyCode) -> String {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    render(manager)
}

fn type_text(manager: &mut FocusManager, text: &str) -> String {
    let mut screen = String::new();
    for ch in text.chars() {
        screen = press(manager, KeyCode::Char(ch));
    }
    screen
}

#[test]
fn bound_inputs_share_their_state() {
    let mut manager = FocusManager::new();
    render(&mut manager);

    press(&mut manager, KeyCode::Tab);
    let screen = type_text(&mut manager, "Ada");
    assert!(screen.contains(r#"name: "Ada""#));
    assert_eq!(
        screen.matches("║ Ada").count(),
        1,
        "the focused input shows it"
    );
    assert_eq!(
        screen.matches("│ Ada").count(),
        1,
        "the other bound input follows"
    );

    press(&mut manager, KeyCode::Tab);
    let screen = type_text(&mut manager, "!");
    assert!(
        screen.contains(r#"name: "Ada!""#),
        "editing the second input updates both"
    );
}

#[test]
fn a_component_passes_its_binding_on() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    for _ in 0..3 {
        press(&mut manager, KeyCode::Tab);
    }
    let screen = type_text(&mut manager, "Lovelace");
    assert!(screen.contains(r#"nickname: "Lovelace""#));
}

#[test]
fn a_read_only_input_shows_what_the_parent_decides() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    for _ in 0..4 {
        press(&mut manager, KeyCode::Tab);
    }
    let screen = type_text(&mut manager, "hey");
    assert!(screen.contains(r#"shout: "HEY""#));
    assert!(
        screen.contains("║ HEY"),
        "the input shows the upper-cased text"
    );
}
