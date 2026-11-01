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
