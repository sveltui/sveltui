use crossterm::style::Color;
use sveltui::element::render_node;
use sveltui::element::taffy_bridge::LayoutEngine;
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;
use sveltui::view;

fn render(node: &VNode, width: u16, height: u16) -> Buffer {
    let mut buffer = Buffer::new(width, height);
    render_node(node, Rect::new(0, 0, width, height), &mut buffer);
    buffer
}

#[test]
fn test_view_flex_row_layout() {
    let child1 = View(
        ViewProps::builder()
            .width(Dimension::Length(10.0))
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let child2 = View(
        ViewProps::builder()
            .width(Dimension::Length(20.0))
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(60.0))
            .height(Dimension::Length(10.0))
            .flex_direction(FlexDirection::Row)
            .gap(2)
            .child(child1)
            .child(child2)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 60, 10);
    assert_eq!(layout.rect.width, 60);
    assert_eq!(layout.rect.height, 10);
    assert_eq!(layout.children.len(), 2);
    assert_eq!(layout.children[0].rect.x, 0);
    assert_eq!(layout.children[0].rect.width, 10);
    assert_eq!(layout.children[1].rect.x, 12); // 10 + gap 2
    assert_eq!(layout.children[1].rect.width, 20);
}

#[test]
fn test_view_flex_column_layout() {
    let child1 = View(
        ViewProps::builder()
            .width(Dimension::Length(20.0))
            .height(Dimension::Length(3.0))
            .build(),
    )
    .into_node();
    let child2 = View(
        ViewProps::builder()
            .width(Dimension::Length(20.0))
            .height(Dimension::Length(4.0))
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(40.0))
            .height(Dimension::Length(20.0))
            .flex_direction(FlexDirection::Column)
            .gap(1)
            .child(child1)
            .child(child2)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 40, 20);
    assert_eq!(layout.children[0].rect.y, 0);
    assert_eq!(layout.children[0].rect.height, 3);
    assert_eq!(layout.children[1].rect.y, 4); // 3 + gap 1
    assert_eq!(layout.children[1].rect.height, 4);
}

#[test]
fn test_view_sizing_lengths_and_percentages() {
    let child_fixed = View(
        ViewProps::builder()
            .width(Dimension::Length(15.0))
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let child_percent = View(
        ViewProps::builder()
            .width(Dimension::Percent(0.5))
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(100.0))
            .height(Dimension::Length(20.0))
            .flex_direction(FlexDirection::Column)
            .child(child_fixed)
            .child(child_percent)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 100, 20);
    assert_eq!(layout.children[0].rect.width, 15);
    assert_eq!(layout.children[1].rect.width, 50);
}

#[test]
fn test_view_min_max_constraints() {
    let child_clamped = View(
        ViewProps::builder()
            .width(Dimension::Length(100.0))
            .max_width(Dimension::Length(35.0))
            .min_width(Dimension::Length(10.0))
            .height(Dimension::Length(2.0))
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(80.0))
            .height(Dimension::Length(20.0))
            .child(child_clamped)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 80, 20);
    assert_eq!(layout.children[0].rect.width, 35);
}

#[test]
fn test_view_padding_and_margin() {
    let child = View(
        ViewProps::builder()
            .width(Dimension::Length(10.0))
            .height(Dimension::Length(5.0))
            .margin(Margin::all(2))
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(50.0))
            .height(Dimension::Length(30.0))
            .padding(Padding::all(3))
            .child(child)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 50, 30);
    // Inside root: padding 3 + child margin 2 => child x = 5, y = 5
    assert_eq!(layout.children[0].rect.x, 5);
    assert_eq!(layout.children[0].rect.y, 5);
    assert_eq!(layout.children[0].rect.width, 10);
    assert_eq!(layout.children[0].rect.height, 5);
}

#[test]
fn test_view_alignment_justify_content_center() {
    let child = View(
        ViewProps::builder()
            .width(Dimension::Length(20.0))
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(60.0))
            .height(Dimension::Length(20.0))
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::Center)
            .child(child)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 60, 20);
    // (60 - 20) / 2 = 20
    assert_eq!(layout.children[0].rect.x, 20);
}

#[test]
fn test_view_alignment_align_self() {
    let child1 = View(
        ViewProps::builder()
            .width(Dimension::Length(10.0))
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let child2 = View(
        ViewProps::builder()
            .width(Dimension::Length(10.0))
            .height(Dimension::Length(5.0))
            .align_self(AlignSelf::FlexEnd)
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(50.0))
            .height(Dimension::Length(20.0))
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::FlexStart)
            .child(child1)
            .child(child2)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 50, 20);
    assert_eq!(layout.children[0].rect.y, 0);
    assert_eq!(layout.children[1].rect.y, 15); // 20 - 5
}

#[test]
fn test_view_flex_grow() {
    let child1 = View(
        ViewProps::builder()
            .flex_grow(1.0)
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let child2 = View(
        ViewProps::builder()
            .flex_grow(2.0)
            .height(Dimension::Length(5.0))
            .build(),
    )
    .into_node();
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(60.0))
            .height(Dimension::Length(20.0))
            .flex_direction(FlexDirection::Row)
            .child(child1)
            .child(child2)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 60, 20);
    assert_eq!(layout.children[0].rect.width, 20);
    assert_eq!(layout.children[1].rect.width, 40);
}

#[test]
fn test_view_scrolling_overflow_content_size() {
    let mut children = Vec::new();
    for _ in 0..10 {
        children.push(
            View(
                ViewProps::builder()
                    .width(Dimension::Length(20.0))
                    .height(Dimension::Length(3.0))
                    .flex_shrink(0.0)
                    .build(),
            )
            .into_node(),
        );
    }
    let root = View(
        ViewProps::builder()
            .width(Dimension::Length(30.0))
            .height(Dimension::Length(10.0))
            .flex_direction(FlexDirection::Column)
            .overflow_y(Overflow::Scroll)
            .children(children)
            .build(),
    )
    .into_node();

    let mut engine = LayoutEngine::new();
    let layout = engine.compute_layout(&root, 30, 10);
    assert_eq!(layout.rect.width, 30);
    assert_eq!(layout.rect.height, 10);
    // Total content height is 10 items * 3 = 30
    assert_eq!(layout.content_size.1, 30);
    assert_eq!(layout.scroll_size.1, 20); // 30 - 10 = 20 scrollable range
}

#[test]
fn test_view_color_cascading() {
    let node = view! {
        <View width=30 height=5 background_color=Color::Blue foreground_color=Color::Yellow>
            <View width=20 height=3>
                <TextSpan>"Cascaded Text"</TextSpan>
            </View>
        </View>
    };

    let buffer = render(&node, 40, 10);
    // Background of container
    assert_eq!(buffer.get(0, 0).unwrap().bg, Some(Color::Blue));
    // Text inherits both background and foreground
    let text_cell = buffer.get(0, 0).unwrap();
    assert_eq!(text_cell.ch, 'C');
    assert_eq!(text_cell.fg, Some(Color::Yellow));
    assert_eq!(text_cell.bg, Some(Color::Blue));
}

#[test]
fn test_pressable_event_registration() {
    let id = FocusId::new();
    let node = view! {
        <Pressable focus_id=id tab_stop=true>
            <View width=15 height=3 border=BorderStyle::Single>
                <TextSpan>"Button"</TextSpan>
            </View>
        </Pressable>
    };

    let mut buffer = Buffer::new(30, 10);
    let mut hits = sveltui::element::HitMap::default();
    sveltui::element::render_node_interactive(
        &node,
        Rect::new(0, 0, 30, 10),
        &mut buffer,
        &mut hits,
    );

    assert!(hits.hit_test(2, 1).is_some());
    assert_eq!(hits.hit_test(2, 1).unwrap().focus_id, Some(id));
}
