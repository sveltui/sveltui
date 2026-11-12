use crate::hover_popup::hover_popup_tests::{Harness, page};
use std::time::Duration;
use sveltui::prelude::*;

#[test]
fn the_tooltip_describes_its_trigger() {
    let mut page = Harness::new(page(state(0)));
    page.hover("Save");
    page.wait(Duration::from_millis(600));

    let nodes = page.tree.nodes();
    let tooltip = nodes
        .iter()
        .find(|n| n.interaction.semantics.role == Role::Tooltip)
        .expect("a tooltip");
    let described = nodes
        .iter()
        .find(|n| n.interaction.semantics.described_by == Some(tooltip.id))
        .expect("a trigger described by the tooltip");
    assert!(
        nodes
            .iter()
            .any(|n| n.parent == Some(described.id) || n.ancestors.contains(&described.id)),
        "the trigger wraps the button it describes"
    );
}
