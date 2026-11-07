use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Props)]
pub struct TextAreaProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// The text. Pass it to control the text area yourself: edits then only
    /// call `on_value_change`, and the text resets unless you pass the new
    /// value back.
    /// Text shared with the parent, updated on every edit. Pass it as
    /// `bind:value={state}`.
    #[prop(bindable)]
    pub value: Bindable<String>,
    /// The text it starts with when nothing else decides.
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(String)>>,
    /// The whole buffer, cursor included. Pass it to control the buffer
    /// yourself; it takes over from `value`.
    /// A buffer shared with the parent, updated on every edit and cursor
    /// move. Pass it as `bind:buffer={state}`.
    #[prop(bindable)]
    pub buffer: Bindable<TextBuffer>,
    #[prop(optional)]
    pub on_buffer_change: Option<Rc<dyn Fn(TextBuffer)>>,
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
