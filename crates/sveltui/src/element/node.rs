use crate::element::focus::FocusId;
use crate::element::interaction::{
    Activation, DismissLayer, EventContext, EventResult, FocusGroup, FocusScope, Interaction,
    KeyHandler, Semantics,
};
use crate::element::snippet::{IntoPropOption, Snippet};
use crate::layout::{
    BorderStyle, Dimension, FlexDirection, Inset, IntoDimension, Margin, Overflow, Padding,
    Placement, ScrollbarStyle, ScrollbarVisibility,
};
use crate::terminal::CursorStyle;
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::fmt;
use std::rc::Rc;
use taffy::style::{
    AlignContent, AlignItems, AlignSelf, Display, FlexWrap, JustifyContent, Position,
};

#[derive(Clone, Debug, Default)]
pub struct TextElement {
    pub content: String,
    /// Wrap paragraphs at word boundaries within their allocated width.
    pub wrap: bool,
    /// Maximum visible lines; truncated content ends with an ellipsis.
    pub max_lines: Option<usize>,
    /// Repeat this character after each line's content to its allocated width.
    /// This does not add intrinsic width; use a stretching parent to expand it.
    pub fill: Option<char>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl TextElement {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn color(mut self, color: impl IntoPropOption<Color>) -> Self {
        self.fg = color.into_prop_option();
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Fill the unused cells of each rendered line with a character.
    /// An empty string fills one line. Content is clipped to the allocation.
    pub fn fill(mut self, character: char) -> Self {
        self.fill = Some(character);
        self
    }

    pub fn fg(mut self, color: impl IntoPropOption<Color>) -> Self {
        self.fg = color.into_prop_option();
        self
    }

    pub fn bg(mut self, color: impl IntoPropOption<Color>) -> Self {
        self.bg = color.into_prop_option();
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }
}

/// Floating overlay element rendered in the top layer.
#[derive(Clone, Default)]
pub struct OverlayElement {
    pub id: FocusId,
    pub opener: Option<FocusId>,
    /// Layout identity used for positioning, independently of focus restoration.
    pub anchor: Option<FocusId>,
    pub layer: DismissLayer,
    pub focus_group: Option<FocusGroup>,
    pub managed: bool,
    pub on_key: Option<KeyHandler>,
    pub semantics: Semantics,
    pub trigger: Option<Box<VNode>>,
    pub content: Option<Box<VNode>>,
    pub open: bool,
    pub placement: Placement,
    pub backdrop: bool,
    pub modal: bool,
    pub on_close: Option<Rc<dyn Fn()>>,
    pub on_open: Option<Rc<dyn Fn()>>,
}

impl fmt::Debug for OverlayElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverlayElement")
            .field("trigger", &self.trigger)
            .field("content", &self.content)
            .field("open", &self.open)
            .field("placement", &self.placement)
            .field("backdrop", &self.backdrop)
            .field("modal", &self.modal)
            .field("has_close_handler", &self.on_close.is_some())
            .finish()
    }
}

impl OverlayElement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: FocusId) -> Self {
        self.id = id;
        self
    }
    pub fn opener(mut self, id: impl IntoPropOption<FocusId>) -> Self {
        self.opener = id.into_prop_option();
        self
    }
    pub fn anchor(mut self, id: impl IntoPropOption<FocusId>) -> Self {
        self.anchor = id.into_prop_option();
        self
    }
    pub fn layer(mut self, layer: DismissLayer) -> Self {
        self.modal = layer.scope.contain_tab;
        self.layer = layer;
        self
    }
    pub fn managed(mut self, managed: bool) -> Self {
        self.managed = managed;
        self
    }
    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.semantics = semantics;
        self
    }

    pub fn focus_group(mut self, group: impl IntoPropOption<FocusGroup>) -> Self {
        self.focus_group = group.into_prop_option();
        self
    }

    pub fn on_key(mut self, handler: impl IntoKeyHandler) -> Self {
        self.on_key = handler.into_key_handler();
        self
    }

    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        let node = trigger.into_node();
        if matches!(node, VNode::Empty) {
            self.trigger = None;
        } else {
            self.trigger = Some(Box::new(node));
        }
        self
    }

    pub fn content(mut self, content: impl IntoNode) -> Self {
        let node = content.into_node();
        if matches!(node, VNode::Empty) {
            self.content = None;
        } else {
            self.content = Some(Box::new(node));
        }
        self
    }

    pub fn child(self, child: impl IntoNode) -> Self {
        self.content(child)
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn backdrop(mut self, backdrop: bool) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self.layer = if modal {
            DismissLayer::dialog()
        } else {
            DismissLayer::popover()
        };
        self
    }

    pub fn on_open(mut self, handler: impl IntoClickHandler) -> Self {
        self.on_open = handler.into_click_handler();
        self
    }

    pub fn on_close(mut self, f: impl IntoClickHandler) -> Self {
        self.on_close = f.into_click_handler();
        self
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum VNode {
    View(Box<ViewElement>),
    Pressable(Box<PressableElement>),
    Text(TextElement),
    Overlay(OverlayElement),
    Fragment(Vec<VNode>),
    Empty,
}

pub trait IntoNode {
    fn into_node(self) -> VNode;
}

impl IntoNode for VNode {
    fn into_node(self) -> VNode {
        self
    }
}

impl IntoNode for ViewElement {
    fn into_node(self) -> VNode {
        VNode::View(Box::new(self))
    }
}

impl IntoNode for PressableElement {
    fn into_node(self) -> VNode {
        VNode::Pressable(Box::new(self))
    }
}

/// Turns a callback, or an `Option` of one, into a click handler. Passing
/// `None` clears it, so the same setter works whether you have a handler or
/// only maybe have one.
pub trait IntoClickHandler {
    fn into_click_handler(self) -> Option<Rc<dyn Fn()>>;
}
impl<F: Fn() + 'static> IntoClickHandler for F {
    fn into_click_handler(self) -> Option<Rc<dyn Fn()>> {
        Some(Rc::new(self))
    }
}
impl IntoClickHandler for Rc<dyn Fn()> {
    fn into_click_handler(self) -> Option<Rc<dyn Fn()>> {
        Some(self)
    }
}
impl IntoClickHandler for Option<Rc<dyn Fn()>> {
    fn into_click_handler(self) -> Option<Rc<dyn Fn()>> {
        self
    }
}

pub trait IntoClickHandlerAt {
    fn into_click_handler_at(self) -> Option<Rc<dyn Fn(u16, u16)>>;
}
impl<F: Fn(u16, u16) + 'static> IntoClickHandlerAt for F {
    fn into_click_handler_at(self) -> Option<Rc<dyn Fn(u16, u16)>> {
        Some(Rc::new(self))
    }
}
impl IntoClickHandlerAt for Rc<dyn Fn(u16, u16)> {
    fn into_click_handler_at(self) -> Option<Rc<dyn Fn(u16, u16)>> {
        Some(self)
    }
}
impl IntoClickHandlerAt for Option<Rc<dyn Fn(u16, u16)>> {
    fn into_click_handler_at(self) -> Option<Rc<dyn Fn(u16, u16)>> {
        self
    }
}

pub trait IntoKeyHandler {
    fn into_key_handler(self) -> Option<KeyHandler>;
}
impl<F: Fn(KeyEvent, &mut EventContext) -> EventResult + 'static> IntoKeyHandler for F {
    fn into_key_handler(self) -> Option<KeyHandler> {
        Some(Rc::new(self))
    }
}
impl IntoKeyHandler for KeyHandler {
    fn into_key_handler(self) -> Option<KeyHandler> {
        Some(self)
    }
}
impl IntoKeyHandler for Option<KeyHandler> {
    fn into_key_handler(self) -> Option<KeyHandler> {
        self
    }
}

pub trait IntoScrollHandler {
    fn into_scroll_handler(self) -> Rc<dyn Fn(i16, i16)>;
}
impl<F: Fn(i16, i16) + 'static> IntoScrollHandler for F {
    fn into_scroll_handler(self) -> Rc<dyn Fn(i16, i16)> {
        Rc::new(self)
    }
}
impl IntoScrollHandler for Rc<dyn Fn(i16, i16)> {
    fn into_scroll_handler(self) -> Rc<dyn Fn(i16, i16)> {
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct VisualData {
    pub background_color: Option<Color>,
    pub foreground_color: Option<Color>,
    pub border: BorderStyle,
    pub border_color: Option<Color>,
    pub title: Option<String>,
    pub cursor_offset: Option<(u16, u16)>,
    pub cursor_style: Option<CursorStyle>,
}

#[derive(Clone, Default)]
pub struct ScrollData {
    pub id: Option<FocusId>,
    pub scroll_x: u16,
    pub scroll_y: u16,
    pub scrollbar_visibility: ScrollbarVisibility,
    pub scrollbar_style: ScrollbarStyle,
    pub on_scroll: Option<Rc<dyn Fn(i16, i16)>>,
}

impl fmt::Debug for ScrollData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollData")
            .field("id", &self.id)
            .field("scroll_x", &self.scroll_x)
            .field("scroll_y", &self.scroll_y)
            .field("scrollbar_visibility", &self.scrollbar_visibility)
            .field("scrollbar_style", &self.scrollbar_style)
            .finish()
    }
}

/// Every `View` prop, written once. It's expanded into both `ViewElement`
/// and `ViewPropsBuilder`, which each provide `view_mut()` to reach the
/// element being built, so the two can't drift apart.
macro_rules! view_setters {
    () => {
        // Size

        pub fn width(mut self, width: impl IntoDimension) -> Self {
            self.view_mut().style.size.width = width.into_dimension();
            self
        }

        pub fn height(mut self, height: impl IntoDimension) -> Self {
            self.view_mut().style.size.height = height.into_dimension();
            self
        }

        pub fn min_width(mut self, width: impl IntoPropOption<Dimension>) -> Self {
            self.view_mut().style.min_size.width =
                width.into_prop_option().unwrap_or(Dimension::Auto);
            self
        }

        pub fn max_width(mut self, width: impl IntoPropOption<Dimension>) -> Self {
            self.view_mut().style.max_size.width =
                width.into_prop_option().unwrap_or(Dimension::Auto);
            self
        }

        pub fn min_height(mut self, height: impl IntoPropOption<Dimension>) -> Self {
            self.view_mut().style.min_size.height =
                height.into_prop_option().unwrap_or(Dimension::Auto);
            self
        }

        pub fn max_height(mut self, height: impl IntoPropOption<Dimension>) -> Self {
            self.view_mut().style.max_size.height =
                height.into_prop_option().unwrap_or(Dimension::Auto);
            self
        }

        /// Width over height. Terminal cells are about twice as tall as they
        /// are wide, so a square-looking box is roughly `2.0`.
        pub fn aspect_ratio(mut self, ratio: impl IntoPropOption<f32>) -> Self {
            self.view_mut().style.aspect_ratio = ratio.into_prop_option();
            self
        }

        // Spacing

        pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
            self.view_mut().style.padding = padding.into().to_taffy();
            self
        }

        pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
            self.view_mut().style.margin = margin.into().to_taffy();
            self
        }

        /// Space between children, both along and across the main axis.
        pub fn gap(mut self, gap: u16) -> Self {
            let gap = taffy::style::LengthPercentage::Length(f32::from(gap));
            self.view_mut().style.gap = taffy::geometry::Size {
                width: gap,
                height: gap,
            };
            self
        }

        /// Space between rows (wrapped lines in a row layout, or children in
        /// a column).
        pub fn row_gap(mut self, gap: u16) -> Self {
            self.view_mut().style.gap.height =
                taffy::style::LengthPercentage::Length(f32::from(gap));
            self
        }

        /// Space between columns (children in a row layout, or wrapped
        /// lines in a column).
        pub fn column_gap(mut self, gap: u16) -> Self {
            self.view_mut().style.gap.width =
                taffy::style::LengthPercentage::Length(f32::from(gap));
            self
        }

        // Flex

        pub fn display(mut self, display: Display) -> Self {
            self.view_mut().style.display = display;
            self
        }

        pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
            self.view_mut().style.flex_direction = direction;
            self
        }

        pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
            self.view_mut().style.flex_wrap = wrap;
            self
        }

        pub fn flex_grow(mut self, grow: f32) -> Self {
            self.view_mut().style.flex_grow = grow;
            self
        }

        pub fn flex_shrink(mut self, shrink: f32) -> Self {
            self.view_mut().style.flex_shrink = shrink;
            self
        }

        pub fn flex_basis(mut self, basis: impl IntoDimension) -> Self {
            self.view_mut().style.flex_basis = basis.into_dimension();
            self
        }

        pub fn align_items(mut self, align: impl IntoPropOption<AlignItems>) -> Self {
            self.view_mut().style.align_items = align.into_prop_option();
            self
        }

        pub fn align_self(mut self, align: impl IntoPropOption<AlignSelf>) -> Self {
            self.view_mut().style.align_self = align.into_prop_option();
            self
        }

        /// How wrapped lines share the space across the main axis.
        pub fn align_content(mut self, align: impl IntoPropOption<AlignContent>) -> Self {
            self.view_mut().style.align_content = align.into_prop_option();
            self
        }

        pub fn justify_content(mut self, justify: impl IntoPropOption<JustifyContent>) -> Self {
            self.view_mut().style.justify_content = justify.into_prop_option();
            self
        }

        // Position

        /// `Absolute` takes the view out of the flow and places it with
        /// `inset`, relative to its parent.
        pub fn position(mut self, position: Position) -> Self {
            self.view_mut().style.position = position;
            self
        }

        pub fn inset(mut self, inset: impl Into<Inset>) -> Self {
            self.view_mut().style.inset = inset.into().to_taffy();
            self
        }

        // Overflow and scrolling

        pub fn overflow(mut self, overflow: Overflow) -> Self {
            self.view_mut().style.overflow = taffy::geometry::Point {
                x: overflow,
                y: overflow,
            };
            self
        }

        pub fn overflow_x(mut self, overflow: Overflow) -> Self {
            self.view_mut().style.overflow.x = overflow;
            self
        }

        pub fn overflow_y(mut self, overflow: Overflow) -> Self {
            self.view_mut().style.overflow.y = overflow;
            self
        }

        pub fn scroll_x(mut self, x: u16) -> Self {
            self.view_mut().scroll_mut().scroll_x = x;
            self
        }

        pub fn scroll_y(mut self, y: u16) -> Self {
            self.view_mut().scroll_mut().scroll_y = y;
            self
        }

        pub fn scroll_id(mut self, id: impl IntoPropOption<FocusId>) -> Self {
            self.view_mut().scroll_mut().id = id.into_prop_option();
            self
        }

        pub fn focus_id(self, id: impl IntoPropOption<FocusId>) -> Self {
            self.scroll_id(id)
        }

        pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
            self.view_mut().scroll_mut().scrollbar_visibility = visibility;
            self
        }

        pub fn scrollbar_style(mut self, style: ScrollbarStyle) -> Self {
            self.view_mut().scroll_mut().scrollbar_style = style;
            self
        }

        pub fn on_scroll(mut self, handler: impl IntoScrollHandler) -> Self {
            self.view_mut().scroll_mut().on_scroll = Some(handler.into_scroll_handler());
            self
        }

        // Look

        pub fn background_color(mut self, color: impl IntoPropOption<Color>) -> Self {
            self.view_mut().visual_mut().background_color = color.into_prop_option();
            self
        }

        pub fn foreground_color(mut self, color: impl IntoPropOption<Color>) -> Self {
            self.view_mut().visual_mut().foreground_color = color.into_prop_option();
            self
        }

        pub fn bg(self, color: impl IntoPropOption<Color>) -> Self {
            self.background_color(color)
        }

        pub fn fg(self, color: impl IntoPropOption<Color>) -> Self {
            self.foreground_color(color)
        }

        /// The border style. Any border takes up one cell on each side.
        pub fn border(mut self, border: BorderStyle) -> Self {
            self.view_mut().visual_mut().border = border;
            self
        }

        pub fn border_color(mut self, color: impl IntoPropOption<Color>) -> Self {
            self.view_mut().visual_mut().border_color = color.into_prop_option();
            self
        }

        pub fn title(mut self, title: impl IntoPropOption<String>) -> Self {
            self.view_mut().visual_mut().title = title.into_prop_option();
            self
        }

        pub fn cursor_offset(mut self, offset: impl IntoPropOption<(u16, u16)>) -> Self {
            self.view_mut().visual_mut().cursor_offset = offset.into_prop_option();
            self
        }

        pub fn cursor_style(mut self, style: impl IntoPropOption<CursorStyle>) -> Self {
            self.view_mut().visual_mut().cursor_style = style.into_prop_option();
            self
        }

        // Children

        pub fn child(mut self, child: impl IntoNode) -> Self {
            self.view_mut().push_child(child.into_node());
            self
        }

        pub fn slot_children(self, child: impl IntoNode) -> Self {
            self.child(child)
        }

        pub fn children(mut self, children: impl IntoIterator<Item = impl IntoNode>) -> Self {
            for child in children {
                self.view_mut().push_child(child.into_node());
            }
            self
        }
    };
}

/// The box every layout is built from. Its layout lives in a taffy style,
/// so a frame hands it to taffy as is. What it looks like (`visual`) and
/// how it scrolls (`scroll`) are boxed, since most views use neither.
#[derive(Clone, Debug)]
pub struct ViewElement {
    pub(crate) style: taffy::Style,
    pub visual: Option<Box<VisualData>>,
    pub scroll: Option<Box<ScrollData>>,
    pub children: Vec<VNode>,
}

impl Default for ViewElement {
    fn default() -> Self {
        Self {
            style: taffy::Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_shrink: 1.0,
                ..Default::default()
            },
            visual: None,
            scroll: None,
            children: Vec::new(),
        }
    }
}

/// The props `View` takes. They're the element itself, so building a
/// `View` from them copies nothing.
pub type ViewProps = ViewElement;

/// Builds a `View`. `view!` uses this; it has the same setters as
/// `ViewElement`.
#[derive(Clone, Default)]
pub struct ViewPropsBuilder(ViewElement);

impl ViewElement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> ViewPropsBuilder {
        ViewPropsBuilder::default()
    }

    /// The layout style this view hands to taffy.
    pub fn style(&self) -> &taffy::Style {
        &self.style
    }

    pub fn visual_mut(&mut self) -> &mut VisualData {
        self.visual
            .get_or_insert_with(|| Box::new(VisualData::default()))
            .as_mut()
    }

    pub fn scroll_mut(&mut self) -> &mut ScrollData {
        self.scroll
            .get_or_insert_with(|| Box::new(ScrollData::default()))
            .as_mut()
    }

    fn view_mut(&mut self) -> &mut ViewElement {
        self
    }

    fn push_child(&mut self, child: VNode) {
        match child {
            // Fragments never become boxes. Their items join our children,
            // however deeply they're nested.
            VNode::Fragment(items) => items.into_iter().for_each(|item| self.push_child(item)),
            VNode::Empty => {}
            other => self.children.push(other),
        }
    }

    view_setters!();
}

impl ViewPropsBuilder {
    fn view_mut(&mut self) -> &mut ViewElement {
        &mut self.0
    }

    pub fn build(self) -> ViewProps {
        self.0
    }

    view_setters!();
}

#[allow(non_snake_case)]
pub fn View(props: ViewProps) -> ViewElement {
    props
}

#[derive(Clone, Default)]
pub struct PressableElement {
    pub focus_id: Option<FocusId>,
    pub focus_group: Option<FocusGroup>,
    pub focus_scope: Option<FocusScope>,
    pub tab_stop: bool,
    pub disabled: bool,
    pub hidden: bool,
    pub activation: Activation,
    pub semantics: Option<Semantics>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_click_at: Option<Rc<dyn Fn(u16, u16)>>,
    pub on_key: Option<KeyHandler>,
    /// Called when a key press or click moves focus onto this element.
    pub on_focus: Option<Rc<dyn Fn()>>,
    /// Called when the pointer moves onto this element.
    pub on_pointer_enter: Option<Rc<dyn Fn()>>,
    pub interaction: Option<Box<Interaction>>,
    pub children: Vec<VNode>,
}

impl fmt::Debug for PressableElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PressableElement")
            .field("focus_id", &self.focus_id)
            .field("tab_stop", &self.tab_stop)
            .field("disabled", &self.disabled)
            .field("activation", &self.activation)
            .field("has_click", &self.on_click.is_some())
            .field("children", &self.children)
            .finish()
    }
}

impl PressableElement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn focus_id(mut self, id: impl IntoPropOption<FocusId>) -> Self {
        self.focus_id = id.into_prop_option();
        self
    }

    pub fn focus_group(mut self, group: impl IntoPropOption<FocusGroup>) -> Self {
        self.focus_group = group.into_prop_option();
        self
    }

    pub fn focus_scope(mut self, scope: impl IntoPropOption<FocusScope>) -> Self {
        self.focus_scope = scope.into_prop_option();
        self
    }

    pub fn tab_stop(mut self, value: bool) -> Self {
        self.tab_stop = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn hidden(mut self, value: bool) -> Self {
        self.hidden = value;
        self
    }

    pub fn activation(mut self, value: Activation) -> Self {
        self.activation = value;
        self
    }

    pub fn semantics(mut self, semantics: impl IntoPropOption<Semantics>) -> Self {
        self.semantics = semantics.into_prop_option();
        self
    }

    pub fn on_click(mut self, handler: impl IntoClickHandler) -> Self {
        self.on_click = handler.into_click_handler();
        self
    }

    pub fn on_click_at(mut self, handler: impl IntoClickHandlerAt) -> Self {
        self.on_click_at = handler.into_click_handler_at();
        self
    }

    pub fn on_focus(mut self, handler: impl IntoClickHandler) -> Self {
        self.on_focus = handler.into_click_handler();
        self
    }

    pub fn on_pointer_enter(mut self, handler: impl IntoClickHandler) -> Self {
        self.on_pointer_enter = handler.into_click_handler();
        self
    }

    pub fn on_key(mut self, handler: impl IntoKeyHandler) -> Self {
        self.on_key = handler.into_key_handler();
        self
    }

    pub fn interaction(mut self, i: impl IntoPropOption<Interaction>) -> Self {
        self.interaction = i.into_prop_option().map(Box::new);
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        match child.into_node() {
            VNode::Fragment(items) => self.children.extend(items),
            VNode::Empty => {}
            other => self.children.push(other),
        }
        self
    }
}

#[derive(Clone, Default)]
pub struct PressableProps {
    pub focus_id: Option<FocusId>,
    pub focus_group: Option<FocusGroup>,
    pub focus_scope: Option<FocusScope>,
    pub tab_stop: bool,
    pub disabled: bool,
    pub hidden: bool,
    pub activation: Activation,
    pub semantics: Option<Semantics>,
    pub on_click: Option<Rc<dyn Fn()>>,
    pub on_click_at: Option<Rc<dyn Fn(u16, u16)>>,
    pub on_key: Option<KeyHandler>,
    /// Called when a key press or click moves focus onto this element.
    pub on_focus: Option<Rc<dyn Fn()>>,
    /// Called when the pointer moves onto this element.
    pub on_pointer_enter: Option<Rc<dyn Fn()>>,
    pub interaction: Option<Box<Interaction>>,
    pub children: Vec<VNode>,
}

#[derive(Clone, Default)]
pub struct PressablePropsBuilder {
    props: PressableProps,
}

impl PressableProps {
    pub fn builder() -> PressablePropsBuilder {
        PressablePropsBuilder::default()
    }
}

impl PressablePropsBuilder {
    pub fn focus_id(mut self, id: impl IntoPropOption<FocusId>) -> Self {
        self.props.focus_id = id.into_prop_option();
        self
    }

    pub fn focus_group(mut self, group: impl IntoPropOption<FocusGroup>) -> Self {
        self.props.focus_group = group.into_prop_option();
        self
    }

    pub fn focus_scope(mut self, scope: impl IntoPropOption<FocusScope>) -> Self {
        self.props.focus_scope = scope.into_prop_option();
        self
    }

    pub fn tab_stop(mut self, value: bool) -> Self {
        self.props.tab_stop = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.props.disabled = value;
        self
    }

    pub fn hidden(mut self, value: bool) -> Self {
        self.props.hidden = value;
        self
    }

    pub fn activation(mut self, value: Activation) -> Self {
        self.props.activation = value;
        self
    }

    pub fn semantics(mut self, semantics: impl IntoPropOption<Semantics>) -> Self {
        self.props.semantics = semantics.into_prop_option();
        self
    }

    pub fn on_click(mut self, handler: impl IntoClickHandler) -> Self {
        self.props.on_click = handler.into_click_handler();
        self
    }

    pub fn on_click_at(mut self, handler: impl IntoClickHandlerAt) -> Self {
        self.props.on_click_at = handler.into_click_handler_at();
        self
    }

    pub fn on_focus(mut self, handler: impl IntoClickHandler) -> Self {
        self.props.on_focus = handler.into_click_handler();
        self
    }

    pub fn on_pointer_enter(mut self, handler: impl IntoClickHandler) -> Self {
        self.props.on_pointer_enter = handler.into_click_handler();
        self
    }

    pub fn on_key(mut self, handler: impl IntoKeyHandler) -> Self {
        self.props.on_key = handler.into_key_handler();
        self
    }

    pub fn interaction(mut self, i: impl IntoPropOption<Interaction>) -> Self {
        self.props.interaction = i.into_prop_option().map(Box::new);
        self
    }

    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        match node.into_node() {
            VNode::Fragment(items) => self.props.children.extend(items),
            VNode::Empty => {}
            other => self.props.children.push(other),
        }
        self
    }

    pub fn child(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }

    pub fn build(self) -> PressableProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn Pressable(props: PressableProps) -> PressableElement {
    PressableElement {
        focus_id: props.focus_id,
        focus_group: props.focus_group,
        focus_scope: props.focus_scope,
        tab_stop: props.tab_stop,
        disabled: props.disabled,
        hidden: props.hidden,
        activation: props.activation,
        semantics: props.semantics,
        on_click: props.on_click,
        on_click_at: props.on_click_at,
        on_key: props.on_key,
        on_focus: props.on_focus,
        on_pointer_enter: props.on_pointer_enter,
        interaction: props.interaction,
        children: props.children,
    }
}

#[derive(Clone, Default)]
pub struct TextSpanProps {
    pub content: Option<String>,
    pub foreground_color: Option<Color>,
    pub background_color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub children: Option<Snippet<()>>,
}

#[derive(Clone, Default)]
pub struct TextSpanPropsBuilder {
    props: TextSpanProps,
}

impl TextSpanProps {
    pub fn builder() -> TextSpanPropsBuilder {
        TextSpanPropsBuilder::default()
    }
}

impl TextSpanPropsBuilder {
    pub fn content(mut self, content: impl IntoPropOption<String>) -> Self {
        self.props.content = content.into_prop_option();
        self
    }
    pub fn foreground_color(mut self, fg: impl IntoPropOption<Color>) -> Self {
        self.props.foreground_color = fg.into_prop_option();
        self
    }
    pub fn background_color(mut self, bg: impl IntoPropOption<Color>) -> Self {
        self.props.background_color = bg.into_prop_option();
        self
    }
    pub fn bold(mut self, bold: bool) -> Self {
        self.props.bold = bold;
        self
    }
    pub fn italic(mut self, italic: bool) -> Self {
        self.props.italic = italic;
        self
    }
    pub fn underline(mut self, underline: bool) -> Self {
        self.props.underline = underline;
        self
    }
    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        self.props.children = Some(Snippet::from(node.into_node()));
        self
    }
    pub fn children(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }
    pub fn build(self) -> TextSpanProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn TextSpan(props: TextSpanProps) -> TextElement {
    let mut text_content = props.content.unwrap_or_default();
    if let Some(children) = props.children {
        match children.render(()) {
            VNode::Text(t) => {
                if text_content.is_empty() {
                    text_content = t.content;
                } else {
                    text_content.push_str(&t.content);
                }
            }
            VNode::Fragment(nodes) => {
                for node in nodes {
                    if let VNode::Text(t) = node {
                        text_content.push_str(&t.content);
                    }
                }
            }
            _ => {}
        }
    }
    let mut el = TextElement::new(text_content);
    if let Some(fg) = props.foreground_color {
        el = el.fg(fg);
    }
    if let Some(bg) = props.background_color {
        el = el.bg(bg);
    }
    if props.bold {
        el = el.bold(true);
    }
    if props.italic {
        el = el.italic(true);
    }
    if props.underline {
        el = el.underline(true);
    }
    el
}

#[derive(Clone, Default)]
pub struct ParagraphProps {
    pub content: Option<String>,
    /// Limit wrapped lines, adding `…` when content remains. Zero hides all text.
    /// Omit this property to show every line.
    pub max_lines: Option<usize>,
    pub fill: Option<char>,
    pub foreground_color: Option<Color>,
    pub background_color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub children: Option<Snippet<()>>,
}

#[derive(Clone, Default)]
pub struct ParagraphPropsBuilder {
    props: ParagraphProps,
}

impl ParagraphProps {
    pub fn builder() -> ParagraphPropsBuilder {
        ParagraphPropsBuilder::default()
    }
}

impl ParagraphPropsBuilder {
    pub fn content(mut self, content: impl IntoPropOption<String>) -> Self {
        self.props.content = content.into_prop_option();
        self
    }
    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.props.max_lines = Some(max_lines);
        self
    }
    pub fn fill(mut self, fill: char) -> Self {
        self.props.fill = Some(fill);
        self
    }
    pub fn foreground_color(mut self, fg: impl IntoPropOption<Color>) -> Self {
        self.props.foreground_color = fg.into_prop_option();
        self
    }
    pub fn background_color(mut self, bg: impl IntoPropOption<Color>) -> Self {
        self.props.background_color = bg.into_prop_option();
        self
    }
    pub fn bold(mut self, bold: bool) -> Self {
        self.props.bold = bold;
        self
    }
    pub fn italic(mut self, italic: bool) -> Self {
        self.props.italic = italic;
        self
    }
    pub fn underline(mut self, underline: bool) -> Self {
        self.props.underline = underline;
        self
    }
    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        self.props.children = Some(Snippet::from(node.into_node()));
        self
    }
    pub fn children(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }
    pub fn build(self) -> ParagraphProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn Paragraph(props: ParagraphProps) -> TextElement {
    let mut text_content = props.content.unwrap_or_default();
    if let Some(children) = props.children {
        match children.render(()) {
            VNode::Text(t) => {
                if text_content.is_empty() {
                    text_content = t.content;
                } else {
                    text_content.push_str(&t.content);
                }
            }
            VNode::Fragment(nodes) => {
                for node in nodes {
                    if let VNode::Text(t) = node {
                        text_content.push_str(&t.content);
                    }
                }
            }
            _ => {}
        }
    }
    // Filled text is used for single-line rules such as Divider labels.
    let mut el = TextElement::new(text_content).wrap(props.fill.is_none());
    if let Some(max_lines) = props.max_lines {
        el = el.max_lines(max_lines);
    }
    if let Some(fill) = props.fill {
        el = el.fill(fill);
    }
    if let Some(fg) = props.foreground_color {
        el = el.fg(fg);
    }
    if let Some(bg) = props.background_color {
        el = el.bg(bg);
    }
    if props.bold {
        el = el.bold(true);
    }
    if props.italic {
        el = el.italic(true);
    }
    if props.underline {
        el = el.underline(true);
    }
    el
}

#[derive(Clone, Default)]
pub struct OverlayProps {
    pub id: Option<FocusId>,
    pub opener: Option<FocusId>,
    /// Layout identity used for positioning, independently of focus restoration.
    pub anchor: Option<FocusId>,
    pub layer: Option<DismissLayer>,
    pub focus_group: Option<FocusGroup>,
    pub managed: bool,
    pub semantics: Option<Semantics>,
    pub open: bool,
    pub placement: Placement,
    pub backdrop: bool,
    pub modal: bool,
    pub on_key: Option<KeyHandler>,
    pub on_close: Option<Rc<dyn Fn()>>,
    pub on_open: Option<Rc<dyn Fn()>>,
    pub trigger: Option<VNode>,
    pub slot_trigger: Option<Snippet<()>>,
    pub children: Option<Snippet<()>>,
}

#[derive(Clone, Default)]
pub struct OverlayPropsBuilder {
    props: OverlayProps,
}

impl OverlayProps {
    pub fn builder() -> OverlayPropsBuilder {
        OverlayPropsBuilder::default()
    }
}

impl OverlayPropsBuilder {
    pub fn id(mut self, id: impl IntoPropOption<FocusId>) -> Self {
        self.props.id = id.into_prop_option();
        self
    }
    pub fn opener(mut self, opener: impl IntoPropOption<FocusId>) -> Self {
        self.props.opener = opener.into_prop_option();
        self
    }
    pub fn anchor(mut self, id: impl IntoPropOption<FocusId>) -> Self {
        self.props.anchor = id.into_prop_option();
        self
    }
    pub fn layer(mut self, layer: DismissLayer) -> Self {
        self.props.layer = Some(layer);
        self
    }
    pub fn focus_group(mut self, group: impl IntoPropOption<FocusGroup>) -> Self {
        self.props.focus_group = group.into_prop_option();
        self
    }
    pub fn managed(mut self, managed: bool) -> Self {
        self.props.managed = managed;
        self
    }
    pub fn semantics(mut self, semantics: impl IntoPropOption<Semantics>) -> Self {
        self.props.semantics = semantics.into_prop_option();
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.props.open = open;
        self
    }
    pub fn placement(mut self, placement: Placement) -> Self {
        self.props.placement = placement;
        self
    }
    pub fn backdrop(mut self, backdrop: bool) -> Self {
        self.props.backdrop = backdrop;
        self
    }
    pub fn modal(mut self, modal: bool) -> Self {
        self.props.modal = modal;
        self
    }
    pub fn on_key(mut self, handler: impl IntoKeyHandler) -> Self {
        self.props.on_key = handler.into_key_handler();
        self
    }
    pub fn on_close(mut self, handler: impl IntoClickHandler) -> Self {
        self.props.on_close = handler.into_click_handler();
        self
    }
    pub fn on_open(mut self, handler: impl IntoClickHandler) -> Self {
        self.props.on_open = handler.into_click_handler();
        self
    }
    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        let node = trigger.into_node();
        self.props.trigger = (!matches!(node, VNode::Empty)).then_some(node);
        self
    }
    pub fn slot_trigger(mut self, trigger: impl IntoPropOption<Snippet<()>>) -> Self {
        self.props.slot_trigger = trigger.into_prop_option();
        self
    }
    pub fn slot_children(mut self, node: impl IntoNode) -> Self {
        self.props.children = Some(Snippet::from(node.into_node()));
        self
    }
    pub fn children(self, node: impl IntoNode) -> Self {
        self.slot_children(node)
    }
    pub fn build(self) -> OverlayProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn Overlay(props: OverlayProps) -> OverlayElement {
    let mut el = OverlayElement::new();
    if let Some(id) = props.id {
        el = el.id(id);
    }
    if let Some(opener) = props.opener {
        el = el.opener(opener);
    }
    if let Some(anchor) = props.anchor {
        el = el.anchor(anchor);
    }
    if let Some(layer) = props.layer {
        el = el.layer(layer);
    }
    if let Some(fg) = props.focus_group {
        el = el.focus_group(fg);
    }
    if props.managed {
        el = el.managed(true);
    }
    if let Some(sem) = props.semantics {
        el = el.semantics(sem);
    }
    el = el.open(props.open);
    el = el.placement(props.placement);
    el = el.backdrop(props.backdrop);
    if props.modal {
        el = el.modal(true);
    }
    if let Some(ok) = props.on_key {
        el = el.on_key(ok);
    }
    if let Some(oc) = props.on_close {
        el = el.on_close(oc);
    }
    if let Some(oo) = props.on_open {
        el = el.on_open(oo);
    }
    if let Some(trigger) = props.trigger {
        el = el.trigger(trigger);
    } else if let Some(slot_trigger) = props.slot_trigger {
        el = el.trigger(slot_trigger.render(()));
    }
    if let Some(children) = props.children {
        el = el.child(children.render(()));
    }
    el
}

impl IntoNode for TextElement {
    fn into_node(self) -> VNode {
        VNode::Text(self)
    }
}

impl IntoNode for OverlayElement {
    fn into_node(self) -> VNode {
        VNode::Overlay(self)
    }
}

impl IntoNode for &str {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self))
    }
}

impl IntoNode for String {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self))
    }
}

impl IntoNode for &String {
    fn into_node(self) -> VNode {
        VNode::Text(TextElement::new(self.as_str()))
    }
}

impl<T: IntoNode> IntoNode for Option<T> {
    fn into_node(self) -> VNode {
        match self {
            Some(v) => v.into_node(),
            None => VNode::Empty,
        }
    }
}

impl<T: IntoNode> IntoNode for Vec<T> {
    fn into_node(self) -> VNode {
        VNode::Fragment(self.into_iter().map(|item| item.into_node()).collect())
    }
}

impl IntoNode for () {
    fn into_node(self) -> VNode {
        VNode::Empty
    }
}
