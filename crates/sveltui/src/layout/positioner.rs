use crate::layout::{Inset, Placement, Rect};

/// What a floating element is positioned against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Anchor {
    /// No anchor is known; anchored placements fall back to centering.
    #[default]
    None,
    /// A trigger's on-screen rectangle.
    Rect(Rect),
    /// A single cell, such as the pointer position of a right click.
    Point(u16, u16),
}

impl From<Rect> for Anchor {
    fn from(rect: Rect) -> Self {
        Anchor::Rect(rect)
    }
}

impl From<Option<Rect>> for Anchor {
    fn from(rect: Option<Rect>) -> Self {
        rect.map_or(Anchor::None, Anchor::Rect)
    }
}

/// Pure geometry for floating content: where it goes, never how it is
/// focused or dismissed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Positioner {
    pub placement: Placement,
    gap: u16,
    skid: i16,
    flip: bool,
    shift: bool,
    match_anchor_width: bool,
    max_size: Option<(u16, u16)>,
}

impl Default for Positioner {
    fn default() -> Self {
        Self::new(Placement::default())
    }
}

impl Positioner {
    pub const fn new(placement: Placement) -> Self {
        Self {
            placement,
            gap: 0,
            skid: 0,
            flip: true,
            shift: true,
            match_anchor_width: false,
            max_size: None,
        }
    }

    /// Cells between the anchor and the content along the placement axis.
    pub const fn gap(mut self, cells: u16) -> Self {
        self.gap = cells;
        self
    }

    /// Cells to slide the content along the cross axis.
    pub const fn skid(mut self, cells: i16) -> Self {
        self.skid = cells;
        self
    }

    /// Use the opposite side when the preferred side lacks room.
    pub const fn flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    /// Slide along the cross axis to stay inside the bounds. When off, the
    /// content may extend past the right or bottom edge and gets clipped.
    pub const fn shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }

    pub const fn match_anchor_width(mut self, on: bool) -> Self {
        self.match_anchor_width = on;
        self
    }

    pub const fn max_size(mut self, width: u16, height: u16) -> Self {
        self.max_size = Some((width, height));
        self
    }

    /// The rectangle `content` occupies inside `bounds`.
    pub fn position(&self, anchor: impl Into<Anchor>, content: (u16, u16), bounds: Rect) -> Rect {
        if bounds.width == 0 || bounds.height == 0 {
            return Rect::new(0, 0, 0, 0);
        }
        let anchor = anchor.into();
        let (mut width, mut height) = content;
        if let (true, Anchor::Rect(a)) = (self.match_anchor_width, anchor) {
            width = a.width;
        }
        if let Some((max_w, max_h)) = self.max_size {
            width = width.min(max_w);
            height = height.min(max_h);
        }
        width = width.min(bounds.width);
        height = height.min(bounds.height);

        let max_x = bounds.x + bounds.width - width;
        let max_y = bounds.y + bounds.height - height;
        let center = || {
            Rect::new(
                (bounds.x + (bounds.width - width) / 2).clamp(bounds.x, max_x),
                (bounds.y + (bounds.height - height) / 2).clamp(bounds.y, max_y),
                width,
                height,
            )
        };

        if let Placement::Anchor { x, y } = self.placement {
            return Rect::new(
                x.clamp(bounds.x, max_x),
                y.clamp(bounds.y, max_y),
                width,
                height,
            );
        }
        let a = match anchor {
            Anchor::Rect(a) => a,
            Anchor::Point(x, y) => Rect::new(x, y, 0, 0),
            Anchor::None => return center(),
        };
        if self.placement == Placement::Center {
            return center();
        }

        let gap = i32::from(self.gap);
        let skid = i32::from(self.skid);
        let (w, h) = (i32::from(width), i32::from(height));
        let (ax, ay, aw, ah) = (
            i32::from(a.x),
            i32::from(a.y),
            i32::from(a.width),
            i32::from(a.height),
        );
        let (bx, by, bw, bh) = (
            i32::from(bounds.x),
            i32::from(bounds.y),
            i32::from(bounds.width),
            i32::from(bounds.height),
        );
        // `lead` is the preferred side; the content sits after (below/right
        // of) the anchor when `after` and before it otherwise.
        let pick = |after: bool, size: i32, a_start: i32, a_len: i32, b_start: i32, b_len: i32| {
            let room_after = (b_start + b_len - (a_start + a_len) - gap).max(0);
            let room_before = (a_start - b_start - gap).max(0);
            let (room, other) = if after {
                (room_after, room_before)
            } else {
                (room_before, room_after)
            };
            let use_after = if !self.flip || room >= size || room >= other {
                after
            } else {
                !after
            };
            if use_after {
                a_start + a_len + gap
            } else {
                a_start - size - gap
            }
        };
        // Aligns along the cross axis: start, end or centered on the anchor.
        let align = |kind: u8, size: i32, a_start: i32, a_len: i32| match kind {
            0 => a_start,
            1 => (a_start + a_len - size).max(0),
            _ => a_start + (a_len - size).max(0) / 2,
        };

        use Placement::{
            Bottom, BottomEnd, BottomStart, LeftEnd, LeftStart, Right, RightEnd, RightStart, Top,
            TopEnd, TopStart,
        };
        let (x, y) = match self.placement {
            Bottom | BottomStart | BottomEnd | Top | TopStart | TopEnd => {
                let after = matches!(self.placement, Bottom | BottomStart | BottomEnd);
                let kind = match self.placement {
                    BottomStart | TopStart => 0,
                    BottomEnd | TopEnd => 1,
                    _ => 2,
                };
                (
                    align(kind, w, ax, aw) + skid,
                    pick(after, h, ay, ah, by, bh),
                )
            }
            _ => {
                let after = matches!(self.placement, Right | RightStart | RightEnd);
                let kind = match self.placement {
                    RightStart | LeftStart => 0,
                    RightEnd | LeftEnd => 1,
                    _ => 2,
                };
                (
                    pick(after, w, ax, aw, bx, bw),
                    align(kind, h, ay, ah) + skid,
                )
            }
        };

        let cross_vertical = matches!(
            self.placement,
            Bottom | BottomStart | BottomEnd | Top | TopStart | TopEnd
        );
        let fit = |v: i32, lo: u16, hi: u16, shifted: bool| {
            let v = if shifted { v.min(i32::from(hi)) } else { v };
            v.max(i32::from(lo)) as u16
        };
        Rect::new(
            fit(x, bounds.x, max_x, self.shift || !cross_vertical),
            fit(y, bounds.y, max_y, self.shift || cross_vertical),
            width,
            height,
        )
    }

    /// The `position: Absolute` offsets that place `content` the same way
    /// inside `parent`, for in-flow popovers that don't need to escape
    /// their parent's clipping.
    pub fn to_inset(&self, anchor: impl Into<Anchor>, parent: Rect, content: (u16, u16)) -> Inset {
        let rect = self.position(anchor, content, parent);
        Inset::new().left(rect.x - parent.x).top(rect.y - parent.y)
    }
}
