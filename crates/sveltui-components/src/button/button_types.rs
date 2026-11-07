use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Props)]
pub struct ButtonProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub on_click: Option<Rc<dyn Fn()>>,
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[prop(default = Dimension::Auto)]
    pub height: Dimension,
    #[prop(optional)]
    pub border: Option<BorderStyle>,
    #[prop(optional)]
    pub border_color: Option<Color>,
    #[prop(default)]
    pub padding: Padding,
    #[prop(optional)]
    pub label: Option<String>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
