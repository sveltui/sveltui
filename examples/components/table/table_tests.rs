use super::*;
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

fn screen() -> Vec<String> {
    let manager = FocusManager::new();
    reset_focus_counter();
    begin_frame();
    let node = manager.with_context(|| view! { <TableDemo /> });
    end_frame();
    let mut buffer = Buffer::new(80, 24);
    let mut hits = HitMap::new();
    manager.with_context(|| {
        render_node_interactive(&node, Rect::new(0, 0, 80, 24), &mut buffer, &mut hits)
    });
    (0..24)
        .map(|y| {
            (0..80)
                .map(|x| buffer.get(x, y).unwrap().ch)
                .collect::<String>()
        })
        .collect()
}

#[test]
fn the_table_starts_with_every_rule_inside_a_rounded_border() {
    let rows = screen();
    let top = rows
        .iter()
        .position(|r| r.contains("Popular crates"))
        .expect("the table");
    assert!(
        rows[top].trim_start().starts_with("╭─ Popular crates ─"),
        "{}",
        rows[top]
    );
    assert!(
        rows[top + 2].contains('┼'),
        "a line under the header crosses the columns"
    );
    assert!(rows.iter().any(|r| r.contains("│ serde ")));
}

#[test]
fn the_total_spans_three_columns() {
    let rows = screen();
    let total = rows
        .iter()
        .find(|r| r.contains("Total"))
        .expect("a total row");
    let sum: u32 = PACKAGES.iter().map(|p| p.downloads).sum();
    assert!(total.contains(&sum.to_string()));
    // Only one column line on the total row: the one before the last column.
    let inside = total.trim().trim_matches('│');
    assert_eq!(inside.matches('│').count(), 1, "{total}");
}
