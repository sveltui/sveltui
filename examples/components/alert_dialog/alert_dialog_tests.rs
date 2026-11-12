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
        let node = manager.with_context(|| view! { <AlertDialogDemo /> });
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
fn cancel_keeps_the_branch_and_action_deletes_it() {
    let mut manager = FocusManager::new();
    render(&mut manager);

    press(&mut manager, KeyCode::Tab);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Delete feature/toast?"), "{screen}");
    // Focus starts on the safe answer.
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Kept it."), "{screen}");
    assert!(screen.contains("feature/toast"), "{screen}");

    press(&mut manager, KeyCode::Enter);
    press(&mut manager, KeyCode::Tab);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(screen.contains("Deleted feature/toast."), "{screen}");
    assert!(screen.contains("Branches: main, develop"), "{screen}");
}
