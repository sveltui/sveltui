use super::*;

fn text(content: &str, wrap: bool) -> TextElement {
    TextElement {
        content: content.to_owned(),
        wrap,
        ..Default::default()
    }
}

/// What `size` has to agree with: the size of the lines we actually paint.
fn painted(text: &TextElement, width: Option<u16>) -> (usize, usize) {
    let lines = lines(text, width);
    let widest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    (widest, lines.len())
}

#[test]
fn size_matches_the_painted_lines() {
    let long = "x y z ".repeat(40);
    let samples = [
        "",
        "a",
        "Once upon a midnight dreary, while I pondered, weak and weary",
        "supercalifragilisticexpialidocious and more",
        "  leading and   doubled   spaces  ",
        "two\nlines",
        "trailing newline\n",
        "\n\nblank lines first",
        "naïve café — unicode words",
        long.as_str(),
    ];
    for sample in samples {
        for wrap in [false, true] {
            let text = text(sample, wrap);
            for width in [
                None,
                Some(0),
                Some(1),
                Some(2),
                Some(3),
                Some(5),
                Some(8),
                Some(13),
                Some(40),
                Some(200),
            ] {
                assert_eq!(
                    size(&text, width),
                    painted(&text, width),
                    "{sample:?}, wrap {wrap}, width {width:?}"
                );
            }
        }
    }
}

#[test]
fn truncated_text_falls_back_to_the_real_lines() {
    let mut text = text("one two three four five six", true);
    text.max_lines = Some(2);
    assert_eq!(size(&text, Some(8)), painted(&text, Some(8)));
}
