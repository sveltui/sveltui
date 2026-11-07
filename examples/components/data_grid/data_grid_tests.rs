use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer, Runtime,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

const WIDTH: u16 = 80;
const HEIGHT: u16 = 24;

/// Renders until focus and the grid's sizing settle, and returns the screen.
fn render(manager: &mut FocusManager) -> Vec<String> {
    for _ in 0..10 {
        Runtime::set_dirty(false);
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <DataGridDemo /> });
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
                .map(|y| (0..WIDTH).map(|x| buffer.get(x, y).unwrap().ch).collect())
                .collect();
        }
    }
    panic!("the example did not settle");
}

fn press(manager: &mut FocusManager, code: KeyCode) -> Vec<String> {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    render(manager)
}

#[test]
fn the_grid_fills_the_screen_above_the_status_lines() {
    let mut manager = FocusManager::new();
    let screen = render(&mut manager);
    assert!(screen[1].contains("Orders"), "{}", screen[1]);
    let bottom = screen.iter().rposition(|row| row.contains('╰')).unwrap();
    // Below the grid: a gap, the status line, a gap, the hint and the padding.
    assert_eq!(bottom, usize::from(HEIGHT) - 6);
    assert!(screen.iter().any(|row| row.contains("#10000")));
}

#[test]
fn sorting_and_opening_an_order_show_in_the_status_line() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    press(&mut manager, KeyCode::Tab);
    press(&mut manager, KeyCode::Up); // the header
    press(&mut manager, KeyCode::Right);
    press(&mut manager, KeyCode::Right);
    press(&mut manager, KeyCode::Right); // Total
    press(&mut manager, KeyCode::Enter);
    let screen = press(&mut manager, KeyCode::Enter); // descending
    assert!(screen.iter().any(|r| r.contains("sorted by column 4 ▼")));
    assert!(
        screen.iter().any(|r| r.contains("499.99")),
        "the biggest total first"
    );

    press(&mut manager, KeyCode::Down);
    press(&mut manager, KeyCode::Char(' '));
    let screen = press(&mut manager, KeyCode::Enter);
    assert!(
        screen
            .iter()
            .any(|r| r.contains("1 selected") && r.contains("opened #"))
    );
}
