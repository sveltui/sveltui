use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod alert_dialog_tests;

#[component]
fn AlertDialogDemo() -> impl IntoNode {
    let open = state(false);
    let branches = state(vec!["main", "develop", "feature/toast"]);
    let outcome = state(String::from("Nothing deleted yet."));

    let target = branches.get().last().copied();
    let delete = move || {
        if let Some(name) = target {
            branches.update(|b| {
                b.pop();
            });
            outcome.set(format!("Deleted {name}."));
        }
    };
    let keep = move || outcome.set("Kept it.".to_string());

    view! {
        <View padding=1 gap=1>
            <TextSpan>{format!("Branches: {}", branches.get().join(", "))}</TextSpan>
            <AlertDialog::Root bind:open={open} outside_press=OutsidePress::Bell>
                <AlertDialog::Trigger label="Delete last branch" />
                <AlertDialog::Content>
                    <AlertDialog::Title>
                        {format!("Delete {}?", target.unwrap_or("nothing"))}
                    </AlertDialog::Title>
                    <AlertDialog::Description>
                        "Its unmerged commits will be lost. This can't be undone."
                    </AlertDialog::Description>
                    <View flex_direction=FlexDirection::Row gap=1
                        justify_content=JustifyContent::End>
                        <AlertDialog::Cancel label="Keep it" on_click=keep />
                        <AlertDialog::Action label="Delete" on_click=delete />
                    </View>
                </AlertDialog::Content>
            </AlertDialog::Root>
            <TextSpan foreground_color=Color::DarkGrey>{outcome.get()}</TextSpan>
            <TextSpan foreground_color=Color::DarkGrey>
                "Tab/Enter · focus starts on Keep it · Esc cancels · clicking outside rings the bell · q quits"
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <AlertDialogDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
