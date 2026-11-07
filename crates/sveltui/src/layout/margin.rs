use taffy::style::LengthPercentageAuto;

/// One side of a margin: a number of cells, or `Auto` to soak up the free
/// space on that side. Auto margins on both sides center a view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarginEdge {
    Cells(u16),
    Auto,
}

impl Default for MarginEdge {
    fn default() -> Self {
        Self::Cells(0)
    }
}

impl MarginEdge {
    pub(crate) fn to_taffy(self) -> LengthPercentageAuto {
        match self {
            Self::Cells(cells) => LengthPercentageAuto::length(f32::from(cells)),
            Self::Auto => LengthPercentageAuto::auto(),
        }
    }
}

/// Space outside a view's border, per side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Margin {
    pub top: MarginEdge,
    pub right: MarginEdge,
    pub bottom: MarginEdge,
    pub left: MarginEdge,
}

impl Margin {
    pub const fn zero() -> Self {
        Self::all(0)
    }

    pub const fn all(val: u16) -> Self {
        Self::only(val, val, val, val)
    }

    pub const fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self::only(vertical, horizontal, vertical, horizontal)
    }

    pub const fn only(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top: MarginEdge::Cells(top),
            right: MarginEdge::Cells(right),
            bottom: MarginEdge::Cells(bottom),
            left: MarginEdge::Cells(left),
        }
    }

    /// Auto on every side, which centers the view both ways in its parent.
    pub const fn auto() -> Self {
        Self {
            top: MarginEdge::Auto,
            right: MarginEdge::Auto,
            bottom: MarginEdge::Auto,
            left: MarginEdge::Auto,
        }
    }

    /// Auto on the left and right, which centers the view horizontally.
    pub const fn auto_x() -> Self {
        Self {
            left: MarginEdge::Auto,
            right: MarginEdge::Auto,
            ..Self::zero()
        }
    }

    /// Auto on the top and bottom, which centers the view vertically.
    pub const fn auto_y() -> Self {
        Self {
            top: MarginEdge::Auto,
            bottom: MarginEdge::Auto,
            ..Self::zero()
        }
    }

    pub(crate) fn to_taffy(self) -> taffy::geometry::Rect<LengthPercentageAuto> {
        taffy::geometry::Rect {
            left: self.left.to_taffy(),
            right: self.right.to_taffy(),
            top: self.top.to_taffy(),
            bottom: self.bottom.to_taffy(),
        }
    }
}

impl From<u16> for Margin {
    fn from(val: u16) -> Self {
        Margin::all(val)
    }
}

impl From<(u16, u16)> for Margin {
    fn from((horizontal, vertical): (u16, u16)) -> Self {
        Margin::symmetric(horizontal, vertical)
    }
}

impl From<(u16, u16, u16, u16)> for Margin {
    fn from((top, right, bottom, left): (u16, u16, u16, u16)) -> Self {
        Margin::only(top, right, bottom, left)
    }
}
