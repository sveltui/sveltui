use super::*;
use sveltui::element::reset_focus_counter;
use sveltui::element::taffy_bridge::{ComputedLayoutNode, LayoutEngine};

fn tag_box() -> ComputedLayoutNode {
    reset_focus_counter();
    begin_frame();
    let node = view! { <WrappingDemo /> };
    end_frame();
    let root = LayoutEngine::new().compute_layout(&node, 80, 24);
    root.children[2].clone()
}

#[test]
fn tags_wrap_within_the_box() {
    let tags = tag_box();
    let inner_right = tags.rect.width - 2;
    for tag in &tags.children {
        assert!(tag.rect.x + tag.rect.width <= inner_right, "{:?}", tag.rect);
    }
    let lines: std::collections::BTreeSet<_> = tags.children.iter().map(|t| t.rect.y).collect();
    assert!(lines.len() > 1, "twelve tags don't fit on one line of 40");
}

#[test]
fn gaps_differ_between_rows_and_columns() {
    let tags = tag_box();
    let first = tags.children[0].rect;
    let second = tags.children[1].rect;
    assert_eq!(second.x - (first.x + first.width), 2, "column_gap");

    let next_line = tags
        .children
        .iter()
        .find(|t| t.rect.y > first.y)
        .unwrap()
        .rect;
    assert_eq!(next_line.y - (first.y + first.height), 1, "row_gap");
}
