//! `Pressable` attaches its interaction to a view instead of being a box
//! of its own.

use sveltui::element::render_node;
use sveltui::element::taffy_bridge::LayoutEngine;
use sveltui::element::{FocusId, InteractionTree};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;
use sveltui::view;

#[test]
fn wrapping_one_view_adds_no_box() {
    let node = view! {
        <View flex_direction=FlexDirection::Row>
            <Pressable on_click={|| {}}>
                <View width=10 height=2 />
            </Pressable>
        </View>
    }
    .into_node();

    let VNode::View(root) = &node else {
        panic!("expected a view")
    };
    let VNode::View(child) = &root.children[0] else {
        panic!("the pressable should become the view it wraps")
    };
    assert_eq!(child.pressables.len(), 1);
    assert!(child.children.is_empty(), "no wrapper around the view");

    let layout = LayoutEngine::new().compute_layout(&node, 40, 5);
    let rect = layout.children[0].rect;
    assert_eq!((rect.x, rect.y, rect.width, rect.height), (0, 0, 10, 2));
}

#[test]
fn several_children_are_laid_out_not_stacked_on_each_other() {
    let node = view! {
        <Pressable on_click={|| {}}>
            <TextSpan>"one"</TextSpan>
            <TextSpan>"two"</TextSpan>
        </Pressable>
    }
    .into_node();
    let mut buffer = Buffer::new(10, 2);
    render_node(&node, Rect::new(0, 0, 10, 2), &mut buffer);
    let row = |y| -> String { (0..10).map(|x| buffer.get(x, y).unwrap().ch).collect() };
    assert_eq!(row(0).trim_end(), "one");
    assert_eq!(row(1).trim_end(), "two");
}

#[test]
fn nested_pressables_share_one_view_and_keep_their_nesting() {
    let outer = FocusId(700_001);
    let inner = FocusId(700_002);
    let node = view! {
        <Pressable focus_id=outer semantics={Semantics::new(Role::Toolbar, "Tools")}>
            <Pressable focus_id=inner tab_stop=true>
                <View width=4 height=1 />
            </Pressable>
        </Pressable>
    }
    .into_node();

    let VNode::View(view) = &node else {
        panic!("expected a view")
    };
    let ids: Vec<_> = view.pressables.iter().map(|p| p.focus_id).collect();
    assert_eq!(ids, [Some(outer), Some(inner)], "outermost first");

    let tree = InteractionTree::from_node(&node);
    assert_eq!(tree.node(inner).unwrap().parent, Some(outer));
}

#[test]
fn a_lone_text_keeps_its_width_in_a_tight_row() {
    let node = view! {
        <View width=8 flex_direction=FlexDirection::Row>
            <Pressable on_click={|| {}}><TextSpan>"label"</TextSpan></Pressable>
            <View width=6 height=1 />
        </View>
    }
    .into_node();
    let layout = LayoutEngine::new().compute_layout(&node, 8, 1);
    assert_eq!(
        layout.children[0].rect.width, 5,
        "the text doesn't shrink, just as it wouldn't unwrapped"
    );
}
