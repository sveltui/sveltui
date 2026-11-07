use crate::element::taffy_bridge::LAYOUT_PASSES;
use crate::element::{FocusId, HitMap, IntoNode, OverlayElement, VNode, render_node_interactive};
use crate::layout::Rect;
use crate::prelude::*;
use crate::terminal::Buffer;
use crate::view;

/// Renders one frame and returns how many layout passes it took.
fn passes_for(node: &VNode, width: u16, height: u16) -> usize {
    LAYOUT_PASSES.with(|passes| passes.set(0));
    let mut buffer = Buffer::new(width, height);
    let mut hits = HitMap::new();
    render_node_interactive(node, Rect::new(0, 0, width, height), &mut buffer, &mut hits);
    LAYOUT_PASSES.with(|passes| passes.get())
}

#[test]
fn a_frame_is_laid_out_once() {
    let node = view! {
        <View border=BorderStyle::Single padding=1>
            <View flex_direction=FlexDirection::Row gap=1>
                <TextSpan>"Name"</TextSpan>
                <Paragraph>"Some wrapped text that is long enough to wrap"</Paragraph>
            </View>
            <Pressable on_click={|| {}}>
                <TextSpan>"one"</TextSpan>
                <TextSpan>"two"</TextSpan>
            </Pressable>
        </View>
    }
    .into_node();
    assert_eq!(passes_for(&node, 40, 12), 1);
}

#[test]
fn an_open_overlay_adds_one_pass_for_its_content() {
    let overlay = OverlayElement::new()
        .id(FocusId(900_001))
        .trigger(view! { <TextSpan>"open"</TextSpan> })
        .content(view! { <View border=BorderStyle::Single><TextSpan>"menu"</TextSpan></View> })
        .open(true);
    let node = view! { <View>{overlay.into_node()}</View> }.into_node();
    assert_eq!(passes_for(&node, 40, 12), 2);
}
