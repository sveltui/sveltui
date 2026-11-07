use super::*;

#[test]
fn reads_the_long_form() {
    assert_eq!(
        hex_color("#ff8800"),
        Color::Rgb {
            r: 255,
            g: 136,
            b: 0
        }
    );
    assert_eq!(hex_color("#000000"), Color::Rgb { r: 0, g: 0, b: 0 });
}

#[test]
fn short_form_doubles_each_digit() {
    assert_eq!(hex_color("#f80"), hex_color("#ff8800"));
    assert_eq!(
        hex_color("#fff"),
        Color::Rgb {
            r: 255,
            g: 255,
            b: 255
        }
    );
}

#[test]
fn ignores_case() {
    assert_eq!(hex_color("#7AA2F7"), hex_color("#7aa2f7"));
    assert_eq!(
        hex_color("#7aA2f7"),
        Color::Rgb {
            r: 122,
            g: 162,
            b: 247
        }
    );
}

#[test]
fn works_in_a_const() {
    const ACCENT: Color = hex_color("#7aa2f7");
    assert_eq!(
        ACCENT,
        Color::Rgb {
            r: 122,
            g: 162,
            b: 247
        }
    );
}

#[test]
#[should_panic(expected = "invalid hex color")]
fn needs_the_hash() {
    hex_color("ff8800");
}

#[test]
#[should_panic(expected = "invalid hex color")]
fn rejects_other_lengths() {
    hex_color("#ff88");
}

#[test]
#[should_panic(expected = "invalid hex color")]
fn rejects_non_hex_digits() {
    hex_color("#ff88z0");
}

#[test]
#[should_panic(expected = "invalid hex color")]
fn rejects_an_empty_string() {
    hex_color("");
}
