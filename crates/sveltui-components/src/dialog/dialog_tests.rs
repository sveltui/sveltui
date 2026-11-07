use crate::*;
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::terminal::Buffer;

#[test]
fn test_nested_scroll_box_scroll() {
    let scroll_id = FocusId::new(); // User calls FocusId::new() in main()
    let tree = ViewElement::new() // Root View
        .child(
            ViewElement::new() // Dialog View
                .child(
                    ViewElement::new() // Scrollable View
                        .scroll_id(scroll_id)
                        .height(3)
                        .overflow_y(Overflow::Scroll)
                        .child(TextElement::new("Line 1"))
                        .child(TextElement::new("Line 2"))
                        .child(TextElement::new("Line 3"))
                        .child(TextElement::new("Line 4"))
                        .child(TextElement::new("Line 5"))
                        .child(TextElement::new("Line 6")),
                ),
        )
        .into_node();

    reset_focus_counter();
    let mut buffer = Buffer::new(20, 10);
    let mut hit_map = HitMap::new();
    render_node_interactive(&tree, Rect::new(0, 0, 20, 10), &mut buffer, &mut hit_map);

    // Hit test scroll container
    let target = hit_map
        .targets
        .iter()
        .find(|t| t.focus_id == Some(scroll_id))
        .expect("target should be found");
    let on_scroll = target.on_scroll.as_ref().expect("on_scroll should exist");
    on_scroll(0, 2);

    // Next frame render
    reset_focus_counter();
    let mut buffer2 = Buffer::new(20, 10);
    let mut hit_map2 = HitMap::new();
    render_node_interactive(&tree, Rect::new(0, 0, 20, 10), &mut buffer2, &mut hit_map2);

    assert_eq!(get_scroll(scroll_id), (0, 2));
}

#[test]
fn test_dialog_component_into_node() {
    let dialog = Dialog(
        DialogProps::builder()
            .title("Confirm Action")
            .open(true)
            .children(Snippet::from(
                Button(ButtonProps::builder().children(Snippet::from("OK")).build()).into_node(),
            ))
            .build(),
    )
    .into_node();

    match dialog {
        VNode::Overlay(overlay) => {
            assert!(overlay.open);
            assert!(overlay.modal);
            assert!(overlay.backdrop);
            assert!(overlay.content.is_some());
        }
        _ => panic!("Expected VNode::Overlay"),
    }
}
