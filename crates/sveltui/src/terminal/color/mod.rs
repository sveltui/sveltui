use crossterm::style::Color;

#[cfg(test)]
mod color_tests;

const INVALID: &str = "invalid hex color: expected \"#rgb\" or \"#rrggbb\"";

/// Builds an RGB color from a hex string, `#rgb` or `#rrggbb`, in either
/// case. In the short form each digit doubles, so `#f80` is `#ff8800`.
///
/// It panics on anything else, since a bad color is almost always a typo.
/// It's a `const fn`, so when we use it in a `const` that typo turns into a
/// build error instead.
///
/// ```
/// use sveltui::{Color, hex_color};
///
/// const ACCENT: Color = hex_color("#7aa2f7");
/// assert_eq!(hex_color("#f80"), Color::Rgb { r: 255, g: 136, b: 0 });
/// ```
///
/// ```compile_fail
/// const BROKEN: sveltui::Color = sveltui::hex_color("#ff88z0");
/// ```
pub const fn hex_color(hex: &str) -> Color {
    let bytes = hex.as_bytes();
    if bytes.is_empty() || bytes[0] != b'#' {
        panic!("{}", INVALID);
    }
    match bytes.len() {
        4 => Color::Rgb {
            r: nibble(bytes[1]) * 17,
            g: nibble(bytes[2]) * 17,
            b: nibble(bytes[3]) * 17,
        },
        7 => Color::Rgb {
            r: nibble(bytes[1]) * 16 + nibble(bytes[2]),
            g: nibble(bytes[3]) * 16 + nibble(bytes[4]),
            b: nibble(bytes[5]) * 16 + nibble(bytes[6]),
        },
        _ => panic!("{}", INVALID),
    }
}

/// Reads one hex digit, in either case.
const fn nibble(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        b'a'..=b'f' => digit - b'a' + 10,
        b'A'..=b'F' => digit - b'A' + 10,
        _ => panic!("{}", INVALID),
    }
}
