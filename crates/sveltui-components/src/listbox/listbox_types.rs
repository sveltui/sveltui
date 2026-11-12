use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};
use sveltui::prelude::*;

/// An option you can pick, with a value and a label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListboxOption {
    /// What identifies the option.
    pub value: String,
    /// What's shown for the option, and what typing and filtering match
    /// against.
    pub label: String,
    pub disabled: bool,
}

impl ListboxOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl From<&str> for ListboxOption {
    fn from(value: &str) -> Self {
        Self::new(value, value)
    }
}

impl From<String> for ListboxOption {
    fn from(value: String) -> Self {
        Self::new(value.clone(), value)
    }
}

/// An option as it was registered during a render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) option: ListboxOption,
    /// Whether the option passes the current filter, and so gets shown.
    pub(crate) visible: bool,
}

/// Where to move the highlight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Move {
    Next,
    Prev,
    First,
    Last,
}

/// Tracks the options and which one is highlighted. The highlight is virtual
/// focus (`aria-activedescendant`): real focus stays where it is while the
/// arrow keys move the highlight. Items register themselves on every render, so
/// there's no separate options list to keep in sync.
#[derive(Clone)]
pub(crate) struct Listbox {
    /// The options registered so far in this render.
    current: Rc<RefCell<Vec<Entry>>>,
    /// The options from the last render. Parts that render before the items
    /// read these, and so do event handlers.
    previous: Rc<RefCell<Vec<Entry>>>,
    /// A stable id per option value, so `active_descendant` keeps pointing at
    /// the right option.
    ids: Rc<RefCell<HashMap<String, FocusId>>>,
    highlighted: State<Option<String>>,
    typeahead: Rc<RefCell<(String, Option<Instant>)>>,
    /// Bumped whenever the options change. That schedules one more render, so
    /// anything drawn before the items catches up.
    revision: State<u64>,
    pub(crate) trigger_id: FocusId,
    pub(crate) list_id: FocusId,
    pub(crate) scroll_id: FocusId,
    pub(crate) wrap: bool,
}

impl Listbox {
    /// Sets up the listbox's hooks. Call it exactly once on every render of the
    /// owning component, since hooks depend on being called in the same order
    /// each time.
    pub(crate) fn use_listbox(trigger_id: Option<FocusId>, wrap: bool) -> Self {
        let current = use_hook(|| Rc::new(RefCell::new(Vec::new())));
        let previous = use_hook(|| Rc::new(RefCell::new(Vec::new())));
        let ids = use_hook(|| Rc::new(RefCell::new(HashMap::new())));
        let highlighted = state(None::<String>);
        let typeahead = use_hook(|| Rc::new(RefCell::new((String::new(), None))));
        let revision = state(0u64);
        let default_trigger = use_hook(FocusId::new);
        let list_id = use_hook(FocusId::new);
        let scroll_id = use_hook(FocusId::new);
        Self {
            current,
            previous,
            ids,
            highlighted,
            typeahead,
            revision,
            trigger_id: trigger_id.unwrap_or(default_trigger),
            list_id,
            scroll_id,
            wrap,
        }
    }

    /// Starts a render: the last render's options become the previous ones.
    pub(crate) fn begin_frame(&self) {
        let registered = std::mem::take(&mut *self.current.borrow_mut());
        *self.previous.borrow_mut() = registered;
    }

    /// Ends a render, and schedules another one if the options changed.
    pub(crate) fn end_frame(&self) {
        if *self.current.borrow() != *self.previous.borrow() {
            self.revision.update(|r| *r = r.wrapping_add(1));
        }
    }

    /// Registers an option for this render and hands back its stable id.
    pub(crate) fn register(&self, option: ListboxOption, visible: bool) -> FocusId {
        let id = self.option_id(&option.value);
        self.current.borrow_mut().push(Entry { option, visible });
        id
    }

    pub(crate) fn option_id(&self, value: &str) -> FocusId {
        let mut ids = self.ids.borrow_mut();
        if let Some(id) = ids.get(value) {
            return *id;
        }
        // `FocusId::new()`, not `default()`: the default is a per-render id
        // that changes between frames.
        let id = FocusId::new();
        ids.insert(value.to_owned(), id);
        id
    }

    /// This render's options, or the last render's if nothing has registered
    /// yet.
    pub(crate) fn entries(&self) -> Vec<Entry> {
        let current = self.current.borrow();
        if current.is_empty() {
            self.previous.borrow().clone()
        } else {
            current.clone()
        }
    }

    pub(crate) fn label_of(&self, value: &str) -> Option<String> {
        self.entries()
            .into_iter()
            .find(|e| e.option.value == value)
            .map(|e| e.option.label)
    }

    /// How many visible options have registered so far in this render.
    pub(crate) fn visible_count(&self) -> usize {
        self.current.borrow().iter().filter(|e| e.visible).count()
    }

    pub(crate) fn highlighted(&self) -> Option<String> {
        self.highlighted.get()
    }

    /// The highlighted option's id, as long as it's visible.
    pub(crate) fn active_descendant(&self) -> Option<FocusId> {
        let value = self.highlighted()?;
        self.entries()
            .iter()
            .any(|e| e.visible && e.option.value == value)
            .then(|| self.option_id(&value))
    }

    pub(crate) fn highlight(&self, value: Option<String>) {
        if self.highlighted.get() != value {
            self.highlighted.set(value);
        }
    }

    /// The highlighted option, as long as it's visible and enabled.
    pub(crate) fn highlighted_choice(&self) -> Option<String> {
        let value = self.highlighted()?;
        self.selectable().into_iter().find(|v| *v == value)
    }

    /// The values of the visible, enabled options, in order.
    fn selectable(&self) -> Vec<String> {
        self.entries()
            .into_iter()
            .filter(|e| e.visible && !e.option.disabled)
            .map(|e| e.option.value)
            .collect()
    }

    /// Moves the highlight and returns the newly highlighted value.
    pub(crate) fn step(&self, direction: Move) -> Option<String> {
        let values = self.selectable();
        if values.is_empty() {
            return None;
        }
        let last = values.len() - 1;
        let current = self
            .highlighted()
            .and_then(|h| values.iter().position(|v| *v == h));
        let index = match (direction, current) {
            (Move::First, _) | (Move::Next, None) => 0,
            (Move::Last, _) | (Move::Prev, None) => last,
            (Move::Next, Some(i)) if i == last => {
                if self.wrap {
                    0
                } else {
                    last
                }
            }
            (Move::Next, Some(i)) => i + 1,
            (Move::Prev, Some(0)) => {
                if self.wrap {
                    last
                } else {
                    0
                }
            }
            (Move::Prev, Some(i)) => i - 1,
        };
        let next = values[index].clone();
        self.highlight(Some(next.clone()));
        Some(next)
    }

    /// Highlights `preferred` if it can be picked; otherwise falls back to the
    /// first or last option, depending on `fallback`.
    pub(crate) fn highlight_or(&self, preferred: Option<&str>, fallback: Move) -> Option<String> {
        if let Some(value) = preferred
            && self.selectable().iter().any(|v| v == value)
        {
            self.highlight(Some(value.to_owned()));
            return Some(value.to_owned());
        }
        self.highlight(None);
        self.step(fallback)
    }

    /// Highlights the next option whose label starts with what's been typed.
    /// The typed text resets after a short pause, and pressing the same letter
    /// again cycles through its matches.
    pub(crate) fn typeahead(&self, ch: char) -> Option<String> {
        let prefix = {
            let mut state = self.typeahead.borrow_mut();
            let now = Instant::now();
            let stale = state
                .1
                .is_none_or(|last| now.duration_since(last) > Duration::from_millis(700));
            if stale {
                state.0.clear();
            }
            state.1 = Some(now);
            let repeat = !state.0.is_empty() && state.0.chars().all(|c| c == ch);
            if !repeat {
                state.0.push(ch);
            }
            state.0.to_lowercase()
        };
        let options: Vec<_> = self
            .entries()
            .into_iter()
            .filter(|e| e.visible && !e.option.disabled)
            .collect();
        if options.is_empty() {
            return None;
        }
        let current = self
            .highlighted()
            .and_then(|h| options.iter().position(|e| e.option.value == h));
        let start = match current {
            Some(i) if prefix.chars().count() == 1 => i + 1,
            Some(i) => i,
            None => 0,
        };
        let found = (0..options.len())
            .map(|offset| &options[(start + offset) % options.len()])
            .find(|e| e.option.label.to_lowercase().starts_with(&prefix))?;
        let value = found.option.value.clone();
        self.highlight(Some(value.clone()));
        Some(value)
    }
}

/// All the plain text inside a node.
pub(crate) fn text_of(node: &VNode) -> String {
    match node {
        VNode::Text(t) => t.content.clone(),
        VNode::View(v) => v.children.iter().map(text_of).collect(),
        VNode::Fragment(items) => items.iter().map(text_of).collect(),
        _ => String::new(),
    }
}

/// A non-modal layer that leaves focus where it is when it opens. Tab won't
/// close it on its own; that's up to whoever owns it.
pub(crate) fn listbox_layer() -> DismissLayer {
    DismissLayer {
        scope: FocusScope {
            contain_tab: false,
            initial_focus: InitialFocus::Preserve,
        },
        tab: TabDismiss::None,
        dismiss_on_left: false,
        chain: false,
        dismiss_on_outside_press: true,
    }
}
