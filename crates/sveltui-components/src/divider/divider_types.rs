use crossterm::style::Color;
use sveltui::prelude::*;

#[derive(Props)]
pub struct DividerProps {
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(optional)]
    pub color: Option<Color>,
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[prop(default = 0.0)]
    pub flex_grow: f32,
    #[prop(default = 0.0)]
    pub flex_shrink: f32,
    #[prop(default = Dimension::Auto)]
    pub flex_basis: Dimension,
    #[prop(optional)]
    pub min_width: Option<Dimension>,
    #[prop(optional)]
    pub max_width: Option<Dimension>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
