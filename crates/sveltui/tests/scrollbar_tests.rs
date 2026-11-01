use sveltui::{
    Buffer,
    element::{get_scroll, render_node_interactive, set_scroll},
    layout::Rect,
    prelude::*,
};

fn render(node: &VNode, width: u16, height: u16) -> (Buffer, HitMap) {
    let mut buffer = Buffer::new(width, height);
    let mut hits = HitMap::new();
    render_node_interactive(node, Rect::new(0, 0, width, height), &mut buffer, &mut hits);
    (buffer, hits)
}

#[test]
fn vertical_thumb_is_proportional_and_reaches_both_track_ends() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=10 overflow_y=Overflow::Scroll scroll_id=id>
            {#each (0..20) as i}<TextSpan>{format!("line {i:02}")}</TextSpan>{/each}
        </View>
    };
    for (offset, start) in [(0, 0), (5, 2), (10, 5)] {
        set_scroll(id, (0, offset));
        let (buffer, _) = render(&node, 12, 10);
        let thumb: Vec<_> = (0..10)
            .filter(|&y| buffer.get(11, y).unwrap().bold)
            .collect();
        assert_eq!(thumb, (start..start + 5).collect::<Vec<_>>());
    }
}

#[test]
fn horizontal_thumb_is_proportional_and_reaches_both_track_ends() {
    let id = FocusId::new();
    let node = view! {
        <View width=10 height=3 overflow_x=Overflow::Scroll scroll_id=id>
            <TextSpan>"0123456789abcdefghij"</TextSpan>
        </View>
    };
    for (offset, start) in [(0, 0), (5, 2), (10, 5)] {
        set_scroll(id, (offset, 0));
        let (buffer, _) = render(&node, 10, 3);
        let thumb: Vec<_> = (0..10)
            .filter(|&x| buffer.get(x, 2).unwrap().bold)
            .collect();
        assert_eq!(thumb, (start..start + 5).collect::<Vec<_>>());
    }
}

#[test]
fn both_tracks_leave_the_last_content_row_and_column_visible_at_the_end() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=7 border=BorderStyle::Single padding=1
            overflow=Overflow::Scroll scroll_id=id>
            {#each (0..6) as i}<TextSpan>{format!("{i}abcdefghijkZ")}</TextSpan>{/each}
        </View>
    };
    let (_, hits) = render(&node, 12, 7);
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    target.on_scroll.as_ref().unwrap()(i16::MAX, i16::MAX);
    let (buffer, _) = render(&node, 12, 7);
    // Seven content columns and two content rows remain after padding and tracks.
    assert_eq!(get_scroll(id), (6, 4));
    assert_eq!(buffer.get(8, 3).unwrap().ch, 'Z');
    assert_eq!(
        buffer.get(9, 4).unwrap().ch,
        ScrollbarChars::default().corner
    );
    assert_eq!(buffer.get(9, 3).unwrap().fg, Some(Color::White));
    assert_eq!(buffer.get(8, 4).unwrap().fg, Some(Color::White));
}

#[test]
fn content_that_fits_a_padded_view_does_not_scroll_into_blank_space() {
    let id = FocusId::new();
    let node = view! {
        <View width=12 height=7 border=BorderStyle::Single padding=1
            overflow=Overflow::Scroll scroll_id=id>
            <TextSpan>"first"</TextSpan><TextSpan>"last"</TextSpan>
        </View>
    };
    let (_, hits) = render(&node, 12, 7);
    let target = hits.targets.iter().find(|t| t.on_scroll.is_some()).unwrap();
    target.on_scroll.as_ref().unwrap()(i16::MAX, i16::MAX);
    assert_eq!(get_scroll(id), (0, 0));
    let (buffer, _) = render(&node, 12, 7);
    assert_eq!(buffer.get(2, 2).unwrap().ch, 'f');
    assert_eq!(buffer.get(2, 3).unwrap().ch, 'l');
    assert!(!buffer.cells.iter().any(|c| matches!(c.ch, '█' | '▄')));
}

#[test]
fn long_scroll_ranges_do_not_overflow_thumb_arithmetic() {
    let id = FocusId::new();
    let node = view! {
        <View width=4 height=100 overflow_y=Overflow::Scroll scroll_id=id>
            <View width=1 height=60000 />
        </View>
    };
    set_scroll(id, (0, 59900));
    let (buffer, _) = render(&node, 4, 100);
    assert_eq!(buffer.get(3, 99).unwrap().fg, Some(Color::White));
}

#[test]
fn hidden_scrollbars_keep_the_full_content_viewport() {
    let id = FocusId::new();
    let child = FocusId::new();
    let node = view! {
        <View width=10 height=3 overflow=Overflow::Scroll scroll_id=id
            scrollbar_visibility=ScrollbarVisibility::Never>
            <Pressable focus_id=child><View width=Size::Fill height=6 /></Pressable>
        </View>
    };
    let (buffer, hits) = render(&node, 10, 3);
    assert_eq!(hits.layouts[&child].width, 10);
    assert!(!buffer.cells.iter().any(|c| matches!(c.ch, '█' | '▄')));
}
