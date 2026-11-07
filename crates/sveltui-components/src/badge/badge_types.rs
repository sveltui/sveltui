use crossterm::style::Color;
use sveltui::prelude::*;

#[derive(Props)]
pub struct BadgeProps {
    #[prop(default = Color::Grey)]
    pub bg: Color,
    #[prop(default = Color::Black)]
    pub fg: Color,
    #[prop(optional)]
    pub label: Option<String>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
