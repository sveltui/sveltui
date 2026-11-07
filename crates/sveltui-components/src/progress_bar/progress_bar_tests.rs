use crate::*;
use sveltui::element::render_node;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn row(node: &VNode) -> String {
    let mut buffer = Buffer::new(30, 1);
    render_node(node, Rect::new(0, 0, 30, 1), &mut buffer);
    let text: String = (0..30).map(|x| buffer.get(x, 0).unwrap().ch).collect();
    text.trim_end().to_owned()
}

#[test]
fn progress_bar_fills_proportionally_and_shows_percentage() {
    let node = view! { <ProgressBar ratio=0.5f32 width=10u16 /> };
    assert_eq!(row(&node), "[█████░░░░░]  50%");
}

#[test]
fn progress_bar_clamps_ratio_and_can_hide_percentage() {
    let node = view! { <ProgressBar ratio=1.5f32 width=4u16 show_percentage=false /> };
    assert_eq!(row(&node), "[████]");
    let node = view! { <ProgressBar ratio={-1.0f32} width=4u16 show_percentage=false /> };
    assert_eq!(row(&node), "[░░░░]");
}
