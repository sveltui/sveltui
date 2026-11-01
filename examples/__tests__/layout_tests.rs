use super::*;
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
};

#[test]
fn actual_layout_footer_renders_white_foreground_on_white_background_after_resizing() {
    let mut manager = FocusManager::new();
    let footer = "Tab: Switch controls | Space/Enter: Click | Q/Esc: Quit";
    for (width, height) in [(100, 32), (80, 24), (120, 50), (100, 32)] {
        let buffer = (0..5)
            .find_map(|_| {
                reset_focus_counter();
                begin_frame();
                let node = manager.with_context(|| {
                    view! {
                        <View width=Size::Fill height=Size::Fill
                            foreground_color=Color::White background_color=Color::White>
                            <LayoutDemo />
                        </View>
                    }
                });
                end_frame();
                let mut buffer = Buffer::new(width, height);
                let mut hits = HitMap::new();
                manager.with_context(|| {
                    render_node_interactive(
                        &node,
                        Rect::new(0, 0, width, height),
                        &mut buffer,
                        &mut hits,
                    )
                });
                if !manager.reconcile(hits.interactions) {
                    return Some(buffer);
                }
                None
            })
            .expect("layout focus did not stabilize");
        let (x, y) = (0..height)
            .find_map(|y| {
                let row: String = (0..width).map(|x| buffer.get(x, y).unwrap().ch).collect();
                row.find(footer)
                    .map(|byte| (row[..byte].chars().count() as u16, y))
            })
            .unwrap_or_else(|| panic!("footer missing at {width}x{height}"));
        for offset in 0..footer.len() as u16 {
            let cell = buffer.get(x + offset, y).unwrap();
            assert_eq!(
                cell.fg,
                Some(Color::White),
                "at {width}x{height}, cell {offset}"
            );
            assert_eq!(
                cell.bg,
                Some(Color::White),
                "at {width}x{height}, cell {offset}"
            );
            assert_eq!(cell.fg, cell.bg, "foreground and background must match");
        }
    }
}
