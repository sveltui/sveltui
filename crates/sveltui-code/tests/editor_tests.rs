use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::Buffer;
use sveltui::element::{render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui_code::prelude::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn render(manager: &mut FocusManager, build: impl Fn() -> VNode) -> (Buffer, HitMap) {
    for _ in 0..5 {
        reset_focus_counter();
        begin_frame();
        let tree = manager.with_context(&build);
        end_frame();
        let mut buffer = Buffer::new(80, 24);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(&tree, Rect::new(0, 0, 80, 24), &mut buffer, &mut hits)
        });
        let changed = manager.reconcile(hits.interactions.clone());
        let revealed = manager.focused().is_some_and(|id| hits.reveal(id));
        if !changed && !revealed {
            return (buffer, hits);
        }
    }
    panic!("focus did not stabilize");
}

#[test]
fn test_code_editor_autofocus_and_typing() {
    let mut manager = FocusManager::new();
    let (code, set_code) = create_signal(String::new());

    let build = move || {
        view! {
            <CodeEditor
                value=code
                set_value=set_code
                language="rs"
                autofocus=true
            />
        }
    };

    render(&mut manager, build);

    // Because autofocus=true, the editor should be focused immediately!
    assert!(manager.focused().is_some());

    // Type "fn " directly without needing Tab
    manager.dispatch(key(KeyCode::Char('f')));
    manager.dispatch(key(KeyCode::Char('n')));
    manager.dispatch(key(KeyCode::Char(' ')));

    assert_eq!(code.get(), "fn ");

    // Type "{", then Enter -> test auto-indent
    manager.dispatch(key(KeyCode::Char('{')));
    manager.dispatch(key(KeyCode::Enter));

    assert_eq!(code.get(), "fn {\n    ");

    // Test typing inside indentation
    manager.dispatch(key(KeyCode::Char('1')));
    assert_eq!(code.get(), "fn {\n    1");

    // Test backspace
    manager.dispatch(key(KeyCode::Backspace));
    assert_eq!(code.get(), "fn {\n    ");

    // Test smart backspace unindenting 4 spaces
    manager.dispatch(key(KeyCode::Backspace));
    assert_eq!(code.get(), "fn {\n");

    // Test undo
    manager.dispatch(ctrl_key('z'));
    assert_eq!(code.get(), "fn {\n    ");
}

#[test]
fn test_code_buffer_standalone_operations() {
    let mut buf = CodeBuffer::new("fn main() {\n    let mut x = 1;\n}");
    assert_eq!(buf.line_count(), 3);
    assert_eq!(buf.cursor_pos(), (2, 1));

    // Move to end of line 1
    buf.set_cursor_pos(1, 18);
    buf.insert_newline();
    assert_eq!(buf.line_count(), 4);
    // Indent of line 1 was 4 spaces, so new line should have 4 spaces
    assert_eq!(buf.line(2), Some("    "));

    // Tab key inserts 4 spaces
    buf.insert_tab();
    assert_eq!(buf.line(2), Some("        "));

    // Undo restores state
    assert!(buf.undo());
    assert_eq!(buf.line(2), Some("    "));
}
