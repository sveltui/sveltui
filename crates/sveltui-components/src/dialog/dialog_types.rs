use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Props)]
pub struct DialogProps {
    /// Whether the dialog is open. Pass it to control that yourself: closing
    /// it (Escape, for instance) then only calls `on_open_change` and
    /// `on_close`.
    /// Whether the dialog is open, shared with the parent and set to `false`
    /// when it's dismissed. Pass it as `bind:open={state}`.
    #[prop(bindable)]
    pub open: Bindable<bool>,
    /// Whether it starts open when nothing else decides.
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub opener: Option<FocusId>,
    #[prop(default = InitialFocus::First)]
    pub initial_focus: InitialFocus,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(default = BorderStyle::Single)]
    pub border: BorderStyle,
    #[prop(default = Color::White)]
    pub border_color: Color,
    #[prop(default = Dimension::Length(50.0))]
    pub width: Dimension,
    #[prop(default = Dimension::Auto)]
    pub height: Dimension,
    #[prop(default = Padding::all(1))]
    pub padding: Padding,
    #[prop(optional)]
    pub on_close: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_open: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_key: Option<KeyHandler>,
    #[prop(default = Placement::Center)]
    pub placement: Placement,
    #[prop(default = true)]
    pub backdrop: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
