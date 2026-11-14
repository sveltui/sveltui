use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer, Runtime,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

const WIDTH: u16 = 90;
const HEIGHT: u16 = 24;

/// Renders until focus holds still and two frames look the same.
fn render(manager: &mut FocusManager) -> String {
    let mut last = String::new();
    for _ in 0..10 {
        Runtime::set_dirty(false);
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <DatePickerDemo /> });
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
fn the_calendar_picks_a_check_in_date_and_the_field_takes_a_birthday() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen.contains("Check-in: none"), "{screen}");

    // Three date segments, then the button that opens the calendar.
    for _ in 0..4 {
        press(&mut manager, KeyCode::Tab);
    }
    let screen = press(&mut manager, KeyCode::Enter);
    let today = date::format_date(date::today());
    assert!(screen.contains("Mo Tu We"), "the calendar opens: {screen}");
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(
        screen.contains(&format!("Check-in: {today}")),
        "today is picked: {screen}"
    );

    // Focus is back on the button; the birthday field comes next.
    press(&mut manager, KeyCode::Tab);
    for ch in "13021990".chars() {
        press(&mut manager, KeyCode::Char(ch));
    }
    let screen = render(&mut manager);
    assert!(screen.contains("Birthday: 1990-02-13"), "{screen}");
}
