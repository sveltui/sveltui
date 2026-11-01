use sveltui_code::highlight::{
    DEFAULT_THEME, HighlightedSpan, clip_spans, find_syntax, find_theme, highlight_text,
};

#[test]
fn test_theme_and_syntax_discovery() {
    let theme = find_theme(Some(DEFAULT_THEME));
    assert_eq!(theme.name.as_deref(), Some("Base16 Ocean Dark"));

    let rs_syntax = find_syntax(Some("rs"));
    assert_eq!(rs_syntax.name, "Rust");

    let py_syntax = find_syntax(Some("py"));
    assert_eq!(py_syntax.name, "Python");

    let json_syntax = find_syntax(Some("json"));
    assert_eq!(json_syntax.name, "JSON");

    let fallback_syntax = find_syntax(Some("unknown_lang_extension"));
    assert_eq!(fallback_syntax.name, "Plain Text");
}

#[test]
fn test_rust_highlighting() {
    let code = "fn main() {\n    let answer = 42;\n}";
    let lines = highlight_text(code, Some("rs"), Some(DEFAULT_THEME));

    assert_eq!(lines.len(), 3);

    // Line 0: "fn main() {"
    let line0_text: String = lines[0].iter().map(|s| s.text.as_str()).collect();
    assert_eq!(line0_text, "fn main() {");

    // The first token should be keyword "fn"
    assert_eq!(lines[0][0].text, "fn");
    // Verify it got a colored token (not plain dark grey or uncolored)
    assert!(lines[0].len() > 1);

    // Line 1: "    let answer = 42;"
    let line1_text: String = lines[1].iter().map(|s| s.text.as_str()).collect();
    assert_eq!(line1_text, "    let answer = 42;");

    // Line 2: "}"
    let line2_text: String = lines[2].iter().map(|s| s.text.as_str()).collect();
    assert_eq!(line2_text, "}");
}

#[test]
fn test_trailing_newline_and_empty_text() {
    let empty_lines = highlight_text("", Some("rs"), None);
    assert_eq!(empty_lines.len(), 1);
    assert!(empty_lines[0].is_empty());

    let trailing = highlight_text("let x = 1;\n", Some("rs"), None);
    assert_eq!(trailing.len(), 2);
    assert!(trailing[1].is_empty());
}

#[test]
fn test_clip_spans_horizontal_scrolling() {
    let spans = vec![
        HighlightedSpan::plain("fn "),
        HighlightedSpan::plain("main"),
        HighlightedSpan::plain("() {"),
    ];

    // Total text is "fn main() {" (len 11)
    // Clip from col 3 with width 4 -> "main"
    let clipped = clip_spans(&spans, 3, 4);
    let result_text: String = clipped.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(result_text, "main");

    // Clip from col 1 with width 5 -> "n mai"
    let clipped2 = clip_spans(&spans, 1, 5);
    let result_text2: String = clipped2.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(result_text2, "n mai");

    // Clip past the end -> empty
    let clipped3 = clip_spans(&spans, 20, 5);
    assert!(clipped3.is_empty());

    // Clip with width 0 -> empty
    let clipped4 = clip_spans(&spans, 0, 0);
    assert!(clipped4.is_empty());
}

#[test]
fn test_highlight_caching() {
    let code = "fn compute() -> i32 { 100 }";
    let first = highlight_text(code, Some("rs"), None);
    let second = highlight_text(code, Some("rs"), None);
    assert_eq!(first, second);
}
