use sveltui::prelude::*;

use super::popover_types::*;

#[component]
pub fn Popover(props: PopoverProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let managed = props.open.is_none();
    let open = props.open.unwrap_or_else(|| is_layer_open(id));

    view! {
        <Overlay
            id=id
            open=open
            managed=managed
            placement=props.placement
            backdrop=props.backdrop
            modal=false
            opt_on_close=props.on_close
            opt_on_open=props.on_open
            opt_on_key=props.on_key
            opt_slot_trigger=props.trigger
        >
            {#if props.children.is_some()}
                {@render props.children}
            {/if}
        </Overlay>
    }
}
