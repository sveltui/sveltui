use crossterm::style::Color;
use sveltui::element::*;
use sveltui::layout::*;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

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
fn centered_layout_footer_inherits_the_stated_foreground() {
    let node = view! {
        <View width=40>
            <View justify_content=JustifyContent::Center align_items=AlignItems::Center
                background_color=Color::Red foreground_color=Color::Blue>
                <TextSpan>"Tab: Switch controls"</TextSpan>
            </View>
        </View>
    };
    assert_text_color(&render(&node), "Tab:Switchcontrols", Color::Blue);
}
