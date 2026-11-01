pub mod alignment;
pub mod border;
pub mod dimension;
pub mod geometry;
pub mod margin;
pub mod padding;
pub mod placement;
pub mod scrollbar;

pub use alignment::FlexDirectionExt;
pub use border::{BorderChars, BorderStyle};
pub use dimension::IntoDimension;
pub use geometry::Rect;
pub use margin::Margin;
pub use padding::Padding;
pub use placement::{Placement, compute_popover_rect};
pub use scrollbar::{OverflowExt, ScrollbarChars, ScrollbarStyle, ScrollbarVisibility};

// Direct Taffy re-exports
pub use taffy::geometry::{Point, Rect as TaffyRect, Size as TaffySize};
pub use taffy::style::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap,
    JustifyContent, Overflow, Position,
};
