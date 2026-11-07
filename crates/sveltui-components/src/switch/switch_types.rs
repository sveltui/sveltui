use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Props)]
pub struct SwitchProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// Whether it's checked. Pass it to control the switch yourself: clicks
    /// then only call `on_checked_change`.
    /// Checked state shared with the parent, updated on every toggle. Pass it
    /// as `bind:checked={state}`.
    #[prop(bindable)]
    pub checked: Bindable<bool>,
    /// Whether it starts checked when nothing else decides.
    #[prop(default = false)]
    pub default_checked: bool,
    #[prop(optional)]
    pub on_checked_change: Option<Rc<dyn Fn(bool)>>,
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
