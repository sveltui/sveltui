use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::alert_types::*;
use crate::inline_button::{Press, inline_button};

/// A message banner that sits in the page. It never takes focus itself, so
/// showing one doesn't pull the user away from what they're doing; with
/// `on_dismiss` it gets a `[×]` button that's a normal tab stop.
#[component]
pub fn Alert(props: AlertProps) -> VNode {
    let dismiss_id = use_hook(FocusId::new);
    let (role, live) = if props.polite {
        (Role::Status, Live::Polite)
    } else {
        (Role::Alert, Live::Assertive)
    };
    let semantics = Semantics::new(role, props.title.clone().unwrap_or_default())
        .live(live)
        .atomic(true);
    let glyph = props.kind.glyph();
    let heading = match &props.title {
        Some(title) => format!("{glyph} {title}"),
        None => glyph.to_owned(),
    };
    let dismiss = props.on_dismiss.map(|on_dismiss| {
        let by_click = on_dismiss.clone();
        let press = Press {
            by_key: Rc::new(move |_| on_dismiss()),
            by_click: Rc::new(move || by_click()),
        };
        inline_button(dismiss_id, props.dismiss_label, "×".to_owned(), None, press)
    });
    view! {
        <Pressable semantics=semantics>
            <View border=BorderStyle::Single border_color=Color::DarkGrey
                padding={Padding::only(0, 1, 0, 1)} flex_direction=FlexDirection::Row gap=1>
                <View flex_grow=1.0>
                    <TextSpan bold=true>{heading}</TextSpan>
                    {props.children.render(())}
                </View>
                {dismiss}
            </View>
        </Pressable>
    }
    .into_node()
}
