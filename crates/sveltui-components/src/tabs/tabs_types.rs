use std::{cell::RefCell, collections::HashMap, rc::Rc};
use sveltui::prelude::*;

/// How a tab gets selected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsActivation {
    /// Focusing a tab selects it.
    #[default]
    Automatic,
    /// Arrows only move focus; press Enter or Space, or click, to select.
    Manual,
}

#[derive(Props)]
pub struct RootProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// The selected tab. Pass it to control the selection yourself.
    #[prop(optional)]
    pub value: Option<String>,
    /// The tab selected at first. Leave it out and the first enabled tab is
    /// used.
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(String)>>,
    #[prop(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[prop(default = TabsActivation::Automatic)]
    pub activation_mode: TabsActivation,
    /// Whether the arrow keys wrap around from the last tab to the first.
    #[prop(default = true)]
    pub loop_focus: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ListProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TriggerProps {
    pub value: String,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ContentProps {
    pub value: String,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

/// What the root shares with the tab parts through context: the selection,
/// stable ids, the select callback and how the list moves focus.
#[derive(Clone)]
pub(super) struct TabsContext {
    /// The selected tab for this render. It starts as the explicit value, and
    /// if there isn't one, the first enabled tab to render claims it.
    pub(super) resolved: Rc<RefCell<Option<String>>>,
    /// A stable (tab id, panel id) pair per value. We keep these across renders
    /// so a tab's `controls` and its panel's `labelled_by` always point at each
    /// other.
    pub(super) ids: Rc<RefCell<HashMap<String, (FocusId, FocusId)>>>,
    pub(super) select: Rc<dyn Fn(String)>,
    pub(super) orientation: Orientation,
    pub(super) activation: TabsActivation,
    pub(super) loop_focus: bool,
}

impl TabsContext {
    /// The (tab id, panel id) pair for `value`, made the first time it's asked
    /// for.
    pub(super) fn ids_for(&self, value: &str) -> (FocusId, FocusId) {
        *self
            .ids
            .borrow_mut()
            .entry(value.to_owned())
            .or_insert_with(|| (FocusId::new(), FocusId::new()))
    }

    pub(super) fn is_selected(&self, value: &str) -> bool {
        self.resolved.borrow().as_deref() == Some(value)
    }
}
