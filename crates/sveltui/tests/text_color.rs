use sveltui::{Buffer, element::render_node, layout::Rect, prelude::*};

fn render(node: &VNode) -> Buffer {
    let mut buffer = Buffer::new(40, 12);
    render_node(node, Rect::new(0, 0, 40, 12), &mut buffer);
    buffer
}

fn assert_text_color(buffer: &Buffer, expected: &str, color: Color) {
    let cells: Vec<_> = buffer.cells.iter().filter(|cell| cell.ch != ' ').collect();
    assert_eq!(
        cells.iter().map(|cell| cell.ch).collect::<String>(),
        expected
    );
    for cell in cells {
        assert_eq!(cell.fg, Some(color), "character {:?}", cell.ch);
    }
}

#[test]
fn span_and_paragraph_inherit_parent_foreground_through_plain_views() {
    let node = view! {
        <View width=30 foreground_color=Color::Blue background_color=Color::Red>
            <TextSpan>"Span"</TextSpan>
            <View><Paragraph>"Paragraph"</Paragraph></View>
        </View>
    };
    assert_text_color(&render(&node), "SpanParagraph", Color::Blue);
}

#[test]
fn closest_foreground_wins_and_does_not_leak_into_siblings() {
    let node = view! {
        <View width=30 foreground_color=Color::White>
            <View foreground_color=Color::DarkGrey>
                <TextSpan>"A"</TextSpan>
                <Paragraph foreground_color=Color::Blue>"B"</Paragraph>
            </View>
            <TextSpan>"C"</TextSpan>
        </View>
    };
    let buffer = render(&node);
    for (y, (ch, color)) in [
        ('A', Color::DarkGrey),
        ('B', Color::Blue),
        ('C', Color::White),
    ]
    .into_iter()
    .enumerate()
    {
        let cell = buffer.get(0, y as u16).unwrap();
        assert_eq!(cell.ch, ch);
        assert_eq!(cell.fg, Some(color));
    }
}

#[test]
fn wrapped_paragraph_inherits_foreground_on_every_line() {
    let node = view! {
        <View width=8 foreground_color=Color::White>
            <Paragraph>"one two three four"</Paragraph>
        </View>
    };
    assert_text_color(&render(&node), "onetwothreefour", Color::White);
}

#[test]
fn explicit_text_colors_override_inherited_colors_and_none_preserves_inheritance() {
    for paragraph in [false, true] {
        let node = view! {
            <View width=20 foreground_color=Color::White background_color=Color::DarkGrey>
                {#if paragraph}
                    <Paragraph foreground_color=Color::Blue background_color=Color::Red>"A"</Paragraph>
                    <Paragraph opt_foreground_color=None opt_background_color=None>"B"</Paragraph>
                {:else}
                    <TextSpan foreground_color=Color::Blue background_color=Color::Red>"A"</TextSpan>
                    <TextSpan opt_foreground_color=None opt_background_color=None>"B"</TextSpan>
                {/if}
            </View>
        };
        let buffer = render(&node);
        for (y, (ch, foreground, background)) in [
            ('A', Color::Blue, Color::Red),
            ('B', Color::White, Color::DarkGrey),
        ]
        .into_iter()
        .enumerate()
        {
            let cell = buffer.get(0, y as u16).unwrap();
            assert_eq!(cell.ch, ch);
            assert_eq!(cell.fg, Some(foreground));
            assert_eq!(cell.bg, Some(background));
        }
    }
}

#[test]
fn overlay_text_inherits_the_foreground_where_it_is_declared() {
    for height in [Dimension::Auto, Dimension::Length(5.0)] {
        let node = view! {
            <View width=30 height=height foreground_color=Color::Blue>
                <Overlay open=true backdrop=false placement={Placement::Anchor { x: 0, y: 0 }}>
                    <View width=20>
                        <TextSpan>"Span"</TextSpan>
                        <Paragraph>"Paragraph"</Paragraph>
                    </View>
                </Overlay>
            </View>
        };
        assert_text_color(&render(&node), "SpanParagraph", Color::Blue);
    }
}
