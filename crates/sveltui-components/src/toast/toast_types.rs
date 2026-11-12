use std::rc::Rc;
use std::time::Duration;
use sveltui::prelude::*;

use super::toast_manager::ToastManager;

/// Identifies a toast for `ToastManager::update` and `close`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ToastId(pub(crate) u64);

/// How urgently assistive tech announces a toast.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPriority {
    /// Announced when the user is idle (`role="status"`, `aria-live="polite"`).
    #[default]
    Low,
    /// Announced right away (`role="alert"`, `aria-live="assertive"`). Keep
    /// it for things the user has to act on.
    High,
}

/// A button on a toast that does something besides closing it, like "Undo".
#[derive(Clone)]
pub struct ToastAction {
    pub label: String,
    pub on_press: Rc<dyn Fn()>,
}

/// What a toast says and how it behaves. Build one with
/// `ToastOptions::new("Saved").description("main.rs")`.
#[derive(Clone, Default)]
pub struct ToastOptions {
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) priority: ToastPriority,
    pub(crate) timeout: Option<Duration>,
    pub(crate) action: Option<ToastAction>,
    pub(crate) on_close: Option<Rc<dyn Fn()>>,
}

impl ToastOptions {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn priority(mut self, priority: ToastPriority) -> Self {
        self.priority = priority;
        self
    }

    /// How long the toast stays up, instead of the provider's timeout.
    /// `Duration::ZERO` keeps it up until it's closed.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Adds a button that calls `on_press` and then closes the toast.
    pub fn action(mut self, label: impl Into<String>, on_press: impl Fn() + 'static) -> Self {
        self.action = Some(ToastAction {
            label: label.into(),
            on_press: Rc::new(on_press),
        });
        self
    }

    /// Called once the toast is gone, however it went: timed out, closed,
    /// or pushed out by newer ones.
    pub fn on_close(mut self, on_close: impl Fn() + 'static) -> Self {
        self.on_close = Some(Rc::new(on_close));
        self
    }
}

/// A toast as the manager lists it, for `Toast::Root toast={..}`.
#[derive(Clone, Default)]
pub struct ToastObject {
    pub id: ToastId,
    pub(crate) focus_id: FocusId,
    pub(crate) options: Rc<ToastOptions>,
}

impl ToastObject {
    pub fn title(&self) -> &str {
        &self.options.title
    }

    pub fn description(&self) -> Option<&str> {
        self.options.description.as_deref()
    }

    pub fn priority(&self) -> ToastPriority {
        self.options.priority
    }

    pub fn action(&self) -> Option<&ToastAction> {
        self.options.action.as_ref()
    }
}

#[derive(Props)]
pub struct ProviderProps {
    /// How long a toast stays up when it doesn't say.
    #[prop(default = Duration::from_secs(5))]
    pub timeout: Duration,
    /// The most toasts up at once. Adding one more closes the oldest.
    #[prop(default = 3)]
    pub limit: usize,
    /// A manager made with `ToastManager::new()`, to add toasts from code
    /// outside the provider. Leave it out and the provider makes its own.
    #[prop(optional)]
    pub manager: Option<ToastManager>,
    /// The key that moves focus to the newest toast.
    #[prop(default = crossterm::event::KeyCode::F(8))]
    pub hotkey: crossterm::event::KeyCode,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ViewportProps {
    /// The corner the toasts stack in. Only the corner placements
    /// (`TopStart`, `TopEnd`, `BottomStart`, `BottomEnd`) and `Top`/`Bottom`
    /// make sense here.
    #[prop(default = Placement::BottomEnd)]
    pub placement: Placement,
    #[prop(default = 40)]
    pub width: u16,
    /// What assistive tech calls the area.
    #[prop(default = "Notifications".to_string())]
    pub label: String,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct RootProps {
    pub toast: ToastObject,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TitleProps {
    /// Shown instead of the toast's title.
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct DescriptionProps {
    /// Shown instead of the toast's description.
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ActionProps {
    /// Shown instead of the action's label.
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct CloseProps {
    /// What assistive tech calls the button.
    #[prop(default = "Close".to_string())]
    pub label: String,
    /// Shown instead of `×`.
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

/// What a `Root` shares with the parts inside it.
#[derive(Clone)]
pub(crate) struct RootContext {
    pub(crate) toast: ToastObject,
    pub(crate) manager: ToastManager,
}
