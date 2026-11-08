use crate::layout::{Positioner, geometry::Rect};

/// Target placement for a floating popover or overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Placement {
    /// Placed in the exact center of the screen or viewport.
    #[default]
    Center,
    /// Centered above the anchor.
    Top,
    /// Above the anchor, aligned to the start (left) edge.
    TopStart,
    /// Above the anchor, aligned to the end (right) edge.
    TopEnd,
    /// Centered below the anchor.
    Bottom,
    /// Below the anchor, aligned to the start (left) edge.
    BottomStart,
    /// Below the anchor, aligned to the end (right) edge.
    BottomEnd,
    /// Centered to the left of the anchor.
    Left,
    /// To the left of the anchor, aligned to the top edge.
    LeftStart,
    /// To the left of the anchor, aligned to the bottom edge.
    LeftEnd,
    /// Centered to the right of the anchor.
    Right,
    /// To the right of the anchor, aligned to the top edge.
    RightStart,
    /// To the right of the anchor, aligned to the bottom edge.
    RightEnd,
    /// Anchored to explicit screen coordinates (e.g. mouse click for context menus).
    Anchor { x: u16, y: u16 },
}

/// Computes the layout [`Rect`] for a popover given its preferred placement,
/// desired content dimensions, optional anchor target, and screen bounds.
///
/// Features collision auto-flipping (e.g. Bottom -> Top when hitting bottom edge)
/// and viewport boundary clamping. See [`Positioner`] for gaps, skids and points.
pub fn compute_popover_rect(
    placement: Placement,
    content_size: (u16, u16),
    anchor: Option<Rect>,
    screen: Rect,
) -> Rect {
    Positioner::new(placement).position(anchor, content_size, screen)
}
