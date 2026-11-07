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

/// Where children sit across the main axis (`align_items`), or where one
/// child sits (`align_self`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

/// Where one child sits across the main axis, overriding its parent's
/// `align_items`.
pub type AlignSelf = AlignItems;

impl AlignItems {
    pub(crate) fn to_taffy(self) -> taffy::style::AlignItems {
        use taffy::style::AlignItems as Taffy;
        match self {
            Self::Start => Taffy::START,
            Self::End => Taffy::END,
            Self::FlexStart => Taffy::FLEX_START,
            Self::FlexEnd => Taffy::FLEX_END,
            Self::Center => Taffy::CENTER,
            Self::Baseline => Taffy::BASELINE,
            Self::Stretch => Taffy::STRETCH,
        }
    }
}

/// How children share the space along the main axis (`justify_content`),
/// or how wrapped lines share it across (`align_content`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

/// How children share the space along the main axis.
pub type JustifyContent = AlignContent;

impl AlignContent {
    pub(crate) fn to_taffy(self) -> taffy::style::AlignContent {
        use taffy::style::AlignContent as Taffy;
        match self {
            Self::Start => Taffy::START,
            Self::End => Taffy::END,
            Self::FlexStart => Taffy::FLEX_START,
            Self::FlexEnd => Taffy::FLEX_END,
            Self::Center => Taffy::CENTER,
            Self::Stretch => Taffy::STRETCH,
            Self::SpaceBetween => Taffy::SPACE_BETWEEN,
            Self::SpaceEvenly => Taffy::SPACE_EVENLY,
            Self::SpaceAround => Taffy::SPACE_AROUND,
        }
    }
}
