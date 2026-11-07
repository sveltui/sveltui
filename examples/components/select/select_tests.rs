use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

/// Renders frames until focus settles, always rendering at least twice so
/// labels that depend on the previous frame catch up.
fn render(manager: &mut FocusManager) -> String {
    for frame in 0..10 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <SelectDemo /> });
        end_frame();
        let mut buffer = Buffer::new(90, 40);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 90, 40), &mut buffer, &mut hits)
        });
        if !manager.reconcile(hits.interactions) && frame > 0 {
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
    panic!("example focus did not settle");
}

fn press(manager: &mut FocusManager, code: KeyCode) -> String {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    render(manager)
}

#[test]
fn keyboard_picks_a_fruit_from_a_group() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen.contains("Fruit: none  Size: md"));

    press(&mut manager, KeyCode::Tab);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Berries") && screen.contains("Stone fruit"));
    press(&mut manager, KeyCode::End);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Fruit: peach"));
}
