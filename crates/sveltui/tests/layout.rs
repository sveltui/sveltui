use sveltui::layout::{BorderStyle, Placement, Rect, compute_popover_rect};

#[test]
fn test_rect_inner() {
    let r = Rect::new(5, 5, 20, 10);
    let inner = r.inner(1, 2, 3, 4);
    assert_eq!(inner.x, 9);
    assert_eq!(inner.y, 6);
    assert_eq!(inner.width, 14);
    assert_eq!(inner.height, 6);
}

#[test]
fn test_border_chars() {
    let rounded = BorderStyle::Rounded.chars().unwrap();
    assert_eq!(rounded.top_left, '╭');
    assert_eq!(rounded.top_right, '╮');
    assert_eq!(rounded.bottom_left, '╰');
    assert_eq!(rounded.bottom_right, '╯');
    assert_eq!(rounded.horizontal, '─');
    assert_eq!(rounded.vertical, '│');
}

#[test]
fn test_rect_intersect() {
    let a = Rect::new(0, 0, 10, 10);
    let b = Rect::new(5, 5, 10, 10);
    let inter = a.intersect(&b).unwrap();
    assert_eq!(inter, Rect::new(5, 5, 5, 5));

    let c = Rect::new(20, 20, 5, 5);
    assert_eq!(a.intersect(&c), None);
}

#[test]
fn test_compute_popover_rect_center() {
    let screen = Rect::new(0, 0, 80, 24);
    let rect = compute_popover_rect(Placement::Center, (20, 10), None, screen);
    assert_eq!(rect, Rect::new(30, 7, 20, 10));
}

#[test]
fn test_compute_popover_rect_anchored_bottom_start() {
    let screen = Rect::new(0, 0, 80, 24);
    let anchor = Rect::new(10, 5, 12, 3);
    let rect = compute_popover_rect(Placement::BottomStart, (20, 6), Some(anchor), screen);
    assert_eq!(rect, Rect::new(10, 8, 20, 6));
}

#[test]
fn test_compute_popover_rect_auto_flip() {
    let screen = Rect::new(0, 0, 80, 24);
    // Anchor placed near bottom edge: y=20, h=2 (ends at y=22)
    let anchor = Rect::new(10, 20, 12, 2);
    // With height=8, bottom would need y=22..30 which exceeds screen height 24.
    // Space below is 2, space above is 20, so it auto-flips to Top!
    let rect = compute_popover_rect(Placement::BottomStart, (20, 8), Some(anchor), screen);
    assert_eq!(rect, Rect::new(10, 12, 20, 8));
}

#[test]
fn test_compute_popover_rect_clamping() {
    let screen = Rect::new(0, 0, 80, 24);
    // Explicit anchor coordinate at edge
    let rect = compute_popover_rect(Placement::Anchor { x: 75, y: 22 }, (20, 10), None, screen);
    assert_eq!(rect, Rect::new(60, 14, 20, 10));
}

#[test]
fn positioner_gap_and_skid_offset_the_content() {
    use sveltui::layout::{Anchor, Positioner};
    let screen = Rect::new(0, 0, 80, 24);
    let anchor = Rect::new(10, 5, 8, 1);
    let rect = Positioner::new(Placement::BottomStart)
        .gap(1)
        .skid(2)
        .position(Anchor::Rect(anchor), (20, 4), screen);
    assert_eq!((rect.x, rect.y), (12, 7));
}

#[test]
fn positioner_flip_can_be_disabled() {
    use sveltui::layout::{Anchor, Positioner};
    let screen = Rect::new(0, 0, 80, 24);
    let anchor = Rect::new(10, 20, 8, 1);
    let flipped =
        Positioner::new(Placement::BottomStart).position(Anchor::Rect(anchor), (20, 8), screen);
    assert_eq!(flipped.y, 12);
    let pinned = Positioner::new(Placement::BottomStart)
        .flip(false)
        .position(Anchor::Rect(anchor), (20, 8), screen);
    assert_eq!(pinned.y, 16);
}

#[test]
fn positioner_point_anchor_flips_at_the_screen_edge() {
    use sveltui::layout::{Anchor, Positioner};
    let screen = Rect::new(0, 0, 80, 24);
    let rect =
        Positioner::new(Placement::BottomStart).position(Anchor::Point(70, 22), (20, 6), screen);
    assert_eq!((rect.x, rect.y), (60, 16));
}

#[test]
fn positioner_shift_off_lets_content_overflow_the_edge() {
    use sveltui::layout::{Anchor, Positioner};
    let screen = Rect::new(0, 0, 80, 24);
    let anchor = Rect::new(70, 2, 8, 1);
    let shifted =
        Positioner::new(Placement::BottomStart).position(Anchor::Rect(anchor), (20, 3), screen);
    assert_eq!(shifted.x, 60);
    let free = Positioner::new(Placement::BottomStart)
        .shift(false)
        .position(Anchor::Rect(anchor), (20, 3), screen);
    assert_eq!(free.x, 70);
}

#[test]
fn positioner_matches_anchor_width_and_caps_size() {
    use sveltui::layout::{Anchor, Positioner};
    let screen = Rect::new(0, 0, 80, 24);
    let anchor = Rect::new(4, 2, 12, 1);
    let rect = Positioner::new(Placement::BottomStart)
        .match_anchor_width(true)
        .max_size(40, 5)
        .position(Anchor::Rect(anchor), (30, 9), screen);
    assert_eq!((rect.width, rect.height), (12, 5));
}

#[test]
fn positioner_to_inset_is_relative_to_the_parent() {
    use sveltui::layout::{Anchor, Inset, Positioner};
    let parent = Rect::new(10, 4, 40, 10);
    let anchor = Rect::new(14, 5, 6, 1);
    let inset =
        Positioner::new(Placement::BottomStart).to_inset(Anchor::Rect(anchor), parent, (8, 3));
    assert_eq!(inset, Inset::new().left(4).top(2));
}
