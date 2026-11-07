use crate::*;
use sveltui::element::*;

#[test]
fn test_badge_measurement() {
    let badge = Badge(
        BadgeProps::builder()
            .children(Snippet::new(|_| "Ok"))
            .build(),
    );
    let node = badge.into_node();
    let (w, h) = measure_node(&node, 80, 24);
    assert!(w >= 4);
    assert_eq!(h, 3);
}
