use crossterm::style::Color;
use sveltui::prelude::*;

#[derive(Props)]
pub struct ProgressBarProps {
    #[prop(default = 0.0)]
    pub ratio: f32,
    #[prop(default = 20)]
    pub width: u16,
    #[prop(default = true)]
    pub show_percentage: bool,
    #[prop(default = Color::White)]
    pub fill_color: Color,
    #[prop(optional)]
    pub title: Option<String>,
}
