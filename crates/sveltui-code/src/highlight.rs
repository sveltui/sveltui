use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

thread_local! {
    static HIGHLIGHT_CACHE: RefCell<Vec<(u64, Vec<Vec<HighlightedSpan>>)>> = const { RefCell::new(Vec::new()) };
}

fn cache_key(text: &str, language: Option<&str>, theme_name: Option<&str>) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    language.hash(&mut hasher);
    theme_name.hash(&mut hasher);
    hasher.finish()
}

pub const DEFAULT_THEME: &str = "base16-ocean.dark";

pub fn syntax_set() -> &'static SyntaxSet {
    &SYNTAX_SET
}

pub fn theme_set() -> &'static ThemeSet {
    &THEME_SET
}

pub fn find_syntax(language: Option<&str>) -> &'static SyntaxReference {
    let ps = syntax_set();
    if let Some(lang) = language {
        let clean = lang.trim();
        if !clean.is_empty() {
            if let Some(syntax) = ps.find_syntax_by_token(clean) {
                return syntax;
            }
            if let Some(syntax) = ps.find_syntax_by_extension(clean) {
                return syntax;
            }
            if let Some(syntax) = ps.find_syntax_by_name(clean) {
                return syntax;
            }
        }
    }
    ps.find_syntax_plain_text()
}

pub fn find_theme(theme_name: Option<&str>) -> &'static Theme {
    let ts = theme_set();
    let name = theme_name.unwrap_or(DEFAULT_THEME);
    if let Some(theme) = ts.themes.get(name) {
        return theme;
    }
    if let Some(theme) = ts.themes.get(DEFAULT_THEME) {
        return theme;
    }
    ts.themes
        .values()
        .next()
        .expect("At least one default theme must exist in syntect")
}

pub fn syntect_color_to_crossterm(c: syntect::highlighting::Color) -> Color {
    Color::Rgb {
        r: c.r,
        g: c.g,
        b: c.b,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HighlightedSpan {
    pub text: String,
    pub fg: Color,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl HighlightedSpan {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            fg: Color::White,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

pub fn highlight_text(
    text: &str,
    language: Option<&str>,
    theme_name: Option<&str>,
) -> Vec<Vec<HighlightedSpan>> {
    if text.is_empty() {
        return vec![Vec::new()];
    }

    let key = cache_key(text, language, theme_name);
    let cached = HIGHLIGHT_CACHE.with(|c| {
        let cache = c.borrow();
        cache
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
    });
    if let Some(res) = cached {
        return res;
    }

    let ps = syntax_set();
    let syntax = find_syntax(language);
    let theme = find_theme(theme_name);
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut result = Vec::new();

    for line in LinesWithEndings::from(text) {
        let ranges = highlighter.highlight_line(line, ps).unwrap_or_default();
        let mut spans = Vec::new();
        for (style, segment) in ranges {
            let clean_text = segment.trim_end_matches(['\r', '\n']);
            if clean_text.is_empty() {
                continue;
            }
            spans.push(HighlightedSpan {
                text: clean_text.to_string(),
                fg: syntect_color_to_crossterm(style.foreground),
                bg: if style.background.a > 0 {
                    Some(syntect_color_to_crossterm(style.background))
                } else {
                    None
                },
                bold: style.font_style.contains(FontStyle::BOLD),
                italic: style.font_style.contains(FontStyle::ITALIC),
                underline: style.font_style.contains(FontStyle::UNDERLINE),
            });
        }
        result.push(spans);
    }

    if text.ends_with('\n') {
        result.push(Vec::new());
    }

    HIGHLIGHT_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.len() >= 32 {
            cache.remove(0);
        }
        cache.push((key, result.clone()));
    });

    result
}

pub fn clip_spans(
    spans: &[HighlightedSpan],
    scroll_col: usize,
    max_width: usize,
) -> Vec<HighlightedSpan> {
    if max_width == 0 {
        return Vec::new();
    }
    let end_col = scroll_col + max_width;
    let mut current_col = 0;
    let mut clipped = Vec::new();

    for span in spans {
        let span_chars = span.text.chars().count();
        let span_start = current_col;
        let span_end = current_col + span_chars;
        current_col = span_end;

        if span_end <= scroll_col || span_start >= end_col {
            continue;
        }

        let slice_start = scroll_col.saturating_sub(span_start);
        let slice_end = (end_col.saturating_sub(span_start)).min(span_chars);

        if slice_start < slice_end {
            let sliced_text: String = span
                .text
                .chars()
                .skip(slice_start)
                .take(slice_end - slice_start)
                .collect();
            if !sliced_text.is_empty() {
                let mut new_span = span.clone();
                new_span.text = sliced_text;
                clipped.push(new_span);
            }
        }
    }

    clipped
}
