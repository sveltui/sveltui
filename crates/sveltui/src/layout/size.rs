#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Size {
    #[default]
    Auto,
    Fixed(u16),
    Percent(f32),
    Fill,
    Fr(u16),
}

impl Size {
    pub const fn auto() -> Self {
        Size::Auto
    }

    pub const fn fixed(v: u16) -> Self {
        Size::Fixed(v)
    }

    pub const fn percent(p: f32) -> Self {
        Size::Percent(p)
    }

    pub const fn fill() -> Self {
        Size::Fill
    }

    pub const fn fr(units: u16) -> Self {
        Size::Fr(units)
    }

    pub fn resolve(&self, parent: u16, content: u16) -> u16 {
        match self {
            Size::Fixed(w) => *w,
            Size::Percent(p) => ((parent as f32 * p / 100.0).round() as u16).min(parent),
            Size::Fill | Size::Fr(_) => parent,
            Size::Auto => content,
        }
    }

    pub fn to_taffy_dimension(&self) -> taffy::style::Dimension {
        match self {
            Size::Auto => taffy::style::Dimension::Auto,
            Size::Fixed(px) => taffy::style::Dimension::Length(*px as f32),
            Size::Percent(p) => taffy::style::Dimension::Percent(*p / 100.0),
            Size::Fill => taffy::style::Dimension::Percent(1.0),
            Size::Fr(_) => taffy::style::Dimension::Auto,
        }
    }

    pub fn to_taffy_min_max(&self) -> taffy::style::LengthPercentageAuto {
        match self {
            Size::Auto => taffy::style::LengthPercentageAuto::Auto,
            Size::Fixed(px) => taffy::style::LengthPercentageAuto::Length(*px as f32),
            Size::Percent(p) => taffy::style::LengthPercentageAuto::Percent(*p / 100.0),
            Size::Fill => taffy::style::LengthPercentageAuto::Percent(1.0),
            Size::Fr(_) => taffy::style::LengthPercentageAuto::Auto,
        }
    }
}

/// Apply dimension limits. Auto adds no bound; a conflicting minimum wins.
pub fn clamp_dimension(val: u16, min: Option<Size>, max: Option<Size>, parent: u16) -> u16 {
    let min = min.map_or(0, |size| size.resolve(parent, 0));
    let max = max.map_or(u16::MAX, |size| size.resolve(parent, u16::MAX));
    val.clamp(min, max.max(min))
}

impl From<u16> for Size {
    fn from(v: u16) -> Self {
        Size::Fixed(v)
    }
}

impl From<i32> for Size {
    fn from(v: i32) -> Self {
        Size::Fixed(v.max(0) as u16)
    }
}
