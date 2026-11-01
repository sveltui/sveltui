/// Alignment of individual children on a flex container's cross axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

/// Distribution of the child group on a flex container's main axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

impl From<Align> for taffy::style::AlignItems {
    fn from(a: Align) -> Self {
        match a {
            Align::Start => taffy::style::AlignItems::FlexStart,
            Align::Center => taffy::style::AlignItems::Center,
            Align::End => taffy::style::AlignItems::FlexEnd,
            Align::Stretch => taffy::style::AlignItems::Stretch,
        }
    }
}

impl From<Justify> for taffy::style::JustifyContent {
    fn from(j: Justify) -> Self {
        match j {
            Justify::Start => taffy::style::JustifyContent::FlexStart,
            Justify::Center => taffy::style::JustifyContent::Center,
            Justify::End => taffy::style::JustifyContent::FlexEnd,
            Justify::SpaceBetween => taffy::style::JustifyContent::SpaceBetween,
        }
    }
}

pub trait FlexDirectionExt {
    /// Convert physical width/height to main/cross dimensions, or vice versa.
    fn axes<T>(self, width: T, height: T) -> (T, T);
}

impl FlexDirectionExt for taffy::style::FlexDirection {
    fn axes<T>(self, width: T, height: T) -> (T, T) {
        match self {
            Self::Column | Self::ColumnReverse => (height, width),
            Self::Row | Self::RowReverse => (width, height),
        }
    }
}
