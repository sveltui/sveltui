use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer, Runtime,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

const WIDTH: u16 = 90;
const HEIGHT: u16 = 24;

/// Renders until focus settles, and returns the screen.
fn render(manager: &mut FocusManager) -> String {
    for _ in 0..10 {
        Runtime::set_dirty(false);
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <ToastDemo /> });
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
        if !manager.reconcile(hits.interactions) && !Runtime::is_dirty() {
            return (0..HEIGHT)
                .map(|y| {
                    (0..WIDTH)
                        .map(|x| buffer.get(x, y).unwrap().ch)
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
    panic!("the example did not settle");
}

fn press(manager: &mut FocusManager, code: KeyCode) -> String {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    render(manager)
}

#[test]
fn the_banner_dismisses() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen.contains("! Disk almost full"), "{screen}");

    // The banner's dismiss button is the first tab stop.
    press(&mut manager, KeyCode::Tab);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(!screen.contains("Disk almost full"), "{screen}");
}

#[test]
fn buttons_raise_toasts_and_escape_closes_the_newest() {
    let mut manager = FocusManager::new();
    render(&mut manager);

    // Past the banner's dismiss button to "Save".
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Tab);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Saved"), "{screen}");
    // Two further along is "Build (urgent)".
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Tab);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Build failed"), "{screen}");

    // F8 then Escape closes the newest and leaves the other.
    press(&mut manager, KeyCode::F(8));
    let screen = press(&mut manager, KeyCode::Esc);
    assert!(!screen.contains("Build failed"), "{screen}");
    assert!(screen.contains("Saved"), "{screen}");
}
