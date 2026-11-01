use crossterm::style::Color;
use sveltui::prelude::*;

#[derive(Props)]
pub struct BadgeProps {
    #[prop(default = Color::Grey)]
    pub bg: Color,
    #[prop(default = Color::Black)]
    pub fg: Color,
    #[prop(optional)]
    pub label: Option<String>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Badge(props: BadgeProps) -> impl IntoNode {
    view! {
        <View
            border=BorderStyle::Single
            border_color=props.fg
            background_color=props.bg
            foreground_color=props.fg
            align_items=AlignItems::Center
            justify_content=JustifyContent::Center
        >
        {#if props.children.is_some()}
            {@render props.children}
        {:else if let Some(label) = &props.label}
            <TextSpan foreground_color=props.fg>{label.clone()}</TextSpan>
        {/if}
        </View>
    }
}
