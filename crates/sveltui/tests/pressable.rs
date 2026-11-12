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

#[test]
fn hover_covers_what_is_inside_and_reports_leaving_once() {
    use std::cell::RefCell;
    use std::rc::Rc;
    use sveltui::element::{HitMap, render_node_interactive};

    let area = FocusId(700_101);
    let button = FocusId(700_102);
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    let node = view! {
        <View flex_direction=FlexDirection::Column>
            <Pressable focus_id=area on_hover={move |inside| sink.borrow_mut().push(inside)}>
                <View width=10 height=3>
                    <Pressable focus_id=button tab_stop=true on_click={|| {}}>
                        <View width=4 height=1 />
                    </Pressable>
                </View>
            </Pressable>
        </View>
    }
    .into_node();
    let mut buffer = Buffer::new(20, 6);
    let mut hits = HitMap::new();
    render_node_interactive(&node, Rect::new(0, 0, 20, 6), &mut buffer, &mut hits);
    let mut manager = FocusManager::new();
    manager.reconcile(hits.interactions.clone());

    // Onto the button inside the area: that's inside the area too.
    assert!(manager.pointer_move(&hits, 1, 0).changed);
    // Across the area's own cells: still inside, nothing to report.
    assert!(!manager.pointer_move(&hits, 8, 2).changed);
    // Out of the area.
    assert!(manager.pointer_move(&hits, 15, 5).changed);
    assert_eq!(*log.borrow(), [true, false]);
}

#[test]
fn a_requested_frame_comes_due_once() {
    use std::time::{Duration, Instant};
    use sveltui::Runtime;

    let now = Instant::now();
    Runtime::wake_at(now + Duration::from_secs(3600));
    Runtime::wake_at(now + Duration::from_secs(60));
    assert_eq!(Runtime::next_wake(), Some(now + Duration::from_secs(60)));
    assert!(!Runtime::wake_if_due(now));
    assert!(Runtime::wake_if_due(now + Duration::from_secs(61)));
    assert!(!Runtime::wake_if_due(now + Duration::from_secs(61)));
}
