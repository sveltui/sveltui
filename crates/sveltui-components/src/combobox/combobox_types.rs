use std::rc::Rc;
use sveltui::prelude::*;

use crate::listbox::{Listbox, ListboxOption};

#[derive(Props)]
pub struct RootProps {
    /// Id of the input.
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
    /// Whether typing hides options whose label doesn't contain the text.
    /// Matching ignores case.
    #[prop(default = true)]
    pub filter: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct InputProps {
    /// What's shown while the input is empty.
    #[prop(default = String::from("Search..."))]
    pub placeholder: String,
    #[prop(default = Dimension::Length(24.0))]
    pub width: Dimension,
}

#[derive(Props)]
pub struct TriggerProps {}

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
    /// The text put in the input when the item is chosen, and matched by
    /// the filter. Leave it out and we use the item's text.
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct EmptyProps {
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ComboboxProps {
    /// Id of the input.
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
    #[prop(default = String::from("Search..."))]
    pub placeholder: String,
    #[prop(default = Dimension::Length(24.0))]
    pub width: Dimension,
    /// What's shown when no option matches the typed text.
    #[prop(default = String::from("No results"))]
    pub empty: String,
    #[prop(default = false)]
    pub disabled: bool,
}

/// What the root shares with the combobox parts through context.
#[derive(Clone)]
pub(super) struct ComboboxContext {
    pub(super) listbox: Listbox,
    pub(super) selected: Option<String>,
    pub(super) open: bool,
    pub(super) disabled: bool,
    /// The input's text.
    pub(super) text: State<String>,
    /// The text options are filtered by. It's empty while the input shows
    /// the selected option's label, so every option is listed.
    pub(super) query: String,
    pub(super) filter: bool,
    pub(super) set_open: Rc<dyn Fn(bool)>,
    /// Picks a value, puts its label in the input and closes the list.
    pub(super) choose: Rc<dyn Fn(String)>,
    /// Closes the list and puts back the text from when it opened.
    pub(super) revert: Rc<dyn Fn()>,
}
