use super::*;
use sveltui::element::reset_focus_counter;
use sveltui::element::taffy_bridge::{ComputedLayoutNode, LayoutEngine};

fn layout() -> ComputedLayoutNode {
    reset_focus_counter();
    begin_frame();
    let node = view! { <PositioningDemo /> };
    end_frame();
    LayoutEngine::new().compute_layout(&node, 80, 24)
}

#[test]
fn the_badge_sits_in_the_cards_top_right_corner() {
    let root = layout();
    let card = &root.children[1];
    let badge = card.children[2].rect;
    // Inset is measured from inside the card's border.
    assert_eq!(badge.y, 1);
    assert_eq!(badge.x + badge.width, card.rect.width - 1 - 1);
}

#[test]
fn absolute_views_leave_the_flow_alone() {
    let root = layout();
    let card = &root.children[1];
    let lines: Vec<_> = card.children[..2].iter().map(|c| c.rect.y).collect();
    assert_eq!(
        lines,
        [1, 2],
        "the messages stack as if the badge weren't there"
    );
}

#[test]
fn the_hint_is_pinned_to_the_bottom() {
    let root = layout();
    let hint = root.children.last().unwrap().rect;
    assert_eq!(hint.y + hint.height, 24);
    assert_eq!(hint.x, 1);
}
