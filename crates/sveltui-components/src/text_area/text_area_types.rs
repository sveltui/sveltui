use crate::text_input::TextInputBinding;
use sveltui::prelude::*;

#[derive(Props)]
pub struct TextAreaProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub value: Option<TextInputBinding>,
    #[prop(optional)]
    pub set_value: Option<WriteSignal<String>>,
    #[prop(optional)]
    pub buffer: Option<ReadSignal<TextBuffer>>,
    #[prop(optional)]
    pub set_buffer: Option<WriteSignal<TextBuffer>>,
    #[prop(default = String::from("Type text here..."))]
    pub placeholder: String,
    #[prop(default = Dimension::Length(40.0))]
    pub width: Dimension,
    #[prop(default = Dimension::Length(8.0))]
    pub height: Dimension,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(default = false)]
    pub show_line_numbers: bool,
    #[prop(default = CursorStyle::Line)]
    pub cursor_style: CursorStyle,
}
