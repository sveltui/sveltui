use super::*;
use sveltui::element::reset_focus_counter;
use sveltui::element::taffy_bridge::{ComputedLayoutNode, LayoutEngine};

fn layout(width: u16, height: u16) -> ComputedLayoutNode {
    reset_focus_counter();
    begin_frame();
    let node = view! { <GridDemo /> };
    end_frame();
    LayoutEngine::new().compute_layout(&node, width, height)
}

#[test]
fn header_and_footer_span_the_sidebar_and_main_area() {
    let root = layout(100, 30);
    let [header, sidebar, main, footer] = &root.children[..] else {
        panic!("four areas")
    };
    assert_eq!((header.rect.width, header.rect.height), (100, 3));
    assert_eq!((sidebar.rect.x, sidebar.rect.width), (0, 22));
    assert_eq!((main.rect.x, main.rect.width), (23, 77));
    assert_eq!((footer.rect.y, footer.rect.width), (29, 100));
}

#[test]
fn services_reflow_into_as_many_columns_as_fit() {
    let columns = |width| {
        let root = layout(width, 40);
        let cards = &root.children[2].children[0].children;
        let first_row = cards[0].rect.y;
        cards.iter().filter(|c| c.rect.y == first_row).count()
    };
    // The services panel is the screen minus the sidebar, gap, border and
    // padding; each column needs 18 cells plus a 1-cell gap.
    assert_eq!(columns(120), 4, "93 cells inside fit four columns");
    assert_eq!(columns(70), 2, "43 cells inside fit two");
}
