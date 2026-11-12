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
        let node = manager.with_context(|| view! { <TooltipDemo /> });
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

#[test]
fn focusing_a_tool_shows_its_tip_once_the_delay_passes() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    manager.dispatch(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    let screen = render(&mut manager);
    assert!(!screen.contains("Write the buffer"), "{screen}");

    // The example runs on the real clock, so wait the delay out.
    std::thread::sleep(std::time::Duration::from_millis(650));
    let screen = render(&mut manager);
    assert!(
        screen.contains("Write the buffer to disk (Ctrl+S)"),
        "{screen}"
    );

    // Enter still presses the button underneath.
    manager.dispatch(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let screen = render(&mut manager);
    assert!(screen.contains("Saved."), "{screen}");
}
