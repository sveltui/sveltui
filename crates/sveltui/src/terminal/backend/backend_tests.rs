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
