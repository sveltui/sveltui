use std::time::Duration;
use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod toast_tests;

/// The buttons that raise toasts. They live under the provider, so they can
/// reach its manager.
#[component]
fn Controls() -> impl IntoNode {
    let toasts = use_toast_manager();
    let undone = state(0u32);
    let count = state(0u32);

    let info = {
        let toasts = toasts.clone();
        move || {
            count.update(|n| *n += 1);
            toasts.add(ToastOptions::new("Saved").description(format!("draft-{}.md", count.get())));
        }
    };
    let success = {
        let toasts = toasts.clone();
        move || {
            toasts.add(ToastOptions::new("Deployed").description("v0.1.0 is live"));
        }
    };
    let error = {
        let toasts = toasts.clone();
        move || {
            toasts.add(
                ToastOptions::new("Build failed")
                    .priority(ToastPriority::High)
                    .description("3 errors in sveltui-image"),
            );
        }
    };
    let sticky = move || {
        toasts.add(
            ToastOptions::new("Archived 12 threads")
                .timeout(Duration::ZERO)
                .action("Undo", move || undone.update(|n| *n += 1)),
        );
    };

    view! {
        <View gap=1>
            <View flex_direction=FlexDirection::Row gap=1>
                <Button label="Save" on_click=info />
                <Button label="Deploy" on_click=success />
                <Button label="Build (urgent)" on_click=error />
                <Button label="Sticky + Undo" on_click=sticky />
            </View>
            <TextSpan foreground_color=Color::DarkGrey>
                {format!("Undo pressed {} times", undone.get())}
            </TextSpan>
        </View>
    }
}

#[component]
fn ToastDemo() -> impl IntoNode {
    let banner = state(true);
    view! {
        <Toast::Provider timeout={Duration::from_secs(5)} limit=4>
            <View padding=1 gap=1>
                {#if banner.get()}
                    <Alert kind=AlertKind::Warning title="Disk almost full"
                        on_dismiss={move || banner.set(false)}>
                        "Less than 1 GB left on /home."
                    </Alert>
                {/if}
                <Controls />
                <TextSpan foreground_color=Color::DarkGrey>
                    "Tab moves · F8 jumps to the newest toast · Esc closes it (or quits) · hover pauses"
                </TextSpan>
            </View>
            <Toast::Viewport placement=Placement::BottomEnd>
                {#each use_toast_manager().list() as toast}
                    <Toast::Root toast={toast.clone()} />
                {/each}
            </Toast::Viewport>
        </Toast::Provider>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ToastDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
