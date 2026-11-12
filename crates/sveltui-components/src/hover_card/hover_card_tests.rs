use crate::hover_popup::hover_popup_tests::{Harness, page};
use std::time::Duration;
use sveltui::prelude::*;

#[test]
fn the_card_is_a_group_that_takes_no_focus() {
    let mut page = Harness::new(page(state(0)));
    page.hover("@ada");
    page.wait(Duration::from_millis(600));
    assert!(page.shows("Ada Lovelace"), "{}", page.screen);

    let card = page
        .tree
        .nodes()
        .iter()
        .find(|n| n.interaction.semantics.role == Role::Group)
        .expect("the card");
    assert_eq!(card.interaction.focusability, Focusability::None);
    assert_eq!(
        page.focus.focused(),
        None,
        "showing the card moved no focus"
    );
}
