use taffy::style::Dimension;

/// Accept native Taffy dimensions or a numeric terminal-cell length.
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
