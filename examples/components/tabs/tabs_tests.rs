use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

fn render(manager: &mut FocusManager) -> String {
    for _ in 0..8 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <RavenTabs /> });
        end_frame();
        let mut buffer = Buffer::new(90, 44);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 90, 44), &mut buffer, &mut hits)
        });
        if !manager.reconcile(hits.interactions) {
            return (0..buffer.height)
                .map(|y| {
                    (0..buffer.width)
                        .map(|x| buffer.get(x, y).unwrap().ch)
                        .collect::<String>()
                        .trim_end()
                        .to_owned()
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
    panic!("tabs example focus did not stabilize");
}

fn press(manager: &mut FocusManager, code: KeyCode) -> String {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    render(manager)
}

#[test]
fn arrows_switch_between_parts_of_the_poem() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    // Stanza II opens part one; it only shows while part one is selected.
    assert!(screen.contains("Ah, distinctly I remember"));
    assert!(!screen.contains("Then this ebony bird beguiling"));

    press(&mut manager, KeyCode::Tab);
    let screen = press(&mut manager, KeyCode::Right);
    assert!(!screen.contains("Ah, distinctly I remember"));
    assert!(screen.contains("Then this ebony bird beguiling"));

    // The disabled "Notes" tab is skipped: Right twice wraps to part one.
    press(&mut manager, KeyCode::Right);
    let screen = press(&mut manager, KeyCode::Right);
    assert!(screen.contains("Ah, distinctly I remember"));
}
