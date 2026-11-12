use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use sveltui::Runtime;
use sveltui::prelude::*;

use crate::clock::now;
use crate::controllable::{Controllable, use_controllable};

thread_local! {
    /// The exclusive popup that's open, if any. Per thread, like the rest of
    /// what a frame reads.
    static OPEN_EXCLUSIVE: RefCell<Option<HoverPopup>> = const { RefCell::new(None) };
}

/// The stable ids the parts point at each other with.
#[derive(Clone, Copy)]
pub(crate) struct Ids {
    pub(crate) trigger: FocusId,
    pub(crate) layer: FocusId,
    pub(crate) content: FocusId,
}

/// The settings a root passes in.
pub(crate) struct Settings {
    pub(crate) open: Bindable<bool>,
    pub(crate) default_open: bool,
    pub(crate) on_open_change: Option<Rc<dyn Fn(bool)>>,
    pub(crate) open_delay: Duration,
    pub(crate) close_delay: Duration,
    /// Whether opening closes any other exclusive popup, so only one shows
    /// at a time. Tooltips are; hover cards aren't.
    pub(crate) exclusive: bool,
}

/// Decides when a hover or focus popup is open. Pointer and focus say what
/// they want; the popup follows once the matching delay has passed.
#[derive(Clone)]
pub(crate) struct HoverPopup {
    pub(crate) open: Controllable<bool>,
    pub(crate) ids: Ids,
    trigger_hovered: Rc<Cell<bool>>,
    content_hovered: Rc<Cell<bool>>,
    /// Set by Escape, and cleared once nothing wants the popup, so a trigger
    /// that still has focus doesn't open it again straight away.
    suppressed: Rc<Cell<bool>>,
    /// What hover and focus wanted last render. We act when it changes,
    /// so `open` set from code holds until the pointer or focus moves.
    wanted_before: Rc<Cell<bool>>,
    /// Where we're heading (open or closed) and since when.
    pending: Rc<Cell<Option<(bool, Instant)>>>,
    open_delay: Duration,
    close_delay: Duration,
    exclusive: bool,
}

impl HoverPopup {
    /// Sets up the popup's hooks. Call it once on every render of the root.
    pub(crate) fn use_hover_popup(settings: Settings) -> Self {
        let open = use_controllable(
            settings.open.bound(),
            settings.open.value(),
            || settings.default_open,
            settings.on_open_change,
        );
        let ids = use_hook(|| Ids {
            trigger: FocusId::new(),
            layer: FocusId::new(),
            content: FocusId::new(),
        });
        Self {
            open,
            ids,
            trigger_hovered: use_hook(|| Rc::new(Cell::new(false))),
            content_hovered: use_hook(|| Rc::new(Cell::new(false))),
            suppressed: use_hook(|| Rc::new(Cell::new(false))),
            wanted_before: use_hook(|| Rc::new(Cell::new(false))),
            pending: use_hook(|| Rc::new(Cell::new(None))),
            open_delay: settings.open_delay,
            close_delay: settings.close_delay,
            exclusive: settings.exclusive,
        }
    }

    fn is(&self, other: &HoverPopup) -> bool {
        self.ids.layer == other.ids.layer
    }

    /// Takes the one exclusive slot, closing whatever popup had it. The one
    /// pushed out stays closed until the pointer and focus leave it, as
    /// after Escape, so two popups that both want to be open don't take
    /// turns.
    fn claim(&self) {
        let previous = OPEN_EXCLUSIVE.with(|slot| slot.borrow_mut().replace(self.clone()));
        if let Some(previous) = previous.filter(|p| !p.is(self)) {
            previous.dismiss();
        }
    }

    /// Gives the slot up, if this popup has it.
    fn release(&self) {
        OPEN_EXCLUSIVE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.as_ref().is_some_and(|p| p.is(self)) {
                *slot = None;
            }
        });
    }

    /// When hover or focus starts or stops wanting the popup, heads that
    /// way: opens or closes once the delay for that has passed, asking for
    /// a frame when it will have. The root calls this every render.
    pub(crate) fn update(&self, trigger_focused: bool) {
        // Also covers being opened from outside, through `open`.
        if self.exclusive && self.open.get() {
            self.claim();
        }
        let wanted = self.trigger_hovered.get() || self.content_hovered.get() || trigger_focused;
        if !wanted {
            self.suppressed.set(false);
        }
        let want = wanted && !self.suppressed.get();
        let now = now();
        if self.wanted_before.replace(want) != want {
            self.pending
                .set((want != self.open.get()).then_some((want, now)));
        }
        let Some((heading, since)) = self.pending.get() else {
            return;
        };
        if heading == self.open.get() {
            self.pending.set(None);
            return;
        }
        let due = since
            + if heading {
                self.open_delay
            } else {
                self.close_delay
            };
        if now >= due {
            self.pending.set(None);
            if self.exclusive {
                if heading {
                    self.claim();
                } else {
                    self.release();
                }
            }
            self.open.set(heading);
        } else {
            Runtime::wake_at(due);
        }
    }

    /// Closes now, and keeps it closed until the pointer and focus have
    /// both left. That's what Escape does.
    pub(crate) fn dismiss(&self) {
        self.suppressed.set(true);
        self.pending.set(None);
        self.release();
        self.open.set(false);
    }

    pub(crate) fn on_trigger_hover(&self) -> impl Fn(bool) + 'static {
        hover_setter(self.trigger_hovered.clone())
    }

    pub(crate) fn on_content_hover(&self) -> impl Fn(bool) + 'static {
        hover_setter(self.content_hovered.clone())
    }
}

fn hover_setter(cell: Rc<Cell<bool>>) -> impl Fn(bool) + 'static {
    move |inside| {
        cell.set(inside);
        Runtime::set_dirty(true);
    }
}
