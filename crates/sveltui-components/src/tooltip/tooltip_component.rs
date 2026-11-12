use sveltui::prelude::*;

use super::tooltip_types::*;
use crate::hover_popup::{self, HoverPopup, Settings};

/// Each root's popup, kept apart from a hover card's so they never mix.
#[derive(Clone)]
struct TooltipContext(HoverPopup);

fn context(part: &str) -> HoverPopup {
    use_context::<TooltipContext>()
        .unwrap_or_else(|| panic!("Tooltip::{part} requires Tooltip::Root"))
        .0
}

/// Shows a short description of its trigger after the pointer or focus has
/// stayed on it for `open_delay`.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let popup = HoverPopup::use_hover_popup(Settings {
        open: props.open,
        default_open: props.default_open,
        on_open_change: props.on_open_change,
        open_delay: props.open_delay,
        close_delay: props.close_delay,
        exclusive: true,
    });
    // Only keyboard focus counts: a click that focuses the trigger is a
    // press, and the pointer leaving should still close the popup.
    popup.update(use_focus(popup.ids.trigger).is_active_visible());
    provide_context(TooltipContext(popup), || props.children.render(()))
}

/// What the tooltip describes. Assistive tech reads the tooltip as its
/// description.
#[component]
pub fn Trigger(props: TriggerProps) -> VNode {
    let popup = context("Trigger");
    let semantics = Semantics::new(Role::Generic, "").described_by(popup.ids.content);
    hover_popup::trigger(
        &popup,
        props.tab_stop,
        Some(semantics),
        props.children.render(()),
    )
}

/// The tooltip's text.
#[component]
pub fn Content(props: ContentProps) -> VNode {
    let popup = context("Content");
    hover_popup::content(
        &popup,
        props.placement,
        Semantics::new(Role::Tooltip, ""),
        Dimension::Auto,
        Padding::only(0, 1, 0, 1),
        props.children.render(()),
    )
}
