use crate::*;
use sveltui::element::*;
use sveltui::layout::*;

#[test]
fn test_popover_component_into_node() {
    let popover = Popover(
        PopoverProps::builder()
            .open(true)
            .placement(Placement::BottomStart)
            .slot_trigger(Snippet::from(
                Button(
                    ButtonProps::builder()
                        .children(Snippet::from("Open Menu"))
                        .build(),
                )
                .into_node(),
            ))
            .children(Snippet::from(
                Button(
                    ButtonProps::builder()
                        .children(Snippet::from("Inside Popover"))
                        .build(),
                )
                .into_node(),
            ))
            .build(),
    )
    .into_node();

    match popover {
        VNode::Overlay(overlay) => {
            assert!(overlay.open);
            assert_eq!(overlay.placement, Placement::BottomStart);
            assert!(overlay.trigger.is_some());
            assert!(overlay.content.is_some());
        }
        _ => panic!("Expected VNode::Overlay"),
    }
}
