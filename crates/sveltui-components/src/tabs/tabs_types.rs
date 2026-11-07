use std::{cell::RefCell, collections::HashMap, rc::Rc};
use sveltui::prelude::*;

/// When focusing a tab also selects it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsActivation {
    /// Moving focus to a tab (arrows, Home/End) selects it immediately.
    #[default]
    Automatic,
    /// Arrows only move focus; Enter/Space (or a click) selects.
    Manual,
}

#[derive(Props)]
pub struct RootProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// Controlled selection. Leave unset for uncontrolled tabs.
    #[prop(optional)]
    pub value: Option<String>,
    /// Initial selection for uncontrolled tabs. Without it the first enabled
    /// tab is selected.
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(String)>>,
    #[prop(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[prop(default = TabsActivation::Automatic)]
    pub activation_mode: TabsActivation,
    /// Whether arrow keys wrap from the last tab to the first.
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

/// Shared state that `Tabs::Root` provides to its parts.
#[derive(Clone)]
pub(super) struct TabsContext {
    /// The selection for this frame. Starts as the explicit value; if that
    /// is `None`, the first enabled trigger to render claims it.
    pub(super) resolved: Rc<RefCell<Option<String>>>,
    /// Stable (tab id, panel id) pair per value, kept across frames so a
    /// tab's `controls` and its panel's `labelled_by` always match.
    pub(super) ids: Rc<RefCell<HashMap<String, (FocusId, FocusId)>>>,
    pub(super) select: Rc<dyn Fn(String)>,
    pub(super) orientation: Orientation,
    pub(super) activation: TabsActivation,
    pub(super) loop_focus: bool,
}

impl TabsContext {
    /// The (tab id, panel id) pair for `value`, created on first use.
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
