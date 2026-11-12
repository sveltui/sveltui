use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use sveltui::Runtime;
use sveltui::prelude::*;

use super::toast_types::{ToastId, ToastObject, ToastOptions};

/// Where a toast's countdown is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Timer {
    /// Added, but no provider has picked a timeout for it yet.
    Waiting,
    Running {
        deadline: Instant,
    },
    Paused {
        remaining: Duration,
    },
    /// Stays up until it's closed.
    Sticky,
}

struct Entry {
    toast: ToastObject,
    timer: Timer,
}

struct Inner {
    /// Oldest first.
    entries: RefCell<Vec<Entry>>,
    next: Cell<u64>,
    timeout: Cell<Duration>,
    limit: Cell<usize>,
    paused: Cell<bool>,
    /// Where focus was before the hotkey moved it to a toast, so closing
    /// that toast can put it back.
    return_focus: Cell<Option<FocusId>>,
    clock: RefCell<Rc<dyn Fn() -> Instant>>,
}

/// The list of toasts and what changes it: `add`, `update` and `close`. It's
/// a cheap handle, so clone it into any event handler. Inside a
/// `Toast::Provider`, `use_toast_manager()` returns the provider's.
#[derive(Clone)]
pub struct ToastManager(Rc<Inner>);

impl IntoPropOption<ToastManager> for ToastManager {
    fn into_prop_option(self) -> Option<ToastManager> {
        Some(self)
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    pub fn new() -> Self {
        Self(Rc::new(Inner {
            entries: RefCell::new(Vec::new()),
            next: Cell::new(1),
            timeout: Cell::new(Duration::from_secs(5)),
            limit: Cell::new(3),
            paused: Cell::new(false),
            return_focus: Cell::new(None),
            clock: RefCell::new(Rc::new(Instant::now)),
        }))
    }

    /// Shows a toast and hands back its id. Past the provider's limit, the
    /// oldest toast closes to make room.
    pub fn add(&self, options: ToastOptions) -> ToastId {
        let id = ToastId(self.0.next.get());
        self.0.next.set(id.0 + 1);
        self.0.entries.borrow_mut().push(Entry {
            toast: ToastObject {
                id,
                focus_id: FocusId::new(),
                options: Rc::new(options),
            },
            timer: Timer::Waiting,
        });
        let limit = self.0.limit.get().max(1);
        let overflow: Vec<ToastObject> = {
            let mut entries = self.0.entries.borrow_mut();
            let extra = entries.len().saturating_sub(limit);
            entries.drain(..extra).map(|e| e.toast).collect()
        };
        Self::closed(overflow);
        Runtime::set_dirty(true);
        id
    }

    /// Changes what a toast says. Its countdown starts over with the new
    /// options' timeout.
    pub fn update(&self, id: ToastId, options: ToastOptions) {
        let mut entries = self.0.entries.borrow_mut();
        if let Some(entry) = entries.iter_mut().find(|e| e.toast.id == id) {
            entry.toast.options = Rc::new(options);
            entry.timer = Timer::Waiting;
            Runtime::set_dirty(true);
        }
    }

    /// Removes a toast. Closing one that's already gone does nothing.
    pub fn close(&self, id: ToastId) {
        let removed: Vec<ToastObject> = {
            let mut entries = self.0.entries.borrow_mut();
            let (gone, kept) = std::mem::take(&mut *entries)
                .into_iter()
                .partition(|e| e.toast.id == id);
            *entries = kept;
            gone.into_iter().map(|e: Entry| e.toast).collect()
        };
        if !removed.is_empty() {
            Self::closed(removed);
            Runtime::set_dirty(true);
        }
    }

    /// The toasts up now, newest first.
    pub fn list(&self) -> Vec<ToastObject> {
        self.0
            .entries
            .borrow()
            .iter()
            .rev()
            .map(|e| e.toast.clone())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.entries.borrow().is_empty()
    }

    /// Runs each closed toast's `on_close`, after the list is no longer
    /// borrowed, so a callback can add or close toasts itself.
    fn closed(toasts: Vec<ToastObject>) {
        for toast in toasts {
            if let Some(on_close) = &toast.options.on_close {
                on_close();
            }
        }
    }

    fn now(&self) -> Instant {
        (self.0.clock.borrow())()
    }

    pub(crate) fn configure(&self, timeout: Duration, limit: usize) {
        self.0.timeout.set(timeout);
        self.0.limit.set(limit.max(1));
    }

    /// Stops or restarts every countdown, keeping the time each had left.
    /// The viewport pauses while the pointer or focus is inside it.
    pub(crate) fn set_paused(&self, paused: bool) {
        if self.0.paused.replace(paused) == paused {
            return;
        }
        let now = self.now();
        for entry in self.0.entries.borrow_mut().iter_mut() {
            entry.timer = match (entry.timer, paused) {
                (Timer::Running { deadline }, true) => Timer::Paused {
                    remaining: deadline.saturating_duration_since(now),
                },
                (Timer::Paused { remaining }, false) => Timer::Running {
                    deadline: now + remaining,
                },
                (timer, _) => timer,
            };
        }
    }

    /// Starts new countdowns, closes the toasts whose time is up, and asks
    /// for a frame when the next one will be. The provider calls this on
    /// every render.
    pub(crate) fn tick(&self) {
        let now = self.now();
        let default = self.0.timeout.get();
        let paused = self.0.paused.get();
        let expired: Vec<ToastObject> = {
            let mut entries = self.0.entries.borrow_mut();
            for entry in entries.iter_mut() {
                if entry.timer == Timer::Waiting {
                    let timeout = entry.toast.options.timeout.unwrap_or(default);
                    entry.timer = if timeout.is_zero() {
                        Timer::Sticky
                    } else if paused {
                        Timer::Paused { remaining: timeout }
                    } else {
                        Timer::Running {
                            deadline: now + timeout,
                        }
                    };
                }
            }
            let mut gone = Vec::new();
            entries.retain(|e| match e.timer {
                Timer::Running { deadline } if deadline <= now => {
                    gone.push(e.toast.clone());
                    false
                }
                _ => true,
            });
            if let Some(next) = entries
                .iter()
                .filter_map(|e| match e.timer {
                    Timer::Running { deadline } => Some(deadline),
                    _ => None,
                })
                .min()
            {
                Runtime::wake_at(next);
            }
            gone
        };
        Self::closed(expired);
    }

    pub(crate) fn set_return_focus(&self, id: Option<FocusId>) {
        self.0.return_focus.set(id);
    }

    /// Closes a toast from the keyboard, sending focus back to where it was
    /// before the hotkey, when it went there that way.
    pub(crate) fn close_from_key(&self, id: ToastId, ctx: &mut EventContext) {
        self.close(id);
        if let Some(back) = self.0.return_focus.take() {
            ctx.focus(back);
        }
    }

    #[cfg(test)]
    pub(crate) fn set_clock(&self, clock: impl Fn() -> Instant + 'static) {
        *self.0.clock.borrow_mut() = Rc::new(clock);
    }
}
