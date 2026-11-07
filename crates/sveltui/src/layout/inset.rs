use taffy::style::LengthPercentageAuto;

/// How far an absolutely positioned view sits from each side of its
/// parent. Sides we leave unset stay `auto`, so `Inset::new().top(0).right(1)`
/// pins a view to the top-right corner and lets its size come from its
/// content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Inset {
    pub top: Option<u16>,
    pub right: Option<u16>,
    pub bottom: Option<u16>,
    pub left: Option<u16>,
}

impl Inset {
    /// Every side unset.
    pub const fn new() -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: None,
        }
    }

    /// The same offset on every side, which stretches the view over its parent.
    pub const fn all(cells: u16) -> Self {
        Self {
            top: Some(cells),
            right: Some(cells),
            bottom: Some(cells),
            left: Some(cells),
        }
    }

    pub const fn top(mut self, cells: u16) -> Self {
        self.top = Some(cells);
        self
    }

    pub const fn right(mut self, cells: u16) -> Self {
        self.right = Some(cells);
        self
    }

    pub const fn bottom(mut self, cells: u16) -> Self {
        self.bottom = Some(cells);
        self
    }

    pub const fn left(mut self, cells: u16) -> Self {
        self.left = Some(cells);
        self
    }

    pub(crate) fn to_taffy(self) -> taffy::geometry::Rect<LengthPercentageAuto> {
        let side = |cells: Option<u16>| {
            cells.map_or(LengthPercentageAuto::Auto, |c| {
                LengthPercentageAuto::Length(f32::from(c))
            })
        };
        taffy::geometry::Rect {
            left: side(self.left),
            right: side(self.right),
            top: side(self.top),
            bottom: side(self.bottom),
        }
    }
}

impl From<u16> for Inset {
    fn from(cells: u16) -> Self {
        Inset::all(cells)
    }
}
