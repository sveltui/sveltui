use std::rc::Rc;
use sveltui::{
    Buffer,
    element::{render_node_interactive, reset_focus_counter},
    layout::Rect,
    prelude::*,
    terminal::ImagePaint,
};

struct Dot;

impl ImagePaint for Dot {
    fn key(&self) -> u64 {
        1
    }

    fn draw(&self, _: (u16, u16), _: Rect, _: (u16, u16)) -> Vec<u8> {
        Vec::new()
    }
}

fn draw(node: &VNode, width: u16, height: u16) -> Buffer {
    let mut buffer = Buffer::new(width, height);
    let mut hits = HitMap::new();
    reset_focus_counter();
    render_node_interactive(node, Rect::new(0, 0, width, height), &mut buffer, &mut hits);
    buffer
}

fn picture() -> ViewElement {
    ViewElement::new()
        .width(Dimension::Length(8.0))
        .height(Dimension::Length(4.0))
        .image(Rc::new(Dot))
}

#[test]
fn a_view_with_an_image_places_it_over_its_cells() {
    let node = ViewElement::new()
        .padding(Padding::only(2, 0, 0, 3))
        .child(picture())
        .into_node();
    let buffer = draw(&node, 30, 10);
    assert_eq!(buffer.images.len(), 1);
    let image = &buffer.images[0];
    assert_eq!(image.at, (3, 2));
    assert_eq!(image.size, (8, 4));
    assert_eq!(image.visible, Rect::new(0, 0, 8, 4));
}

#[test]
fn a_scrolled_view_clips_the_image_to_what_shows() {
    // The picture is 4 rows tall in a 2-row window, scrolled down one row:
    // rows 1 and 2 of the image are what shows.
    let node = ViewElement::new()
        .height(Dimension::Length(2.0))
        .overflow(Overflow::Hidden)
        .child(
            ViewElement::new()
                .margin(Margin::only(0, 0, 0, 0))
                .child(picture()),
        )
        .into_node();
    let buffer = draw(&node, 30, 10);
    let image = &buffer.images[0];
    assert_eq!(image.visible.height, 2);
    assert_eq!(image.size, (8, 4));
}
