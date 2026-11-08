//! Images that live beside the cell grid. A component hands the buffer an
//! [`ImagePaint`] and the rectangle it fills; the terminal draws it after
//! the cells, and only when something about it changed.

use crate::layout::Rect;
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

/// What an image draws itself with. The buffer never looks inside: it asks
/// for bytes to send and compares keys to tell whether anything changed.
pub trait ImagePaint {
    /// Identifies what would be drawn: the same key means the same picture
    /// in the same size, so nothing needs sending again.
    fn key(&self) -> u64;

    /// The bytes that draw the visible part of the image. The cursor is
    /// already on the visible part's top-left cell. `size` is the whole
    /// image in cells and `visible` the part to draw, relative to the
    /// image's own top-left.
    fn draw(&self, size: (u16, u16), visible: Rect, cell_px: (u16, u16)) -> Vec<u8>;

    /// The bytes that remove the image, for terminals that keep images
    /// apart from text (kitty). `None` means painting the cells over it
    /// is what removes it.
    fn erase(&self) -> Option<Vec<u8>> {
        None
    }
}

/// An image placed on the screen.
#[derive(Clone)]
pub struct ImagePlacement {
    /// The screen cell of the visible part's top-left corner.
    pub at: (u16, u16),
    /// The whole image in cells, including any part clipped away.
    pub size: (u16, u16),
    /// The part that shows, relative to the image's own top-left.
    pub visible: Rect,
    pub paint: Rc<dyn ImagePaint>,
}

impl ImagePlacement {
    /// The screen cells the visible part covers.
    pub fn screen_rect(&self) -> Rect {
        Rect::new(
            self.at.0,
            self.at.1,
            self.visible.width,
            self.visible.height,
        )
    }

    /// Whether `other` draws the same picture in the same place.
    pub fn same_as(&self, other: &Self) -> bool {
        self.at == other.at
            && self.size == other.size
            && self.visible == other.visible
            && self.paint.key() == other.paint.key()
    }
}

impl PartialEq for ImagePlacement {
    fn eq(&self, other: &Self) -> bool {
        self.same_as(other)
    }
}

impl Eq for ImagePlacement {}

impl fmt::Debug for ImagePlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImagePlacement")
            .field("at", &self.at)
            .field("size", &self.size)
            .field("visible", &self.visible)
            .field("key", &self.paint.key())
            .finish()
    }
}

/// What a flush has to do about images, given the last frame's and this
/// frame's.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ImagePlan {
    /// Bytes that remove images that left (or changed).
    pub erase: Vec<Vec<u8>>,
    /// Cells to paint again, because an image without an erase sequence sat
    /// on them.
    pub repaint: Vec<Rect>,
    /// Indices into the current placements to draw.
    pub draw: Vec<usize>,
}

pub(crate) fn plan_images(
    prev: &[ImagePlacement],
    curr: &[ImagePlacement],
    cells_changed: impl Fn(Rect) -> bool,
    force: bool,
) -> ImagePlan {
    let mut plan = ImagePlan::default();
    for old in prev {
        if curr.iter().any(|new| new.same_as(old)) {
            continue;
        }
        match old.paint.erase() {
            Some(bytes) => plan.erase.push(bytes),
            None => plan.repaint.push(old.screen_rect()),
        }
    }
    for (i, new) in curr.iter().enumerate() {
        let unchanged = !force && prev.iter().any(|old| old.same_as(new));
        // Cells printed over an image may have wiped it, so it goes back on
        // top whenever the cells under it change.
        if !unchanged || cells_changed(new.screen_rect()) {
            plan.draw.push(i);
        }
    }
    plan
}

thread_local! {
    // Per thread, like the rest of what a frame reads: the thread that
    // draws is the one that asked the terminal.
    static CELL_PIXELS: Cell<(u16, u16)> = const { Cell::new((0, 0)) };
}

/// The size of one terminal cell in pixels, when the terminal reports it.
pub fn cell_pixels() -> Option<(u16, u16)> {
    let (w, h) = CELL_PIXELS.with(Cell::get);
    (w > 0 && h > 0).then_some((w, h))
}

/// Records the size of one cell in pixels; `(0, 0)` means unknown.
pub fn set_cell_pixels(width: u16, height: u16) {
    CELL_PIXELS.with(|cell| cell.set((width, height)));
}
