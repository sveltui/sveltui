use std::rc::Rc;
use sveltui::prelude::*;

/// What an alert is about. It only picks the glyph shown before the title,
/// so the colors stay monochrome.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl AlertKind {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Info => "i",
            Self::Success => "✓",
            Self::Warning => "!",
            Self::Error => "✗",
        }
    }
}

#[derive(Props)]
pub struct AlertProps {
    #[prop(default = AlertKind::Info)]
    pub kind: AlertKind,
    /// The headline, after a glyph for the kind.
    #[prop(optional)]
    pub title: Option<String>,
    /// Announce changes when the user is idle (`role="status"`) instead of
    /// right away (`role="alert"`).
    #[prop(default = false)]
    pub polite: bool,
    /// Adds a `[×]` button that calls this.
    #[prop(optional)]
    pub on_dismiss: Option<Rc<dyn Fn()>>,
    /// What assistive tech calls the dismiss button.
    #[prop(default = "Dismiss".to_string())]
    pub dismiss_label: String,
    /// The message.
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
