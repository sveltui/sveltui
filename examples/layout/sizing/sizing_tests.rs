use super::*;
use sveltui::element::reset_focus_counter;
use sveltui::element::taffy_bridge::{ComputedLayoutNode, LayoutEngine};

fn panel() -> ComputedLayoutNode {
    reset_focus_counter();
    begin_frame();
    let node = view! { <SizingDemo /> };
    end_frame();
    let root = LayoutEngine::new().compute_layout(&node, 100, 24);
    root.children[1].clone()
}

#[test]
fn aspect_ratio_sets_the_height_from_the_width() {
    let content = &panel().children[1];
    let tiles = &content.children[1].children;
    let sizes: Vec<_> = tiles
        .iter()
        .map(|t| (t.rect.width, t.rect.height))
        .collect();
    assert_eq!(sizes, [(8, 4), (12, 3)]);
}

#[test]
fn max_width_caps_a_stretched_view() {
    let content = &panel().children[1];
    assert_eq!(content.children[0].rect.width, 30);
}

#[test]
fn the_visible_sidebar_takes_its_width() {
    let panel = panel();
    assert_eq!(panel.children[0].rect.width, 16);
    assert_eq!(
        panel.children[1].rect.x, 17,
        "content starts after the sidebar and the gap"
    );
}
