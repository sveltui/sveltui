use crossterm::style::Color;
use sveltui::prelude::*;

#[derive(Props)]
pub struct DividerProps {
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(optional)]
    pub color: Option<Color>,
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[prop(default = 0.0)]
    pub flex_grow: f32,
    #[prop(default = 0.0)]
    pub flex_shrink: f32,
    #[prop(default = Dimension::Auto)]
    pub flex_basis: Dimension,
    #[prop(optional)]
    pub min_width: Option<Dimension>,
    #[prop(optional)]
    pub max_width: Option<Dimension>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Divider(props: DividerProps) -> impl IntoNode {
    let color = props.color.unwrap_or(Color::DarkGrey);
    let title = props.title.unwrap_or_default();
    let text = if !title.is_empty() {
        format!("─── {title} ")
    } else {
        String::new()
    };

    view! {
        <Pressable semantics=Semantics::new(Role::Separator, title)>
            <View
                border=BorderStyle::None
                padding=Padding::zero()
                width=props.width
                flex_grow=props.flex_grow
                flex_shrink=props.flex_shrink
                flex_basis=props.flex_basis
                height=1
                overflow=Overflow::Hidden
                align_items=AlignItems::Stretch
                opt_min_width=props.min_width
                opt_max_width=props.max_width
            >
                {#if props.children.is_some()}
                    {@render props.children}
                {:else}
                    <Paragraph fill='─' foreground_color=color>{text}</Paragraph>
                {/if}
            </View>
        </Pressable>
    }
}
