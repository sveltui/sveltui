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
