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
        let node = manager.with_context(|| view! { <CalendarDemo /> });
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
fn a_weekday_can_be_booked_and_a_weekend_cannot() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen.contains("No date picked."), "{screen}");

    // Past the previous-month button, then the next-month one, to the grid.
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Tab);
    // A week from today lands on the same weekday; End is always a Sunday.
    press(&mut manager, KeyCode::End);
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(
        screen.contains("No date picked."),
        "a Sunday can't be booked: {screen}"
    );

    press(&mut manager, KeyCode::Down);
    press(&mut manager, KeyCode::Home);
    let screen = press(&mut manager, KeyCode::Enter);
    let monday = date::start_of_week(date::add_days(date::today(), 7), Weekday::Monday);
    assert!(
        screen.contains(&format!("Picked {}.", date::format_date(monday))),
        "{screen}"
    );
}
