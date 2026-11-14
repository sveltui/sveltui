use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer, Runtime,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

const WIDTH: u16 = 90;
const HEIGHT: u16 = 20;

/// Renders until focus holds still and two frames look the same.
fn render(manager: &mut FocusManager) -> String {
    let mut last = String::new();
    for _ in 0..10 {
        Runtime::set_dirty(false);
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <TimeFieldDemo /> });
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
fn an_alarm_and_a_meeting_can_be_typed_in() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen.contains("Alarm: none · Meeting: none"), "{screen}");

    press(&mut manager, KeyCode::Tab);
    for ch in "0730".chars() {
        press(&mut manager, KeyCode::Char(ch));
    }
    let screen = press(&mut manager, KeyCode::Tab);
    assert!(screen.contains("Alarm: 07:30"), "{screen}");

    // Typing the minutes into the meeting moves on to AM/PM; p makes it PM.
    for ch in "245p".chars() {
        press(&mut manager, KeyCode::Char(ch));
    }
    let screen = render(&mut manager);
    assert!(screen.contains("02:45 PM"), "{screen}");
    assert!(screen.contains("Meeting: 14:45"), "{screen}");
}
