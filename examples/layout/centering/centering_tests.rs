use super::*;
use sveltui::element::reset_focus_counter;
use sveltui::element::taffy_bridge::{ComputedLayoutNode, LayoutEngine};

fn layout() -> ComputedLayoutNode {
    reset_focus_counter();
    begin_frame();
    let node = view! { <CenteringDemo /> };
    end_frame();
    LayoutEngine::new().compute_layout(&node, 90, 24)
}

/// Whether `inner` sits in the middle of `outer`, give or take the half
/// cell taffy rounds away.
fn centered(start: u16, size: u16, outer: u16) -> bool {
    let after = outer - start - size;
    start.abs_diff(after) <= 1
}

#[test]
fn each_frame_centers_its_box_the_way_its_margin_says() {
    let root = layout();
    let frames = &root.children[1].children;
    let expected = [(true, false), (false, true), (true, true)];
    for (frame, (across, down)) in frames.iter().zip(expected) {
        // The block's position is relative to the frame, inside its border.
        let block = frame.children[0].rect;
        let (inner_w, inner_h) = (frame.rect.width - 2, frame.rect.height - 2);
        assert_eq!(
            centered(block.x - 1, block.width, inner_w),
            across,
            "{block:?}"
        );
        assert_eq!(
            centered(block.y - 1, block.height, inner_h),
            down,
            "{block:?}"
        );
    }
}

#[test]
fn one_auto_edge_pushes_the_hint_to_the_end() {
    let root = layout();
    let bar = &root.children[2];
    let hint = bar.children.last().unwrap().rect;
    assert_eq!(
        hint.x + hint.width,
        bar.rect.width - 2,
        "flush with the right padding"
    );
}
