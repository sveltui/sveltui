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
        let node = manager.with_context(|| view! { <Workbench /> });
        end_frame();
        let mut buffer = Buffer::new(100, 30);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 100, 30), &mut buffer, &mut hits)
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
    panic!("resizable example focus did not stabilize");
}

fn press(manager: &mut FocusManager, code: KeyCode) -> String {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    render(manager)
}

#[test]
fn the_sidebar_handle_resizes_with_the_arrow_keys() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen.contains("Sidebar"));
    assert!(screen.contains("Editor"));
    assert!(screen.contains("Terminal"));

    // The first Tab stop is the sidebar handle.
    assert!(screen.contains("25% / 75%"));
    press(&mut manager, KeyCode::Tab);
    let after = press(&mut manager, KeyCode::Right);
    assert!(after.contains("30% / 70%"), "{after}");
    let after = press(&mut manager, KeyCode::Left);
    assert!(after.contains("25% / 75%"), "{after}");
}
