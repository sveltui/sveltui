use std::rc::Rc;
use sveltui::prelude::*;

use crate::listbox::{Listbox, ListboxOption};

#[derive(Props)]
pub struct RootProps {
    /// Id of the trigger.
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// The selected value. Pass it to control the selection yourself.
    #[prop(optional)]
    pub value: Option<String>,
    /// The value selected at first.
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(String)>>,
    /// Whether the popup is open. Pass it to control that yourself.
    #[prop(optional)]
    pub open: Option<bool>,
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
    #[prop(default = false)]
    pub disabled: bool,
    /// Whether the arrow keys wrap around from the last option to the first.
    #[prop(default = true)]
    pub loop_focus: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TriggerProps {
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ValueProps {
    /// What's shown when nothing's selected.
    #[prop(default = String::from("Select..."))]
    pub placeholder: String,
}

#[derive(Props)]
pub struct ContentProps {
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    /// Height including the border. Longer lists scroll.
    #[prop(default = 10)]
    pub max_height: u16,
    #[prop(default = Placement::BottomStart)]
    pub placement: Placement,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ItemProps {
    pub value: String,
    /// The text shown as the selected value and matched by typeahead. Leave it
    /// out and we use the item's text.
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct SelectProps {
    /// Id of the trigger.
    #[prop(optional)]
    pub id: Option<FocusId>,
    pub options: Vec<ListboxOption>,
    /// The selected value. Pass it to control the selection yourself.
    #[prop(optional)]
    pub value: Option<String>,
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(String)>>,
    #[prop(default = String::from("Select..."))]
    pub placeholder: String,
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[prop(default = false)]
    pub disabled: bool,
}

/// What the root shares with the select parts through context.
#[derive(Clone)]
pub(super) struct SelectContext {
    pub(super) listbox: Listbox,
    pub(super) selected: Option<String>,
    pub(super) open: bool,
    pub(super) disabled: bool,
    pub(super) set_open: Rc<dyn Fn(bool)>,
    pub(super) select: Rc<dyn Fn(String)>,
}
