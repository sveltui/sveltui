use sveltui::prelude::*;

use super::hover_card_types::*;
use crate::hover_popup::{self, HoverPopup, Settings};

/// Each root's popup, kept apart from a tooltip's so they never mix.
#[derive(Clone)]
struct HoverCardContext(HoverPopup);

fn context(part: &str) -> HoverPopup {
    use_context::<HoverCardContext>()
        .unwrap_or_else(|| panic!("HoverCard::{part} requires HoverCard::Root"))
        .0
}

/// Shows a card about its trigger after the pointer or focus has stayed on
/// it for `open_delay`. The card stays while the pointer is on it.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let popup = HoverPopup::use_hover_popup(Settings {
        open: props.open,
        default_open: props.default_open,
        on_open_change: props.on_open_change,
        open_delay: props.open_delay,
        close_delay: props.close_delay,
        exclusive: false,
    });
    // Only keyboard focus counts: a click that focuses the trigger is a
    // press, and the pointer leaving should still close the popup.
    popup.update(use_focus(popup.ids.trigger).is_active_visible());
    provide_context(HoverCardContext(popup), || props.children.render(()))
}

/// What the card is about.
#[component]
pub fn Trigger(props: TriggerProps) -> VNode {
    let popup = context("Trigger");
    hover_popup::trigger(&popup, props.tab_stop, None, props.children.render(()))
}

/// The card.
#[component]
pub fn Content(props: ContentProps) -> VNode {
    let popup = context("Content");
    hover_popup::content(
        &popup,
        props.placement,
        Semantics::new(Role::Group, ""),
        props.width,
        Padding::only(0, 1, 0, 1),
        props.children.render(()),
    )
}
