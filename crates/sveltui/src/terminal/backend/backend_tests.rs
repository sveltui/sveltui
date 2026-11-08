use super::*;

#[test]
fn trailing_menu_spaces_keep_the_highlight_background() {
    let mut current = Buffer::new(3, 1);
    current.set_char(0, 0, 'A', Some(Color::Black), Some(Color::White), false);
    current.set_char(1, 0, ' ', None, Some(Color::White), false);
    current.set_char(2, 0, ' ', None, Some(Color::White), false);
    let mut output = Vec::new();
    write_buffer_diff(&mut output, &current, &Buffer::new(3, 1), true).unwrap();
    let output = String::from_utf8(output).unwrap();

    // Resetting just the foreground must preserve the row's white
    // background when the label ends and unstyled padding begins.
    let padding = format!("{}{} ", MoveTo(1, 0), SetForegroundColor(Color::Reset));
    assert!(output.contains(&padding), "{output:?}");
}

#[test]
fn resetting_background_preserves_the_foreground() {
    let mut current = Buffer::new(2, 1);
    current.set_char(0, 0, 'A', Some(Color::White), Some(Color::Black), false);
    current.set_char(1, 0, 'B', Some(Color::White), None, false);
    let mut output = Vec::new();
    write_buffer_diff(&mut output, &current, &Buffer::new(2, 1), true).unwrap();
    let output = String::from_utf8(output).unwrap();
    let next = format!("{}{}B", MoveTo(1, 0), SetBackgroundColor(Color::Reset));
    assert!(output.contains(&next), "{output:?}");
}

#[test]
fn a_frame_flushes_terminal_output_once() {
    #[derive(Default)]
    struct Output {
        bytes: Vec<u8>,
        writes: usize,
        flushes: usize,
    }
    impl Write for Output {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            self.writes += 1;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }
    let mut output = Output::default();
    let mut buffer = Buffer::new(40, 10);
    for cell in &mut buffer.cells {
        cell.ch = 'X';
        cell.fg = Some(Color::White);
    }
    write_buffer_diff(&mut output, &buffer, &Buffer::new(40, 10), true).unwrap();
    assert_eq!(output.flushes, 1);
    assert_eq!(output.writes, 1);
    assert_eq!(
        output.bytes.iter().filter(|&&byte| byte == b'X').count(),
        400
    );
}

#[test]
fn pointer_shape_is_sent_as_osc_22() {
    let mut out = Vec::new();
    write_pointer_shape(&mut out, PointerShape::ColResize, PointerDialect::Css).unwrap();
    assert_eq!(out, b"\x1b]22;col-resize\x1b\\");
}

#[test]
fn kitty_gets_its_own_resize_names() {
    let mut out = Vec::new();
    write_pointer_shape(&mut out, PointerShape::ColResize, PointerDialect::Kitty).unwrap();
    assert_eq!(out, b"\x1b]22;ew-resize\x1b\\");
    let mut out = Vec::new();
    write_pointer_shape(&mut out, PointerShape::RowResize, PointerDialect::Kitty).unwrap();
    assert_eq!(out, b"\x1b]22;ns-resize\x1b\\");
}

#[test]
fn the_terminal_is_recognised_from_its_environment() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        }
    };
    assert_eq!(
        PointerDialect::from_env(env(&[("TERM", "xterm-kitty")])),
        PointerDialect::Kitty
    );
    assert_eq!(
        PointerDialect::from_env(env(&[("KITTY_WINDOW_ID", "3"), ("TERM", "xterm-256color")])),
        PointerDialect::Kitty
    );
    assert_eq!(
        PointerDialect::from_env(env(&[
            ("TERM", "xterm-ghostty"),
            ("TERM_PROGRAM", "ghostty")
        ])),
        PointerDialect::Css
    );
    assert_eq!(
        PointerDialect::from_env(env(&[
            ("XTERM_VERSION", "XTerm(390)"),
            ("TERM", "xterm-256color")
        ])),
        PointerDialect::Xterm
    );
    assert_eq!(
        PointerDialect::from_env(env(&[
            ("TERM", "xterm-kitty"),
            ("SVELTUI_POINTER_DIALECT", "css")
        ])),
        PointerDialect::Css
    );
}

mod images {
    use super::*;
    use crate::layout::Rect;
    use crate::terminal::image::{ImagePaint, ImagePlacement, plan_images};
    use std::rc::Rc;

    struct Paint {
        key: u64,
        erasable: bool,
    }

    impl ImagePaint for Paint {
        fn key(&self) -> u64 {
            self.key
        }

        fn draw(&self, size: (u16, u16), visible: Rect, _: (u16, u16)) -> Vec<u8> {
            format!("<{}:{}x{}@{}>", self.key, size.0, size.1, visible.x).into_bytes()
        }

        fn erase(&self) -> Option<Vec<u8>> {
            self.erasable
                .then(|| format!("<erase {}>", self.key).into_bytes())
        }
    }

    fn placed(key: u64, at: (u16, u16), erasable: bool) -> ImagePlacement {
        ImagePlacement {
            at,
            size: (4, 2),
            visible: Rect::new(0, 0, 4, 2),
            paint: Rc::new(Paint { key, erasable }),
        }
    }

    #[test]
    fn a_new_image_is_drawn_and_an_unchanged_one_is_left_alone() {
        let image = placed(1, (2, 1), false);
        let plan = plan_images(&[], std::slice::from_ref(&image), |_| false, false);
        assert_eq!(plan.draw, [0]);
        let plan = plan_images(
            std::slice::from_ref(&image),
            std::slice::from_ref(&image),
            |_| false,
            false,
        );
        assert!(plan.draw.is_empty() && plan.erase.is_empty() && plan.repaint.is_empty());
    }

    #[test]
    fn cells_changing_under_an_image_draw_it_again() {
        let image = placed(1, (2, 1), false);
        let plan = plan_images(
            std::slice::from_ref(&image),
            std::slice::from_ref(&image),
            |rect| rect == Rect::new(2, 1, 4, 2),
            false,
        );
        assert_eq!(plan.draw, [0]);
    }

    #[test]
    fn a_full_repaint_draws_every_image() {
        let image = placed(1, (0, 0), false);
        let plan = plan_images(
            std::slice::from_ref(&image),
            std::slice::from_ref(&image),
            |_| false,
            true,
        );
        assert_eq!(plan.draw, [0]);
    }

    #[test]
    fn a_moved_image_is_removed_where_it_was() {
        let before = placed(1, (0, 0), false);
        let after = placed(1, (5, 0), false);
        let plan = plan_images(&[before], std::slice::from_ref(&after), |_| false, false);
        assert_eq!(plan.repaint, [Rect::new(0, 0, 4, 2)]);
        assert_eq!(plan.draw, [0]);

        let before = placed(2, (0, 0), true);
        let plan = plan_images(&[before], &[], |_| false, false);
        assert_eq!(plan.erase, [b"<erase 2>".to_vec()]);
        assert!(plan.repaint.is_empty());
    }

    #[test]
    fn drawing_moves_to_the_image_then_writes_its_bytes() {
        let images = [placed(7, (3, 2), false)];
        let mut out = Vec::new();
        write_images(&mut out, &images, &[0], (8, 16)).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!("{}<7:4x2@0>", MoveTo(3, 2))
        );
    }

    #[test]
    fn a_buffer_drops_images_a_popover_covers() {
        let mut buffer = Buffer::new(20, 10);
        buffer.place_image(placed(1, (0, 0), false));
        buffer.place_image(placed(2, (10, 5), false));
        buffer.occlude(Rect::new(9, 4, 4, 4));
        assert_eq!(buffer.images.len(), 1);
        assert_eq!(buffer.images[0].paint.key(), 1);
        buffer.clear();
        assert!(buffer.images.is_empty());
    }
}
