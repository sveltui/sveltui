use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;

fn render(manager: &mut FocusManager) -> Buffer {
    for _ in 0..5 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(|| view! { <FormDemo /> });
        end_frame();
        let mut buffer = Buffer::new(WIDTH, HEIGHT);
        let mut hits = HitMap::new();
        manager.with_context(|| {
            render_node_interactive(
                &node,
                Rect::new(0, 0, WIDTH, HEIGHT),
                &mut buffer,
                &mut hits,
            );
        });
        if !manager.reconcile(hits.interactions) {
            return buffer;
        }
    }
    panic!("form focus did not stabilize");
}

fn screen(buffer: &Buffer) -> String {
    (0..HEIGHT)
        .map(|y| {
            (0..WIDTH)
                .map(|x| buffer.get(x, y).unwrap().ch)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Focuses the control assistive tech knows as `label`.
fn focus(manager: &mut FocusManager, label: &str) {
    let id = manager
        .tree()
        .nodes()
        .iter()
        .find(|node| node.interaction.semantics.label == label)
        .unwrap_or_else(|| panic!("{label} must be focusable"))
        .id;
    manager.focus(id);
}

fn press(manager: &mut FocusManager, code: KeyCode) -> String {
    manager.dispatch(KeyEvent::new(code, KeyModifiers::NONE));
    screen(&render(manager))
}

#[test]
fn the_form_and_preview_sit_side_by_side() {
    let mut manager = FocusManager::new();
    let shown = screen(&render(&mut manager));
    let line = shown
        .lines()
        .find(|l| l.contains("Registration Form"))
        .expect("the form's title");
    assert!(line.contains("Preview"), "{shown}");
    assert!(shown.contains("Needs a name and the terms"), "{shown}");
}

#[test]
fn the_preview_follows_the_fields() {
    let mut manager = FocusManager::new();
    render(&mut manager);
    focus(&mut manager, "Name");
    for ch in "Ada".chars() {
        press(&mut manager, KeyCode::Char(ch));
    }
    let shown = screen(&render(&mut manager));
    assert!(shown.contains("Name         Ada"), "{shown}");

    // Picking from the calendar shows the first weekday from today on.
    focus(&mut manager, "Choose date");
    press(&mut manager, KeyCode::Enter);
    let shown = press(&mut manager, KeyCode::Enter);
    let mut day = date::today();
    while matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) {
        day = date::add_days(day, 1);
    }
    let picked = format!("Start date   {}", date::format_date(day));
    if !shown.contains(&picked) {
        // Today was a weekend: Enter on it doesn't pick, so move on to Monday.
        press(&mut manager, KeyCode::End);
        press(&mut manager, KeyCode::Right);
        press(&mut manager, KeyCode::Enter);
    }
    let shown = screen(&render(&mut manager));
    assert!(shown.contains(&picked), "{shown}");

    focus_first_segment(&mut manager);
    for ch in "930p".chars() {
        press(&mut manager, KeyCode::Char(ch));
    }
    let shown = screen(&render(&mut manager));
    assert!(shown.contains("Call time    21:30"), "{shown}");
}

/// The call-time group isn't focusable itself; its first part is the hour.
fn focus_first_segment(manager: &mut FocusManager) {
    let hours: Vec<_> = manager
        .tree()
        .nodes()
        .iter()
        .filter(|n| n.interaction.semantics.label == "hour")
        .map(|n| n.id)
        .collect();
    manager.focus(*hours.first().expect("an hour part"));
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
