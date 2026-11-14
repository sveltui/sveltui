use crate::element::focus::FocusId;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static SCROLL_MAP: RefCell<HashMap<FocusId, (u16, u16)>> = RefCell::new(HashMap::new());
    static VIEWPORT_MAP: RefCell<HashMap<FocusId, (u16, u16)>> = RefCell::new(HashMap::new());
}

pub fn get_scroll(id: FocusId) -> (u16, u16) {
    SCROLL_MAP.with(|m| *m.borrow().get(&id).unwrap_or(&(0, 0)))
}

pub fn set_scroll(id: FocusId, pos: (u16, u16)) {
    SCROLL_MAP.with(|m| {
        m.borrow_mut().insert(id, pos);
    });
}

pub fn get_viewport(id: FocusId) -> Option<(u16, u16)> {
    VIEWPORT_MAP.with(|m| m.borrow().get(&id).copied())
}

pub fn set_viewport(id: FocusId, size: (u16, u16)) {
    VIEWPORT_MAP.with(|m| {
        m.borrow_mut().insert(id, size);
    });
}

/// A scrollbar thumb being dragged: whose bar, which way it runs, and how
/// far into the thumb the pointer grabbed it, so the thumb doesn't jump to
/// put its top under the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThumbDrag {
    pub id: FocusId,
    pub vertical: bool,
    pub grab: u16,
}

thread_local! {
    static THUMB_DRAG: std::cell::Cell<Option<ThumbDrag>> = const { std::cell::Cell::new(None) };
}

pub(crate) fn thumb_drag() -> Option<ThumbDrag> {
    THUMB_DRAG.with(|d| d.get())
}

pub(crate) fn set_thumb_drag(drag: Option<ThumbDrag>) {
    THUMB_DRAG.with(|d| d.set(drag));
}
