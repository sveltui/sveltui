use crate::element::TextElement;

#[cfg(test)]
mod text_tests;

/// Shared by measurement and painting so wrapped rows receive enough height.
pub(crate) fn lines(text: &TextElement, width: Option<u16>) -> Vec<String> {
    let mut output = wrapped_lines(text, width);
    if let Some(max_lines) = text.max_lines
        && output.len() > max_lines
    {
        output.truncate(max_lines);
        if let Some(last) = output.last_mut() {
            let keep = width.map_or(usize::MAX, |width| usize::from(width).saturating_sub(1));
            *last = last
                .chars()
                .take(keep)
                .collect::<String>()
                .trim_end()
                .to_owned();
            last.push('…');
        }
    }
    output
}

fn wrapped_lines(text: &TextElement, width: Option<u16>) -> Vec<String> {
    if width == Some(0) || text.max_lines == Some(0) {
        return Vec::new();
    }
    let source: Vec<_> = if text.content.is_empty() {
        vec![""]
    } else {
        text.content.lines().collect()
    };
    let Some(width) = width.filter(|_| text.wrap) else {
        return source.into_iter().map(str::to_owned).collect();
    };
    let width = usize::from(width);
    let mut output = Vec::new();
    for source_line in source {
        let mut line = String::new();
        let mut length = 0;
        for word in source_line.split_whitespace() {
            let word_length = word.chars().count();
            if length > 0 && length + 1 + word_length > width {
                output.push(std::mem::take(&mut line));
                length = 0;
            }
            if length > 0 {
                line.push(' ');
                length += 1;
            }
            for ch in word.chars() {
                if length == width {
                    output.push(std::mem::take(&mut line));
                    length = 0;
                }
                line.push(ch);
                length += 1;
            }
        }
        output.push(line);
    }
    output
}

/// The size `lines` would produce, as (widest line, line count), without
/// building the lines. Layout asks for this several times per text node, so
/// skipping the allocations matters. Truncated text (`max_lines`) is rare
/// and needs the real lines for its ellipsis, so it falls back to `lines`.
pub(crate) fn size(text: &TextElement, width: Option<u16>) -> (usize, usize) {
    if text.max_lines.is_some() {
        let lines = lines(text, width);
        let widest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        return (widest, lines.len());
    }
    if width == Some(0) {
        return (0, 0);
    }
    if text.content.is_empty() {
        return (0, 1);
    }
    let mut widest = 0;
    let mut count = 0;
    let Some(width) = width.filter(|_| text.wrap).map(usize::from) else {
        for line in text.content.lines() {
            widest = widest.max(line.chars().count());
            count += 1;
        }
        return (widest, count);
    };
    // The same walk as `wrapped_lines`, counting instead of collecting.
    let mut emit = |length: usize| {
        widest = widest.max(length);
        count += 1;
    };
    for source_line in text.content.lines() {
        let mut length = 0;
        for word in source_line.split_whitespace() {
            let word_length = word.chars().count();
            if length > 0 && length + 1 + word_length > width {
                emit(length);
                length = 0;
            }
            if length > 0 {
                length += 1;
            }
            for _ in 0..word_length {
                if length == width {
                    emit(length);
                    length = 0;
                }
                length += 1;
            }
        }
        emit(length);
    }
    (widest, count)
}
