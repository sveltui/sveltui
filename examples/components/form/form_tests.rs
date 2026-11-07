use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

fn render(manager: &mut FocusManager) -> Buffer {
    for _ in 0..5 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <FormDemo /> });
        end_frame();
        let mut buffer = Buffer::new(80, 40);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 80, 40), &mut buffer, &mut hits);
        });
        if !manager.reconcile(hits.interactions) {
            return buffer;
        }
    }
    panic!("form focus did not stabilize");
}

#[test]
fn biography_keeps_typed_input_after_render() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    let biography = manager
        .tree()
        .nodes()
        .iter()
        .find(|node| node.interaction.semantics.label == "Biography")
        .expect("biography must be focusable")
        .id;
    manager.focus(biography);
    manager.dispatch(KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL));
    manager.dispatch(KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE));
    let buffer = render(&mut manager);
    let screen: String = buffer.cells.iter().map(|cell| cell.ch).collect();
    assert!(screen.contains("Enjoys declarative syntax.!"), "{screen}");
}
