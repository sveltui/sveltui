use crate::element::events::{HitMap, HitTarget, LayoutTarget, ScrollRegion};
use crate::element::focus::next_render_focus_id;
use crate::element::measure::measure_node;
use crate::element::node::{OverlayElement, VNode};
use crate::element::scroll::get_scroll;
use crate::element::taffy_bridge::ComputedLayoutNode;
use crate::layout::{BorderStyle, Rect, compute_popover_rect};
use crate::terminal::Buffer;
use crossterm::style::Color;
use std::rc::Rc;

fn scrollbar_thumb(track: u16, viewport: u16, content: u16, offset: u16) -> std::ops::Range<u16> {
    if track == 0 {
        return 0..0;
    }
    let length = (u32::from(track) * u32::from(viewport) / u32::from(content.max(1)))
        .clamp(1, u32::from(track)) as u16;
    let max_scroll = content.saturating_sub(viewport);
    let start = if max_scroll == 0 {
        0
    } else {
        (u32::from(offset.min(max_scroll)) * u32::from(track - length) / u32::from(max_scroll))
            as u16
    };
    start..start + length
}

struct PendingOverlay {
    overlay: OverlayElement,
    anchor_rect: Option<Rect>,
    foreground: Option<Color>,
}

struct RenderContext<'a> {
    buffer: &'a mut Buffer,
    hit_map: &'a mut HitMap,
    layer: Option<crate::element::FocusId>,
    scrolls: Vec<ScrollRegion>,
    overlays: &'a mut Vec<PendingOverlay>,
    active_fg: Option<Color>,
    active_bg: Option<Color>,
}

// Intersect signed layout coordinates before converting to terminal cells.
// Negative scroll offsets must reduce the visible size, not move a full-sized
// hit target to the origin.
fn visible_rect(x: i32, y: i32, width: u16, height: u16, clip: Rect) -> Rect {
    let left = x.max(i32::from(clip.x));
    let top = y.max(i32::from(clip.y));
    let right = (x + i32::from(width)).min(i32::from(clip.x) + i32::from(clip.width));
    let bottom = (y + i32::from(height)).min(i32::from(clip.y) + i32::from(clip.height));
    Rect::new(
        left.max(0).min(i32::from(u16::MAX)) as u16,
        top.max(0).min(i32::from(u16::MAX)) as u16,
        right.saturating_sub(left).max(0) as u16,
        bottom.saturating_sub(top).max(0) as u16,
    )
}

fn register_target(ctx: &mut RenderContext<'_>, x: i32, y: i32, mut target: HitTarget) {
    target.rect = visible_rect(
        x,
        y,
        target.rect.width,
        target.rect.height,
        ctx.buffer.active_clip(),
    );
    // Preserve element-local click coordinates after clipping its hit area.
    if let Some(handler) = target.on_click_at.take() {
        let dx = (i32::from(target.rect.x) - x).max(0) as u16;
        let dy = (i32::from(target.rect.y) - y).max(0) as u16;
        target.on_click_at = Some(Rc::new(move |cx, cy| {
            handler(cx.saturating_add(dx), cy.saturating_add(dy));
        }));
    }
    target.layer = target.layer.or(ctx.layer);
    ctx.hit_map.register(target);
}

pub fn render_node(node: &VNode, rect: Rect, buffer: &mut Buffer) {
    let mut hit_map = HitMap::new();
    render_node_interactive(node, rect, buffer, &mut hit_map);
}

pub fn render_node_interactive(
    node: &VNode,
    rect: Rect,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
) {
    hit_map.collect_interactions(node);
    let (width, height) = measure_node(node, rect.width, rect.height);
    let mut overlays = Vec::new();
    {
        let mut ctx = RenderContext {
            buffer,
            hit_map,
            layer: None,
            scrolls: Vec::new(),
            overlays: &mut overlays,
            active_fg: None,
            active_bg: None,
        };
        render_node_offset_internal(
            node,
            None,
            rect.x as i32,
            rect.y as i32,
            width,
            height,
            &mut ctx,
        );
    }

    while !overlays.is_empty() {
        let current_overlays = std::mem::take(&mut overlays);
        for pending in current_overlays {
            render_single_overlay(pending, rect, buffer, hit_map, &mut overlays);
        }
    }
    hit_map.finish_interactions();
}

pub fn render_node_offset(
    node: &VNode,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
) {
    hit_map.collect_interactions(node);
    let (width, height) = measure_node(node, width, height);
    let mut overlays = Vec::new();
    {
        let mut ctx = RenderContext {
            buffer,
            hit_map,
            layer: None,
            scrolls: Vec::new(),
            overlays: &mut overlays,
            active_fg: None,
            active_bg: None,
        };
        render_node_offset_internal(node, None, x, y, width, height, &mut ctx);
    }

    let screen_rect = Rect::new(0, 0, buffer.width, buffer.height);
    while !overlays.is_empty() {
        let current_overlays = std::mem::take(&mut overlays);
        for pending in current_overlays {
            render_single_overlay(pending, screen_rect, buffer, hit_map, &mut overlays);
        }
    }
    hit_map.finish_interactions();
}

fn render_single_overlay(
    pending: PendingOverlay,
    screen: Rect,
    buffer: &mut Buffer,
    hit_map: &mut HitMap,
    overlays: &mut Vec<PendingOverlay>,
) {
    let PendingOverlay {
        overlay,
        anchor_rect,
        foreground,
    } = pending;
    if !overlay.open {
        return;
    }

    let anchor_rect = overlay
        .anchor
        .and_then(|id| hit_map.layouts.get(&id))
        .map(|layout| {
            Rect::new(
                layout.x.max(0) as u16,
                layout.y.max(0) as u16,
                layout.width,
                layout.height,
            )
        })
        .or(anchor_rect);
    if overlay.backdrop {
        for cell in &mut buffer.cells {
            cell.fg = Some(Color::DarkGrey);
            cell.bold = false;
        }
    }

    hit_map.register(HitTarget {
        rect: screen,
        layer: Some(overlay.id),
        dismiss: Some(overlay.id),
        on_click: None,
        on_click_at: None,
        focus_id: None,
        on_key: None,
        on_scroll: None,
    });

    if let Some(content) = overlay.content {
        let (cw, ch) = measure_node(&content, screen.width, screen.height);
        let popover_rect = compute_popover_rect(overlay.placement, (cw, ch), anchor_rect, screen);

        hit_map.register(HitTarget {
            rect: popover_rect,
            layer: Some(overlay.id),
            dismiss: None,
            on_click: None,
            on_click_at: None,
            focus_id: None,
            on_key: None,
            on_scroll: None,
        });

        for cy in popover_rect.y..(popover_rect.y + popover_rect.height) {
            for cx in popover_rect.x..(popover_rect.x + popover_rect.width) {
                buffer.set_char(cx, cy, ' ', None, None, false);
            }
        }

        let mut ctx = RenderContext {
            buffer,
            hit_map,
            layer: Some(overlay.id),
            scrolls: Vec::new(),
            overlays,
            active_fg: foreground,
            active_bg: None,
        };
        render_node_offset_internal(
            &content,
            None,
            popover_rect.x as i32,
            popover_rect.y as i32,
            popover_rect.width,
            popover_rect.height,
            &mut ctx,
        );
    }
}

fn render_node_offset_internal(
    node: &VNode,
    layout: Option<&ComputedLayoutNode>,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    ctx: &mut RenderContext,
) {
    if let VNode::Pressable(p) = node
        && let Some(id) = p.focus_id
    {
        ctx.hit_map.layouts.insert(
            id,
            LayoutTarget {
                x,
                y,
                width,
                height,
                scrolls: ctx.scrolls.clone(),
            },
        );
    }
    if let VNode::View(v) = node
        && let Some(id) = v.scroll.as_ref().and_then(|s| s.id)
    {
        ctx.hit_map.layouts.insert(
            id,
            LayoutTarget {
                x,
                y,
                width,
                height,
                scrolls: ctx.scrolls.clone(),
            },
        );
    }
    if let VNode::Overlay(overlay) = node {
        let anchor_rect = if let Some(ref trigger) = overlay.trigger {
            let anchor = Rect::new(x.max(0) as u16, y.max(0) as u16, width, height);
            render_node_offset_internal(trigger, None, x, y, width, height, ctx);
            Some(anchor)
        } else {
            None
        };

        if overlay.open {
            ctx.overlays.push(PendingOverlay {
                overlay: overlay.clone(),
                anchor_rect,
                foreground: ctx.active_fg,
            });
        }
        return;
    }

    if width == 0 || height == 0 {
        // Out-of-flow overlays still render when their ancestor has no
        // intrinsic size (for example, a root containing only a dialog).
        let previous_fg = ctx.active_fg;
        if let VNode::View(view) = node
            && let Some(foreground) = view
                .visual
                .as_ref()
                .and_then(|visual| visual.foreground_color)
        {
            ctx.active_fg = Some(foreground);
        }
        let children = match node {
            VNode::View(v) => Some(&v.children),
            VNode::Pressable(p) => Some(&p.children),
            VNode::Fragment(children) => Some(children),
            _ => None,
        };
        if let Some(children) = children {
            for child in children {
                render_node_offset_internal(child, None, x, y, 0, 0, ctx);
            }
        }
        ctx.active_fg = previous_fg;
        return;
    }
    if (x >= ctx.buffer.width as i32
        || y >= ctx.buffer.height as i32
        || x + width as i32 <= 0
        || y + height as i32 <= 0)
        && !matches!(
            node,
            VNode::View(_) | VNode::Pressable(_) | VNode::Fragment(_)
        )
    {
        return;
    }

    match node {
        VNode::Empty | VNode::Overlay(_) => {}
        VNode::Text(text) => {
            let lines = crate::element::text::lines(text, Some(width));
            for (i, line) in lines.iter().enumerate() {
                let curr_y = y + i as i32;
                if curr_y < 0 || curr_y >= (y + height as i32) || curr_y >= ctx.buffer.height as i32
                {
                    continue;
                }
                let clip = ctx.buffer.active_clip();
                if curr_y < clip.y as i32 || curr_y >= (clip.y as i32 + clip.height as i32) {
                    continue;
                }
                for (char_idx, ch) in line
                    .chars()
                    .chain(text.fill.into_iter().cycle())
                    .take(usize::from(width))
                    .enumerate()
                {
                    let curr_x = x + char_idx as i32;
                    if curr_x >= x + width as i32 || curr_x >= ctx.buffer.width as i32 {
                        break;
                    }
                    if curr_x >= 0 {
                        let fg = text.fg.or(ctx.active_fg);
                        let bg = text.bg.or(ctx.active_bg).or_else(|| {
                            ctx.buffer
                                .get(curr_x as u16, curr_y as u16)
                                .and_then(|c| c.bg)
                        });
                        ctx.buffer
                            .set_char(curr_x as u16, curr_y as u16, ch, fg, bg, text.bold);
                    }
                }
            }
        }
        VNode::Fragment(children) => {
            let mut curr_y = y;
            for (idx, child) in children.iter().enumerate() {
                let child_layout = layout.and_then(|l| l.children.get(idx));
                let (cw, ch) = if let Some(cl) = child_layout {
                    (cl.rect.width, cl.rect.height)
                } else {
                    let (_, ch) = measure_node(child, width, height);
                    (width, ch.min(height))
                };
                render_node_offset_internal(child, child_layout, x, curr_y, cw, ch, ctx);
                curr_y = curr_y.saturating_add(ch as i32);
            }
        }
        VNode::Pressable(p) => {
            if p.focus_id.is_some() || p.on_click.is_some() || p.on_key.is_some() || p.tab_stop {
                let target = HitTarget {
                    rect: Rect::new(x.max(0) as u16, y.max(0) as u16, width, height),
                    layer: ctx.layer,
                    dismiss: None,
                    on_click: p.on_click.clone(),
                    on_click_at: p.on_click_at.clone(),
                    focus_id: p.focus_id,
                    on_key: p.on_key.clone(),
                    on_scroll: None,
                };
                register_target(ctx, x, y, target);
            }
            if p.children.len() == 1 {
                render_node_offset_internal(&p.children[0], layout, x, y, width, height, ctx);
            } else {
                for (idx, child) in p.children.iter().enumerate() {
                    let child_layout = layout.and_then(|l| l.children.get(idx));
                    render_node_offset_internal(child, child_layout, x, y, width, height, ctx);
                }
            }
        }
        VNode::View(v) => {
            let prev_fg = ctx.active_fg;
            let prev_bg = ctx.active_bg;
            if let Some(vis) = &v.visual {
                if let Some(fg) = vis.foreground_color {
                    ctx.active_fg = Some(fg);
                }
                if let Some(bg) = vis.background_color {
                    ctx.active_bg = Some(bg);
                }
            }

            let (bg, border, border_color, title) = if let Some(vis) = &v.visual {
                (
                    vis.background_color,
                    vis.border,
                    vis.border_color,
                    vis.title.as_deref(),
                )
            } else {
                (None, BorderStyle::None, None, None)
            };

            // 1. Fill background if specified
            if let Some(bg) = bg {
                for curr_y in y.max(0)..(y + height as i32).min(ctx.buffer.height as i32) {
                    for curr_x in x.max(0)..(x + width as i32).min(ctx.buffer.width as i32) {
                        if curr_x >= 0 && curr_y >= 0 {
                            ctx.buffer.set_char(
                                curr_x as u16,
                                curr_y as u16,
                                ' ',
                                None,
                                Some(bg),
                                false,
                            );
                        }
                    }
                }
            }

            // 2. Draw border
            let border_w: u16 = if border != BorderStyle::None { 1 } else { 0 };
            let border_h: u16 = if border != BorderStyle::None { 1 } else { 0 };

            if let Some(chars) = border.chars().filter(|_| width >= 2 && height >= 2) {
                let right = x + width as i32 - 1;
                let bottom = y + height as i32 - 1;

                let draw_ch = |cx: i32, cy: i32, ch: char, buf: &mut Buffer| {
                    if cx >= 0 && cy >= 0 && cx < i32::from(buf.width) && cy < i32::from(buf.height)
                    {
                        buf.set_char(cx as u16, cy as u16, ch, border_color, bg, false);
                    }
                };

                draw_ch(x, y, chars.top_left, ctx.buffer);
                draw_ch(right, y, chars.top_right, ctx.buffer);
                draw_ch(x, bottom, chars.bottom_left, ctx.buffer);
                draw_ch(right, bottom, chars.bottom_right, ctx.buffer);

                for cx in (x + 1).max(0)..right.min(i32::from(ctx.buffer.width)) {
                    draw_ch(cx, y, chars.horizontal, ctx.buffer);
                    draw_ch(cx, bottom, chars.horizontal, ctx.buffer);
                }

                for cy in (y + 1).max(0)..bottom.min(i32::from(ctx.buffer.height)) {
                    draw_ch(x, cy, chars.vertical, ctx.buffer);
                    draw_ch(right, cy, chars.vertical, ctx.buffer);
                }
            }

            // 3. Draw title
            if let Some(t) = title.filter(|t| width > 4 && !t.is_empty()) {
                let title_text = format!(" {} ", t);
                let max_len = (width.saturating_sub(4)) as usize;
                let displayed_title: String = title_text.chars().take(max_len).collect();
                let start_x = x + 2;
                for (idx, ch) in displayed_title.chars().enumerate() {
                    let cx = start_x + idx as i32;
                    if cx >= 0
                        && y >= 0
                        && cx < i32::from(ctx.buffer.width)
                        && y < i32::from(ctx.buffer.height)
                    {
                        ctx.buffer
                            .set_char(cx as u16, y as u16, ch, border_color, None, true);
                    }
                }
            }

            let inner_x = x + border_w as i32 + v.padding.left as i32;
            let inner_y = y + border_h as i32 + v.padding.top as i32;
            let extra_w = border_w * 2 + v.padding.left + v.padding.right;
            let extra_h = border_h * 2 + v.padding.top + v.padding.bottom;
            let inner_w = width.saturating_sub(extra_w);
            let inner_h = height.saturating_sub(extra_h);

            if let Some(vis) = &v.visual {
                if let Some((cx_off, cy_off)) = vis.cursor_offset {
                    let screen_cx = inner_x + cx_off as i32;
                    let screen_cy = inner_y + cy_off as i32;
                    let clip = ctx.buffer.active_clip();
                    let in_clip = screen_cx >= clip.x as i32
                        && screen_cx < (clip.x + clip.width) as i32
                        && screen_cy >= clip.y as i32
                        && screen_cy < (clip.y + clip.height) as i32;
                    if in_clip
                        && screen_cx >= 0
                        && screen_cy >= 0
                        && screen_cx < ctx.buffer.width as i32
                        && screen_cy < ctx.buffer.height as i32
                    {
                        ctx.buffer.set_cursor(screen_cx as u16, screen_cy as u16);
                    }
                }
            }

            if width > 0 && height > 0 && !v.children.is_empty() {
                let computed_storage;
                let computed = match layout {
                    Some(l) => l,
                    None => {
                        let mut engine = crate::element::taffy_bridge::LayoutEngine::new();
                        computed_storage = engine.compute_layout(node, width, height);
                        &computed_storage
                    }
                };

                let is_scroll_x = v.overflow_x == taffy::style::Overflow::Scroll;
                let is_scroll_y = v.overflow_y == taffy::style::Overflow::Scroll;
                let is_scrollable = is_scroll_x || is_scroll_y;
                // A fixed-size overlay may be allocated less than its requested
                // size. Keep its content clip and scroll range inside that allocation.
                let viewport_w = computed
                    .viewport_size
                    .0
                    .saturating_sub(computed.rect.width.saturating_sub(width));
                let viewport_h = computed
                    .viewport_size
                    .1
                    .saturating_sub(computed.rect.height.saturating_sub(height));

                let max_scroll_x = if is_scroll_x {
                    computed.content_size.0.saturating_sub(viewport_w)
                } else {
                    0
                };
                let max_scroll_y = if is_scroll_y {
                    computed.content_size.1.saturating_sub(viewport_h)
                } else {
                    0
                };

                let (scroll_x, scroll_y, scroll_fid) = if is_scrollable {
                    let fid = v
                        .scroll
                        .as_ref()
                        .and_then(|s| s.id)
                        .unwrap_or_else(next_render_focus_id);
                    let (mut sx, mut sy) = get_scroll(fid);
                    if let Some(s) = &v.scroll {
                        if s.scroll_x > 0 {
                            sx = s.scroll_x;
                        }
                        if s.scroll_y > 0 {
                            sy = s.scroll_y;
                        }
                    }
                    sx = sx.min(max_scroll_x);
                    sy = sy.min(max_scroll_y);
                    crate::element::scroll::set_scroll(fid, (sx, sy));
                    (sx, sy, Some(fid))
                } else if let Some(s) = &v.scroll {
                    (s.scroll_x, s.scroll_y, None)
                } else {
                    (0, 0, None)
                };

                if is_scrollable && let Some(id) = scroll_fid {
                    let on_scroll: Rc<dyn Fn(i16, i16)> = Rc::new(move |mut dx, dy| {
                        if is_scroll_x && !is_scroll_y && dx == 0 && dy != 0 {
                            dx = dy * 2;
                        }
                        let (cx, cy) = get_scroll(id);
                        let nx = (cx as i32 + dx as i32).max(0).min(max_scroll_x as i32) as u16;
                        let ny = (cy as i32 + dy as i32).max(0).min(max_scroll_y as i32) as u16;
                        crate::element::scroll::set_scroll(id, (nx, ny));
                    });

                    let target = HitTarget {
                        rect: Rect::new(x.max(0) as u16, y.max(0) as u16, width, height),
                        layer: ctx.layer,
                        dismiss: None,
                        on_click: None,
                        on_click_at: None,
                        focus_id: Some(id),
                        on_key: None,
                        on_scroll: Some(on_scroll),
                    };
                    register_target(ctx, x, y, target);

                    ctx.scrolls.push(ScrollRegion {
                        id,
                        x: inner_x,
                        y: inner_y,
                        width: viewport_w,
                        height: viewport_h,
                        max_x: (v.scroll.as_ref().map_or(0, |s| s.scroll_x) == 0)
                            .then_some(max_scroll_x),
                        max_y: (v.scroll.as_ref().map_or(0, |s| s.scroll_y) == 0)
                            .then_some(max_scroll_y),
                    });
                }

                let is_clip_x = is_scroll_x || v.overflow_x == taffy::style::Overflow::Hidden;
                let is_clip_y = is_scroll_y || v.overflow_y == taffy::style::Overflow::Hidden;
                let mut clipped = false;
                if is_clip_x || is_clip_y {
                    let clip_x = inner_x.max(0) as u16;
                    let clip_y = inner_y.max(0) as u16;
                    let clip_w = if is_clip_x {
                        viewport_w
                    } else {
                        ctx.buffer.width.saturating_sub(clip_x)
                    };
                    let clip_h = if is_clip_y {
                        viewport_h
                    } else {
                        ctx.buffer.height.saturating_sub(clip_y)
                    };
                    let clip_rect = Rect::new(clip_x, clip_y, clip_w, clip_h);
                    ctx.buffer.push_clip(clip_rect);
                    clipped = true;
                }

                for (idx, child) in v.children.iter().enumerate() {
                    if let Some(child_layout) = computed.children.get(idx) {
                        render_node_offset_internal(
                            child,
                            Some(child_layout),
                            x + child_layout.rect.x as i32 - scroll_x as i32,
                            y + child_layout.rect.y as i32 - scroll_y as i32,
                            child_layout.rect.width,
                            child_layout.rect.height,
                            ctx,
                        );
                    }
                }

                if clipped {
                    ctx.buffer.pop_clip();
                }

                if is_scrollable {
                    ctx.scrolls.pop();
                }

                let default_sb = crate::layout::ScrollbarStyle::default();
                let sb_style = v
                    .scroll
                    .as_ref()
                    .map(|s| &s.scrollbar_style)
                    .unwrap_or(&default_sb);
                let v_track_ch = sb_style.chars.vertical_track;
                let v_thumb_ch = sb_style.chars.vertical_thumb;
                let v_track_fg = sb_style.track_color.or(border_color);
                let v_thumb_fg = sb_style.thumb_color.or(border_color);

                let sb_vis = v
                    .scroll
                    .as_ref()
                    .map_or(crate::layout::ScrollbarVisibility::Auto, |s| {
                        s.scrollbar_visibility
                    });
                let show_v_sb = is_scroll_y
                    && match sb_vis {
                        crate::layout::ScrollbarVisibility::Always => true,
                        crate::layout::ScrollbarVisibility::Never => false,
                        crate::layout::ScrollbarVisibility::Auto => max_scroll_y > 0,
                    };
                let show_h_sb = is_scroll_x
                    && match sb_vis {
                        crate::layout::ScrollbarVisibility::Always => true,
                        crate::layout::ScrollbarVisibility::Never => false,
                        crate::layout::ScrollbarVisibility::Auto => max_scroll_x > 0,
                    };

                if show_v_sb && inner_w > 0 && inner_h > 0 {
                    let track_h = inner_h.saturating_sub(u16::from(show_h_sb));
                    let thumb =
                        scrollbar_thumb(track_h, viewport_h, computed.content_size.1, scroll_y);
                    for row in 0..track_h {
                        let sb_x = inner_x + i32::from(inner_w) - 1;
                        let sb_y = inner_y + i32::from(row);
                        if sb_x >= 0 && sb_y >= 0 {
                            let active = thumb.contains(&row);
                            ctx.buffer.set_char(
                                sb_x as u16,
                                sb_y as u16,
                                if active { v_thumb_ch } else { v_track_ch },
                                if active { v_thumb_fg } else { v_track_fg },
                                None,
                                active,
                            );
                        }
                    }
                }

                if show_h_sb && inner_w > 0 && inner_h > 0 {
                    let h_track_ch = sb_style.chars.horizontal_track;
                    let h_thumb_ch = sb_style.chars.horizontal_thumb;
                    let h_track_fg = sb_style.track_color.or(border_color);
                    let h_thumb_fg = sb_style.thumb_color.or(border_color);

                    let track_w = inner_w.saturating_sub(u16::from(show_v_sb));
                    let thumb =
                        scrollbar_thumb(track_w, viewport_w, computed.content_size.0, scroll_x);
                    for col in 0..track_w {
                        let sb_x = inner_x + i32::from(col);
                        let sb_y = inner_y + i32::from(inner_h) - 1;
                        if sb_x >= 0 && sb_y >= 0 {
                            let active = thumb.contains(&col);
                            ctx.buffer.set_char(
                                sb_x as u16,
                                sb_y as u16,
                                if active { h_thumb_ch } else { h_track_ch },
                                if active { h_thumb_fg } else { h_track_fg },
                                None,
                                active,
                            );
                        }
                    }
                }
                if show_v_sb && show_h_sb && inner_w > 0 && inner_h > 0 {
                    let sb_x = inner_x + i32::from(inner_w) - 1;
                    let sb_y = inner_y + i32::from(inner_h) - 1;
                    if sb_x >= 0 && sb_y >= 0 {
                        ctx.buffer.set_char(
                            sb_x as u16,
                            sb_y as u16,
                            sb_style.chars.corner,
                            v_track_fg,
                            None,
                            false,
                        );
                    }
                }
            }

            ctx.active_fg = prev_fg;
            ctx.active_bg = prev_bg;
        }
    }
}
