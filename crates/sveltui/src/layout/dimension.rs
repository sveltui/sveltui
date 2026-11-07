/// A length along one axis: a number of cells, a fraction of the parent
/// (`Percent(1.0)` is all of it), or `Auto` to size from content. It's our
/// own type, so code using it doesn't change when taffy's does.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dimension {
    Length(f32),
    Percent(f32),
    #[default]
    Auto,
}

impl Dimension {
    pub(crate) fn to_taffy(self) -> taffy::style::Dimension {
        match self {
            Self::Length(cells) => taffy::style::Dimension::length(cells),
            Self::Percent(fraction) => taffy::style::Dimension::percent(fraction),
            Self::Auto => taffy::style::Dimension::auto(),
        }
    }
}

/// Accept a `Dimension` or a numeric terminal-cell length.
pub trait IntoDimension {
    fn into_dimension(self) -> Dimension;
}

impl IntoDimension for Dimension {
    fn into_dimension(self) -> Dimension {
        self
    }
}

impl IntoDimension for u16 {
    fn into_dimension(self) -> Dimension {
        Dimension::Length(f32::from(self))
    }
}

impl IntoDimension for i32 {
    fn into_dimension(self) -> Dimension {
        Dimension::Length(self.max(0) as f32)
    }
}

impl IntoDimension for f32 {
    fn into_dimension(self) -> Dimension {
        Dimension::Length(self.max(0.0))
    }
}

impl IntoDimension for f64 {
    fn into_dimension(self) -> Dimension {
        Dimension::Length(self.max(0.0) as f32)
    }
}

impl From<Dimension> for taffy::style::Dimension {
    fn from(dimension: Dimension) -> Self {
        dimension.to_taffy()
    }
}
