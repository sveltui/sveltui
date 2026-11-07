use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Props)]
pub struct DialogProps {
    #[prop(default = false)]
    pub open: bool,
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
