use std::rc::Rc;
use std::time::Duration;
use sveltui::prelude::*;

#[derive(Props)]
pub struct RootProps {
    /// Whether the card shows. Use `bind:open` to share it, or pass a plain
    /// value and follow `on_open_change`.
    #[prop(bindable)]
    pub open: Bindable<bool>,
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
    /// How long the pointer or focus has to stay before it shows.
    #[prop(default = Duration::from_millis(600))]
    pub open_delay: Duration,
    /// How long it stays after the pointer and focus leave: time enough to
    /// move the pointer from the trigger onto the card.
    #[prop(default = Duration::from_millis(300))]
    pub close_delay: Duration,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TriggerProps {
    /// Makes the trigger itself a tab stop, for content that can't take
    /// focus (a name, say). Leave it off when wrapping a button.
    #[prop(default = false)]
    pub tab_stop: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ContentProps {
    #[prop(default = Placement::Bottom)]
    pub placement: Placement,
    #[prop(default = Dimension::Length(36.0))]
    pub width: Dimension,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
