//! Draws a grid's rules: lines in the gaps between its tracks that join
//! each other and the grid's border.

use crate::element::node::{VNode, ViewElement};
use crate::element::taffy_bridge::ComputedLayoutNode;
use crate::layout::{BorderStyle, Rect, Side};
use crate::terminal::Buffer;

const VERTICAL: u8 = 1;
const HORIZONTAL: u8 = 2;

/// Spans of a line hidden by cells that cross it, as `start..end` along the
/// line, in content coordinates. One list per line.
fn covered_spans(
    lines: &[u16],
    cells: &[Rect],
    across: impl Fn(&Rect) -> (u16, u16),
    along: impl Fn(&Rect) -> (u16, u16),
) -> Vec<Vec<(u16, u16)>> {
    let mut spans = vec![Vec::new(); lines.len()];
    for cell in cells {
        let (start, end) = across(cell);
        // Only a cell that spans two tracks can sit over the gap between
        // them. The lines are sorted, so we jump to the first one it might
        // cover.
        let first = lines.partition_point(|&line| line < start);
        for (index, &line) in lines.iter().enumerate().skip(first) {
            if line >= end {
                break;
            }
            spans[index].push(along(cell));
        }
    }
    spans
}

fn is_covered(spans: &[(u16, u16)], at: i32) -> bool {
    spans
        .iter()
        .any(|&(start, end)| i32::from(start) <= at && at < i32::from(end))
}

/// Draws `view`'s grid rules into `buffer`. `x`/`y` is the view's
/// top-left corner on screen and `scroll` its scroll offset, so rules
/// move with the content they separate.
pub(crate) fn draw_grid_rules(
    buffer: &mut Buffer,
    view: &ViewElement,
    layout: &ComputedLayoutNode,
    (x, y): (i32, i32),
    (width, height): (u16, u16),
    (scroll_x, scroll_y): (u16, u16),
) {
    let (Some(visual), Some(lines)) = (view.visual.as_ref(), layout.rules.as_deref()) else {
        return;
    };
    let style = visual.grid_rules;
    if style == BorderStyle::None {
        return;
    }
    let axes = visual.grid_rule_axes;
    let border = visual.border;
    let edge = i32::from(border != BorderStyle::None);

    // Rules fill the inside of the border, padding included, so they meet it.
    let (left, top) = (x + edge, y + edge);
    let (right, bottom) = (x + i32::from(width) - edge, y + i32::from(height) - edge);
    let clip = buffer.active_clip();
    let visible_left = left.max(i32::from(clip.x)).max(0);
    let visible_top = top.max(i32::from(clip.y)).max(0);
    let visible_right = right
        .min(i32::from(clip.x) + i32::from(clip.width))
        .min(i32::from(buffer.width));
    let visible_bottom = bottom
        .min(i32::from(clip.y) + i32::from(clip.height))
        .min(i32::from(buffer.height));
    if visible_left >= visible_right || visible_top >= visible_bottom {
        return;
    }

    let columns: &[u16] = if axes.columns() { &lines.columns } else { &[] };
    let rows: &[u16] = if axes.rows() { &lines.rows } else { &[] };

    // Separator children are lines themselves: they cross the column lines
    // instead of hiding them.
    let is_separator = |child: &VNode| matches!(child, VNode::View(v) if v.visual.as_ref().is_some_and(|vis| vis.separator));
    let (separators, cells): (Vec<_>, Vec<_>) = view
        .children
        .iter()
        .zip(&layout.children)
        .partition(|(child, _)| is_separator(child));
    let cells: Vec<Rect> = cells.into_iter().map(|(_, l)| l.rect).collect();
    let column_cover = covered_spans(
        columns,
        &cells,
        |r| (r.x, r.x + r.width),
        |r| (r.y, r.y + r.height),
    );
    let row_cover = covered_spans(
        rows,
        &cells,
        |r| (r.y, r.y + r.height),
        |r| (r.x, r.x + r.width),
    );

    // Mark which visible cells carry a vertical or horizontal line.
    let cols = (visible_right - visible_left) as usize;
    let mut marks = vec![0u8; cols * (visible_bottom - visible_top) as usize];
    let index =
        |sx: i32, sy: i32| (sy - visible_top) as usize * cols + (sx - visible_left) as usize;
    let content_x = |sx: i32| sx - x + i32::from(scroll_x);
    let content_y = |sy: i32| sy - y + i32::from(scroll_y);
    for (line, cover) in columns.iter().zip(&column_cover) {
        let sx = x + i32::from(*line) - i32::from(scroll_x);
        if (visible_left..visible_right).contains(&sx) {
            for sy in visible_top..visible_bottom {
                if !is_covered(cover, content_y(sy)) {
                    marks[index(sx, sy)] |= VERTICAL;
                }
            }
        }
    }
    for (line, cover) in rows.iter().zip(&row_cover) {
        let sy = y + i32::from(*line) - i32::from(scroll_y);
        if (visible_top..visible_bottom).contains(&sy) {
            for sx in visible_left..visible_right {
                if !is_covered(cover, content_x(sx)) {
                    marks[index(sx, sy)] |= HORIZONTAL;
                }
            }
        }
    }

    for (_, separator) in &separators {
        let sy = y + i32::from(separator.rect.y) - i32::from(scroll_y);
        if (visible_top..visible_bottom).contains(&sy) {
            for sx in visible_left..visible_right {
                marks[index(sx, sy)] |= HORIZONTAL;
            }
        }
    }

    // A line continues past the visible area (or into the border) unless
    // something stopped it there.
    let mark = |sx: i32, sy: i32, flag: u8, fallback: bool| {
        if (visible_left..visible_right).contains(&sx)
            && (visible_top..visible_bottom).contains(&sy)
        {
            marks[index(sx, sy)] & flag != 0
        } else {
            fallback
        }
    };
    let bordered = edge == 1;
    let color = visual.border_color;
    for sy in visible_top..visible_bottom {
        for sx in visible_left..visible_right {
            let here = marks[index(sx, sy)];
            if here == 0 {
                continue;
            }
            let vertical = here & VERTICAL != 0;
            let horizontal = here & HORIZONTAL != 0;
            // Past the inner edge is the border: a line reaches it only when
            // there is one. Past the visible edge, we assume it carries on.
            let up = vertical && mark(sx, sy - 1, VERTICAL, sy > top || bordered);
            let down = vertical && mark(sx, sy + 1, VERTICAL, sy + 1 < bottom || bordered);
            let left_side = horizontal && mark(sx - 1, sy, HORIZONTAL, sx > left || bordered);
            let right_side = horizontal && mark(sx + 1, sy, HORIZONTAL, sx + 1 < right || bordered);
            let ch = style.junction(up, down, left_side, right_side);
            let bg = buffer.get(sx as u16, sy as u16).and_then(|c| c.bg);
            buffer.set_char(sx as u16, sy as u16, ch, color, bg, false);
        }
    }

    // Where a line reaches the border, the border gets a tee.
    if bordered {
        let tee = |buffer: &mut Buffer, sx: i32, sy: i32, side: Side| {
            if sx >= 0 && sy >= 0 && (sx as u16) < buffer.width && (sy as u16) < buffer.height {
                let bg = buffer.get(sx as u16, sy as u16).and_then(|c| c.bg);
                buffer.set_char(
                    sx as u16,
                    sy as u16,
                    border.tee(style, side),
                    color,
                    bg,
                    false,
                );
            }
        };
        // The title sits in the top border, two cells in, as ` title `;
        // a tee there would cut through it.
        let title = visual
            .title
            .as_deref()
            .filter(|t| width > 4 && !t.is_empty())
            .map(|t| {
                let length = (t.chars().count() + 2).min(usize::from(width) - 4) as i32;
                (x + 2)..(x + 2 + length)
            });
        for sx in visible_left..visible_right {
            let under_title = title.as_ref().is_some_and(|t| t.contains(&sx));
            if visible_top == top && !under_title && mark(sx, top, VERTICAL, false) {
                tee(buffer, sx, y, Side::Top);
            }
            if visible_bottom == bottom && mark(sx, bottom - 1, VERTICAL, false) {
                tee(buffer, sx, bottom, Side::Bottom);
            }
        }
        for sy in visible_top..visible_bottom {
            if visible_left == left && mark(left, sy, HORIZONTAL, false) {
                tee(buffer, x, sy, Side::Left);
            }
            if visible_right == right && mark(right - 1, sy, HORIZONTAL, false) {
                tee(buffer, right, sy, Side::Right);
            }
        }
    }
}
