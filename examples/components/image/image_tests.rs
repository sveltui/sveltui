use super::*;
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

#[test]
fn the_gallery_draws_the_wheel_three_ways() {
    let manager = FocusManager::new();
    reset_focus_counter();
    begin_frame();
    let node = manager.with_context(|| view! { <Gallery /> });
    end_frame();
    let mut buffer = Buffer::new(100, 20);
    let mut hits = HitMap::new();
    manager.with_context(|| {
        render_node_interactive(&node, Rect::new(0, 0, 100, 20), &mut buffer, &mut hits)
    });
    let colored = (0..buffer.height)
        .flat_map(|y| (0..buffer.width).map(move |x| (x, y)))
        .filter(|&(x, y)| matches!(buffer.get(x, y).unwrap().ch, '▀' | '▄' | '█'))
        .count();
    assert!(colored > 100, "only {colored} image cells drawn");
}
