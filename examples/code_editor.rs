use sveltui::prelude::*;
use sveltui_code::prelude::*;

#[component]
fn CodeEditorDemo() -> impl IntoNode {
    let code = state(String::from(
        r#"// Interactive Code Editor with TextMate Highlighting!
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let result = fibonacci(10);
    println!("Fibonacci(10) = {}", result);
}
"#,
    ));

    view! {
        <View
            width=Dimension::Percent(1.0)
            height=Dimension::Percent(1.0)
            flex_direction=FlexDirection::Column
            padding=Padding::all(1)
            gap=1
        >
            <View
                flex_direction=FlexDirection::Row
                width=Dimension::Percent(1.0)
                height=Dimension::Percent(1.0)
                gap=2
            >
                <View
                    flex_direction=FlexDirection::Column
                    width=Dimension::Percent(0.5)
                    height=Dimension::Percent(1.0)
                    gap=0
                >
                    <CodeEditor
                        value=code
                        language="rs"
                        show_line_numbers=true
                        width=Dimension::Percent(1.0)
                        height=Dimension::Percent(1.0)
                        title="editor.rs (Editable)"
                    />
                </View>

                <View
                    flex_direction=FlexDirection::Column
                    width=Dimension::Percent(0.5)
                    height=Dimension::Percent(1.0)
                    gap=0
                >
                    <CodePreview
                        value=code
                        language="rs"
                        show_line_numbers=true
                        width=Dimension::Percent(1.0)
                        height=Dimension::Percent(1.0)
                        title="preview.rs (Live Preview)"
                    />
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <CodeEditorDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
