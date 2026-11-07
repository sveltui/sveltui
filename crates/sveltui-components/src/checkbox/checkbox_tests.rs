use crate::*;
use sveltui::element::render_node;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn row(node: &VNode) -> String {
    let mut buffer = Buffer::new(20, 1);
    render_node(node, Rect::new(0, 0, 20, 1), &mut buffer);
    let text: String = (0..20).map(|x| buffer.get(x, 0).unwrap().ch).collect();
    text.trim_end().to_owned()
}

#[test]
fn checkbox_renders_its_binding_and_label() {
    let checked = state(true);
    let node = view! { <Checkbox checked=checked label="Accept" /> };
    assert_eq!(row(&node), "  [x] Accept");

    checked.set(false);
    let node = view! { <Checkbox checked=checked label="Accept" /> };
    assert_eq!(row(&node), "  [ ] Accept");
}

#[test]
fn checkbox_without_binding_renders_unchecked() {
    let node = view! { <Checkbox label="Remember me" /> };
    assert_eq!(row(&node), "  [ ] Remember me");
}
