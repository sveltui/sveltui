use std::rc::Rc;
use std::time::Duration;
use sveltui::prelude::*;

#[derive(Props)]
pub struct RootProps {
    /// Whether the tooltip shows. Use `bind:open` to share it, or pass a
    /// plain value and follow `on_open_change`.
    #[prop(bindable)]
    pub open: Bindable<bool>,
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
    /// How long the pointer or focus has to stay before it shows.
    #[prop(default = Duration::from_millis(600))]
    pub open_delay: Duration,
    /// How long it stays after the pointer and focus leave.
    #[prop(default = Duration::ZERO)]
    pub close_delay: Duration,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TriggerProps {
    /// Makes the trigger itself a tab stop, for content that can't take
    /// focus. Leave it off when wrapping a button: focus inside counts.
    #[prop(default = false)]
    pub tab_stop: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ContentProps {
    #[prop(default = Placement::Top)]
    pub placement: Placement,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
