use crate::element::node::{IntoNode, VNode};
use std::fmt;
use std::rc::Rc;

pub trait Props: Sized {
    type Builder: Default;
}

#[derive(Clone)]
pub struct Snippet<T = ()>(Option<Rc<dyn Fn(T) -> VNode>>);

impl<T> fmt::Debug for Snippet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snippet")
            .field("is_some", &self.is_some())
            .finish()
    }
}

impl<T> Default for Snippet<T> {
    fn default() -> Self {
        Self::none()
    }
}

impl<T> Snippet<T> {
    pub fn new<N: IntoNode>(f: impl Fn(T) -> N + 'static) -> Self {
        Self(Some(Rc::new(move |arg| f(arg).into_node())))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_snippet_present(&self) -> bool {
        self.is_some()
    }

    pub fn render(&self, arg: T) -> VNode {
        match &self.0 {
            Some(f) => f(arg),
            None => VNode::Empty,
        }
    }
}

impl Snippet<()> {
    pub fn render_empty(&self) -> VNode {
        self.render(())
    }
}

impl IntoNode for Snippet<()> {
    fn into_node(self) -> VNode {
        self.render(())
    }
}

impl IntoNode for &Snippet<()> {
    fn into_node(self) -> VNode {
        self.render(())
    }
}

impl From<&'static str> for Snippet<()> {
    fn from(s: &'static str) -> Self {
        Self::new(move |_| s)
    }
}

impl From<String> for Snippet<()> {
    fn from(s: String) -> Self {
        Self::new(move |_| s.clone())
    }
}

impl From<VNode> for Snippet<()> {
    fn from(node: VNode) -> Self {
        Self::new(move |_| node.clone())
    }
}

pub trait OptionSnippetExt<T> {
    fn is_snippet_present(&self) -> bool;
    fn render(&self, arg: T) -> VNode;
}

impl<T> OptionSnippetExt<T> for Option<Snippet<T>> {
    fn is_snippet_present(&self) -> bool {
        match self {
            Some(s) => s.is_some(),
            None => false,
        }
    }

    fn render(&self, arg: T) -> VNode {
        match self {
            Some(s) => s.render(arg),
            None => VNode::Empty,
        }
    }
}

impl<T> OptionSnippetExt<T> for Option<&Snippet<T>> {
    fn is_snippet_present(&self) -> bool {
        match self {
            Some(s) => s.is_some(),
            None => false,
        }
    }

    fn render(&self, arg: T) -> VNode {
        match self {
            Some(s) => s.render(arg),
            None => VNode::Empty,
        }
    }
}

pub trait IntoPropOption<T> {
    fn into_prop_option(self) -> Option<T>;
}

impl<T> IntoPropOption<T> for Option<T> {
    fn into_prop_option(self) -> Option<T> {
        self
    }
}

impl IntoPropOption<String> for String {
    fn into_prop_option(self) -> Option<String> {
        Some(self)
    }
}

impl IntoPropOption<String> for &str {
    fn into_prop_option(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoPropOption<String> for &String {
    fn into_prop_option(self) -> Option<String> {
        Some(self.clone())
    }
}

impl<T> IntoPropOption<Snippet<T>> for Snippet<T> {
    fn into_prop_option(self) -> Option<Snippet<T>> {
        if self.is_some() { Some(self) } else { None }
    }
}

impl IntoPropOption<Snippet<()>> for &'static str {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        Some(Snippet::from(self))
    }
}

impl IntoPropOption<Snippet<()>> for String {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        Some(Snippet::from(self))
    }
}

impl IntoPropOption<Snippet<()>> for VNode {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        Some(Snippet::from(self))
    }
}

impl IntoPropOption<Snippet<()>> for Option<VNode> {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        self.map(Snippet::from)
    }
}

impl<F: Fn() + 'static> IntoPropOption<Rc<dyn Fn()>> for F {
    fn into_prop_option(self) -> Option<Rc<dyn Fn()>> {
        Some(Rc::new(self))
    }
}

impl IntoPropOption<Rc<dyn Fn()>> for Rc<dyn Fn()> {
    fn into_prop_option(self) -> Option<Rc<dyn Fn()>> {
        Some(self)
    }
}

impl IntoPropOption<crate::element::focus::FocusId> for crate::element::focus::FocusId {
    fn into_prop_option(self) -> Option<crate::element::focus::FocusId> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::BorderStyle> for crate::layout::BorderStyle {
    fn into_prop_option(self) -> Option<crate::layout::BorderStyle> {
        Some(self)
    }
}

impl IntoPropOption<crossterm::style::Color> for crossterm::style::Color {
    fn into_prop_option(self) -> Option<crossterm::style::Color> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::Dimension> for crate::layout::Dimension {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::Dimension> for u16 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::Dimension::Length(f32::from(self)))
    }
}

impl IntoPropOption<crate::layout::Dimension> for i32 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::Dimension::Length(self.max(0) as f32))
    }
}

impl IntoPropOption<crate::layout::Dimension> for f32 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::IntoDimension::into_dimension(self))
    }
}

impl IntoPropOption<crate::layout::Dimension> for f64 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::IntoDimension::into_dimension(self))
    }
}

impl IntoPropOption<crate::layout::Padding> for u16 {
    fn into_prop_option(self) -> Option<crate::layout::Padding> {
        Some(crate::layout::Padding::all(self))
    }
}

impl IntoPropOption<crate::layout::Padding> for i32 {
    fn into_prop_option(self) -> Option<crate::layout::Padding> {
        Some(crate::layout::Padding::all(self as u16))
    }
}

impl IntoPropOption<crate::layout::Padding> for crate::layout::Padding {
    fn into_prop_option(self) -> Option<crate::layout::Padding> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::Margin> for u16 {
    fn into_prop_option(self) -> Option<crate::layout::Margin> {
        Some(crate::layout::Margin::all(self))
    }
}

impl IntoPropOption<crate::layout::Margin> for i32 {
    fn into_prop_option(self) -> Option<crate::layout::Margin> {
        Some(crate::layout::Margin::all(self as u16))
    }
}

impl IntoPropOption<crate::layout::Margin> for crate::layout::Margin {
    fn into_prop_option(self) -> Option<crate::layout::Margin> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::AlignItems> for taffy::style::AlignItems {
    fn into_prop_option(self) -> Option<taffy::style::AlignItems> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::JustifyContent> for taffy::style::JustifyContent {
    fn into_prop_option(self) -> Option<taffy::style::JustifyContent> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::FlexDirection> for taffy::style::FlexDirection {
    fn into_prop_option(self) -> Option<taffy::style::FlexDirection> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::FlexWrap> for taffy::style::FlexWrap {
    fn into_prop_option(self) -> Option<taffy::style::FlexWrap> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::Overflow> for taffy::style::Overflow {
    fn into_prop_option(self) -> Option<taffy::style::Overflow> {
        Some(self)
    }
}

impl IntoPropOption<bool> for bool {
    fn into_prop_option(self) -> Option<bool> {
        Some(self)
    }
}

impl IntoPropOption<u32> for u32 {
    fn into_prop_option(self) -> Option<u32> {
        Some(self)
    }
}

impl IntoPropOption<i32> for i32 {
    fn into_prop_option(self) -> Option<i32> {
        Some(self)
    }
}

impl IntoPropOption<u16> for u16 {
    fn into_prop_option(self) -> Option<u16> {
        Some(self)
    }
}

impl IntoPropOption<f32> for f32 {
    fn into_prop_option(self) -> Option<f32> {
        Some(self)
    }
}

impl IntoPropOption<f64> for f64 {
    fn into_prop_option(self) -> Option<f64> {
        Some(self)
    }
}

impl IntoPropOption<usize> for usize {
    fn into_prop_option(self) -> Option<usize> {
        Some(self)
    }
}

impl<T: 'static, F: Fn(T) + 'static> IntoPropOption<Rc<dyn Fn(T)>> for F {
    fn into_prop_option(self) -> Option<Rc<dyn Fn(T)>> {
        Some(Rc::new(self))
    }
}
impl<T: 'static> IntoPropOption<Rc<dyn Fn(T)>> for Rc<dyn Fn(T)> {
    fn into_prop_option(self) -> Option<Rc<dyn Fn(T)>> {
        Some(self)
    }
}
impl IntoPropOption<Option<String>> for Option<String> {
    fn into_prop_option(self) -> Option<Option<String>> {
        Some(self)
    }
}
impl IntoPropOption<crate::layout::Placement> for crate::layout::Placement {
    fn into_prop_option(self) -> Option<crate::layout::Placement> {
        Some(self)
    }
}

/// Implements `IntoPropOption<T> for T` for types passed to props as-is.
/// Together with the blanket `Option<T>` impl, one setter accepts both
/// `value` and `Option<value>`.
macro_rules! identity_prop_option {
    ($($ty:ty),* $(,)?) => {$(
        impl IntoPropOption<$ty> for $ty {
            fn into_prop_option(self) -> Option<$ty> {
                Some(self)
            }
        }
    )*};
}

identity_prop_option!(
    (u16, u16),
    crate::terminal::CursorStyle,
    crate::element::interaction::Semantics,
    crate::element::interaction::FocusScope,
    crate::element::interaction::FocusGroup,
    crate::element::interaction::Interaction,
);

impl IntoPropOption<crate::element::focus::FocusId> for usize {
    fn into_prop_option(self) -> Option<crate::element::focus::FocusId> {
        Some(self.into())
    }
}

impl<T> IntoPropOption<crate::reactive::ReadSignal<T>> for crate::reactive::ReadSignal<T> {
    fn into_prop_option(self) -> Option<crate::reactive::ReadSignal<T>> {
        Some(self)
    }
}

impl<T> IntoPropOption<crate::reactive::WriteSignal<T>> for crate::reactive::WriteSignal<T> {
    fn into_prop_option(self) -> Option<crate::reactive::WriteSignal<T>> {
        Some(self)
    }
}
