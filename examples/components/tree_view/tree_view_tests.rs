use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer, Runtime,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

const WIDTH: u16 = 100;
const HEIGHT: u16 = 24;

/// Renders until focus holds still and two frames look the same.
fn render(manager: &mut FocusManager) -> String {
    let mut last = String::new();
    for _ in 0..10 {
        Runtime::set_dirty(false);
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <TreeViewDemo /> });
        end_frame();
        let mut buffer = Buffer::new(WIDTH, HEIGHT);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(
                &node,
                Rect::new(0, 0, WIDTH, HEIGHT),
                &mut buffer,
                &mut hits,
            )
        });
        let changed = manager.reconcile(hits.interactions);
        let screen = (0..HEIGHT)
            .map(|y| {
                (0..WIDTH)
                    .map(|x| buffer.get(x, y).unwrap().ch)
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        if !changed && screen == last {
            return screen;
        }
        last = screen;
    }
    panic!("the example did not settle");
}

fn press(manager: &mut FocusManager, code: KeyCode) -> String {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    render(manager)
}

#[test]
fn the_file_tree_reads_directories_as_they_open() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen.contains("▸ sveltui-components"), "{screen}");
    assert!(!screen.contains("Cargo.toml"), "{screen}");

    // "sveltui" sorts first among the crates; open it and go in.
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Right);
    let screen = press(&mut manager, KeyCode::Right);
    assert!(screen.contains("Cargo.toml"), "{screen}");
    assert!(screen.contains("▸ src"), "{screen}");

    press(&mut manager, KeyCode::End);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Opened "), "{screen}");
}

#[test]
fn the_settings_tree_is_written_as_parts() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Right);
    press(&mut manager, KeyCode::Right);
    let screen = press(&mut manager, KeyCode::Char(' '));
    assert!(screen.contains("Setting: editor.font"), "{screen}");
}
