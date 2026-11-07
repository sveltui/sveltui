use sveltui::prelude::*;

use super::dialog_types::*;

#[component]
pub fn Dialog(props: DialogProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let mut layer = DismissLayer::dialog();
    layer.scope.initial_focus = props.initial_focus;

    let title_str = props.title.unwrap_or_default();
    let semantics = Semantics {
        modal: Some(true),
        ..Semantics::new(Role::Dialog, title_str.clone())
    };

    view! {
        <Overlay
            id=id
            open=props.open
            placement=props.placement
            layer=layer
            backdrop=props.backdrop
            modal=true
            opener=props.opener
            on_close=props.on_close
            on_open=props.on_open
            on_key=props.on_key
            semantics=semantics
        >
            <View
                border=props.border
                border_color=props.border_color
                width=props.width
                height=props.height
                padding=props.padding
                title={(!title_str.is_empty()).then_some(title_str)}
            >
                {#if props.children.is_some()}
                    {@render props.children}
                {/if}
            </View>
        </Overlay>
    }
}
