use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Clone)]
pub enum TextInputBinding {
    Read(ReadSignal<String>),
    State(State<String>),
}

impl From<ReadSignal<String>> for TextInputBinding {
    fn from(s: ReadSignal<String>) -> Self {
        Self::Read(s)
    }
}

impl From<State<String>> for TextInputBinding {
    fn from(s: State<String>) -> Self {
        Self::State(s)
    }
}

impl IntoPropOption<TextInputBinding> for TextInputBinding {
    fn into_prop_option(self) -> Option<TextInputBinding> {
        Some(self)
    }
}

impl IntoPropOption<TextInputBinding> for ReadSignal<String> {
    fn into_prop_option(self) -> Option<TextInputBinding> {
        Some(TextInputBinding::Read(self))
    }
}

impl IntoPropOption<TextInputBinding> for State<String> {
    fn into_prop_option(self) -> Option<TextInputBinding> {
        Some(TextInputBinding::State(self))
    }
}

#[derive(Props)]
pub struct TextInputProps {
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
    #[prop(default = String::from("Type here..."))]
    pub placeholder: String,
    #[prop(default = Dimension::Length(24.0))]
    pub width: Dimension,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(optional)]
    pub on_submit: Option<Rc<dyn Fn(String)>>,
    #[prop(default = CursorStyle::Line)]
    pub cursor_style: CursorStyle,
    /// Gets every key before the input does. Return `EventResult::Handled` and
    /// the input won't process that key itself.
    #[prop(optional)]
    pub on_key: Option<KeyHandler>,
    /// The input's semantics. Defaults to `Role::Textbox`.
    #[prop(optional)]
    pub semantics: Option<Semantics>,
    /// Called when the input is clicked.
    #[prop(optional)]
    pub on_click: Option<Rc<dyn Fn()>>,
}
