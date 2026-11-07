# AGENTS.md — Guidelines for AI Coding Agents

Welcome to the **sveltui** codebase. This document outlines architectural rules, workflows, and mandatory verification steps for any AI coding agent modifying or extending this repository.

---

## 1. Mandatory Code Verification (Build, Lint & Test)

Before concluding any work or reporting back to the user, **you MUST verify that your changes build cleanly, pass all lint checks without warnings, conform to standard formatting, and pass all unit tests.**

### Quick Verification Command

Run the full verification pipeline:
```bash
make verify
```
This single command executes:
1. `make fmt-check` (`cargo fmt --check`)
2. `make check` (`cargo check --workspace --all-targets`)
3. `make lint` (`cargo clippy --workspace --all-targets -- -D warnings`)
4. `make test` (`cargo test --workspace`)

### Individual Verification Commands

If any step in `make verify` fails, use the individual commands below to diagnose and resolve issues:

| Command | Tool | Purpose |
|---|---|---|
| `make check` | `cargo check --workspace --all-targets` | Type-checks all crates, tests, and examples |
| `make lint` | `cargo clippy --workspace --all-targets -- -D warnings` | Catches code quality and idiomatic Rust issues |
| `make fmt-check` | `cargo fmt --check` | Validates indentation and formatting |
| `make fmt` | `cargo fmt` | Automatically fixes code formatting |
| `make test` | `cargo test --workspace` | Runs all unit and integration tests |
| `make build` | `cargo build --workspace --all-targets` | Full debug compilation of all crates and examples |

> [!IMPORTANT]
> **Zero Warnings Policy**: All changes must compile with **0 compiler warnings** and **0 Clippy warnings** under `-D warnings`. Never ignore or suppress warnings without explicit justification.

---

## 2. Workspace & Architecture Overview

`sveltui` is a declarative, reactive Terminal User Interface (TUI) framework built from scratch on top of `crossterm`. It is structured as a Cargo workspace:

```
sveltui/
├── Cargo.toml                  # Workspace root manifest
├── Makefile                    # Standard developer & agent targets
├── AGENTS.md                   # Instructions and standards for agents
├── crates/
│   ├── sveltui/                  # Core crate (App, reactive, layout, terminal, element)
│   ├── sveltui-components/       # Built-in components (Button, TextInput, Dialog, etc.)
│   ├── sveltui-code/             # Syntax-highlighted code preview & editor
│   └── sveltui-macros/           # Declarative JSX macro (`view! { ... }`, `#[component]`)
└── examples/                   # `sveltui-examples` package (publish = false)
    ├── Cargo.toml                # Registers every example with an explicit [[example]]
    ├── hello_world/main.rs
    ├── framework/<name>/main.rs  # cargo run --example framework_<name>
    ├── components/<name>/        # cargo run --example component_<name>
    │   ├── main.rs
    │   └── <name>_tests.rs       # optional; example sets `test = true`
    └── code/editor/main.rs       # cargo run --example code_editor
```

### 2.1. Dependency Rule

The core `sveltui` crate depends on **no workspace crate except
`sveltui-macros`**, including dev-dependencies. Anything that needs
components (tests, examples) lives in `sveltui-components`, `sveltui-code`
or the `examples/` package.

### 2.2. Module & Test Layout

Each component in `sveltui-components` is a folder module:

```
src/button/
├── mod.rs                # declares the submodules and re-exports them
├── button_types.rs       # Props structs, bindings, enums, type aliases
├── button_component.rs   # #[component] functions and private helpers
└── button_tests.rs       # unit tests (`#[cfg(test)] mod button_tests;`)
```

- A unit test file lives next to its source as `<modname>_tests.rs` (for
  example `crates/sveltui/src/app/app_tests.rs`). Do not use `#[path]`.
- `crates/*/tests/` holds integration tests that span several modules or
  components (for example `sveltui-components/tests/keyboard.rs`). Name
  them after the topic, without a `_tests` suffix.

---

## 3. Core Development Principles

### 3.1. Pure JSX Syntax for Components
- Components are instantiated via JSX syntax directly inside `view! { ... }`:
  ```rust
  view! {
      <View border=BorderStyle::Single title=" Dialog ">
          <TextInput value=username set_value=set_username placeholder="User..." />
          <Divider title="Actions" />
          <Button variant=ButtonVariant::Primary on_click={move || submit()}>
              "Submit"
          </Button>
      </View>
  }
  ```
- **Do not** use raw block expressions (`{ Component::new(...).into_node() }`) for components when standard JSX tags (`<Component ... />`) are available.
- All components in `sveltui-components` implement:
  - `Default` (required for macro tag instantiation)
  - Builder setters returning `Self`
  - `IntoNode` (rendering into `VNode`)
  - `.child(...)` / `.label(...)` for child text or elements where appropriate.

### 3.2. Conservative "Base-UI" Aesthetic
- Stick to clean, minimal monochrome styling:
  - Foreground: `Color::White`
  - Secondary/Hint/Unfocused: `Color::DarkGrey`
  - Borders: `BorderStyle::Single` (unfocused/default), `BorderStyle::Double` (focused/active)
  - Avoid loud, multi-colored backgrounds unless specifically requested by the user.

### 3.3. Opt-in DOM-Like Focus Model
- Interactive elements register hit targets with a stable `FocusId`.
- Focus querying uses `use_focus(id)`:
  - `focus.is_focused()`: Element currently has keyboard focus.
  - `focus.has_focus_within()`: Element or any of its nested children contains focus (`:focus-within`).
  - `focus.is_active()`: Either focused or contains focus.
- Per-frame focus IDs are stabilized via `reset_focus_counter()` called before each render pass in `App`.

### 3.4. Signal-Driven Reactivity
- Signals are created using `let (read, write) = create_signal(val)`.
- Updates use `write.set(new_val)` or `write.update(|val| *val += 1)`.
- Triggering a write automatically marks the runtime dirty (`Runtime::set_dirty(true)`), which schedules an instant double-buffered terminal repaint on the next frame.

---

## 4. Git Commit Guidelines for AI Agents

When authoring or executing git commits as an AI agent, you must follow these standards:

- **100% AI-Generated Commit Prefix**: Any commit that is 100% AI generated **must** use the prefix `evil-` before the conventional commit type:
  - `evil-feat: ...`
  - `evil-fix: ...`
  - `evil-refactor: ...`
  - `evil-chore: ...`
  - `evil-docs: ...`
- **Subject Line Length**: Prefer ~50 characters for the commit subject line (keep concise and actionable).
- **Body / Description Wrap**: Wrap commit description/body text at a maximum of **72 characters** per line. Break to the next line when exceeded.
