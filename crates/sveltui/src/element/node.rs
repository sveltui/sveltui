use crate::element::focus::FocusId;
use crate::element::interaction::{
    Activation, DismissLayer, EventContext, EventResult, FocusGroup, FocusScope, Interaction,
    KeyHandler, Semantics,
};
use crate::element::snippet::Snippet;
use crate::layout::{
    BorderStyle, FlexDirection, Margin, Overflow, Padding, Placement, ScrollbarStyle,
    ScrollbarVisibility, Size,
};
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::fmt;
use std::rc::Rc;
use taffy::style::{AlignItems, AlignSelf, FlexWrap, JustifyContent};

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

    pub fn color(mut self, color: Color) -> Self {
        self.fg = Some(color);
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

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn opt_fg(mut self, color: Option<Color>) -> Self {
        self.fg = color;
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn opt_bg(mut self, color: Option<Color>) -> Self {
        self.bg = color;
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
    pub fn opener(mut self, id: FocusId) -> Self {
        self.opener = Some(id);
        self
    }
    pub fn anchor(mut self, id: FocusId) -> Self {
        self.anchor = Some(id);
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

    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        self.focus_group = Some(group);
        self
    }
    pub fn opt_on_key(mut self, handler: Option<KeyHandler>) -> Self {
        self.on_key = handler;
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.on_key = Some(Rc::new(handler));
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

    pub fn opt_trigger(mut self, trigger: Option<impl IntoNode>) -> Self {
        self.trigger = trigger.and_then(|t| {
            let node = t.into_node();
            if matches!(node, VNode::Empty) {
                None
            } else {
                Some(Box::new(node))
            }
        });
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

    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_open(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.on_open = handler;
        self
    }

    pub fn on_close(mut self, f: impl Fn() + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    pub fn opt_on_close(mut self, f: Option<Rc<dyn Fn()>>) -> Self {
        self.on_close = f;
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

pub trait IntoClickHandler {
    fn into_click_handler(self) -> Rc<dyn Fn()>;
}
impl<F: Fn() + 'static> IntoClickHandler for F {
    fn into_click_handler(self) -> Rc<dyn Fn()> {
        Rc::new(self)
    }
}
impl IntoClickHandler for Rc<dyn Fn()> {
    fn into_click_handler(self) -> Rc<dyn Fn()> {
        self
    }
}

pub trait IntoClickHandlerAt {
    fn into_click_handler_at(self) -> Rc<dyn Fn(u16, u16)>;
}
impl<F: Fn(u16, u16) + 'static> IntoClickHandlerAt for F {
    fn into_click_handler_at(self) -> Rc<dyn Fn(u16, u16)> {
        Rc::new(self)
    }
}
impl IntoClickHandlerAt for Rc<dyn Fn(u16, u16)> {
    fn into_click_handler_at(self) -> Rc<dyn Fn(u16, u16)> {
        self
    }
}

pub trait IntoKeyHandler {
    fn into_key_handler(self) -> KeyHandler;
}
impl<F: Fn(KeyEvent, &mut EventContext) -> EventResult + 'static> IntoKeyHandler for F {
    fn into_key_handler(self) -> KeyHandler {
        Rc::new(self)
    }
}
impl IntoKeyHandler for KeyHandler {
    fn into_key_handler(self) -> KeyHandler {
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

#[derive(Clone, Debug)]
pub struct ViewElement {
    pub width: Size,
    pub height: Size,
    pub min_width: Option<Size>,
    pub max_width: Option<Size>,
    pub min_height: Option<Size>,
    pub max_height: Option<Size>,
    pub margin: Margin,
    pub padding: Padding,
    pub gap: u16,
    pub flex_direction: FlexDirection,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub align_self: Option<AlignSelf>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_wrap: FlexWrap,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub scroll: Option<Box<ScrollData>>,
    pub visual: Option<Box<VisualData>>,
    pub children: Vec<VNode>,
}

impl Default for ViewElement {
    fn default() -> Self {
        Self {
            width: Size::default(),
            height: Size::default(),
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            margin: Margin::default(),
            padding: Padding::default(),
            gap: 0,
            flex_direction: FlexDirection::Column,
            align_items: None,
            justify_content: None,
            align_self: None,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_wrap: FlexWrap::NoWrap,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            scroll: None,
            visual: None,
            children: Vec::new(),
        }
    }
}

impl ViewElement {
    pub fn new() -> Self {
        Self::default()
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

    pub fn flex_direction(mut self, dir: FlexDirection) -> Self {
        self.flex_direction = dir;
        self
    }

    pub fn width(mut self, w: impl Into<Size>) -> Self {
        self.width = w.into();
        self
    }

    pub fn height(mut self, h: impl Into<Size>) -> Self {
        self.height = h.into();
        self
    }

    pub fn min_width(mut self, mw: impl Into<Size>) -> Self {
        self.min_width = Some(mw.into());
        self
    }

    pub fn opt_min_width(mut self, mw: Option<Size>) -> Self {
        self.min_width = mw;
        self
    }

    pub fn max_width(mut self, mw: impl Into<Size>) -> Self {
        self.max_width = Some(mw.into());
        self
    }

    pub fn opt_max_width(mut self, mw: Option<Size>) -> Self {
        self.max_width = mw;
        self
    }

    pub fn min_height(mut self, mh: impl Into<Size>) -> Self {
        self.min_height = Some(mh.into());
        self
    }

    pub fn opt_min_height(mut self, mh: Option<Size>) -> Self {
        self.min_height = mh;
        self
    }

    pub fn max_height(mut self, mh: impl Into<Size>) -> Self {
        self.max_height = Some(mh.into());
        self
    }

    pub fn opt_max_height(mut self, mh: Option<Size>) -> Self {
        self.max_height = mh;
        self
    }

    pub fn gap(mut self, g: u16) -> Self {
        self.gap = g;
        self
    }

    pub fn align_self(mut self, a: impl Into<AlignSelf>) -> Self {
        self.align_self = Some(a.into());
        self
    }

    pub fn flex_grow(mut self, fg: f32) -> Self {
        self.flex_grow = fg;
        self
    }

    pub fn flex_shrink(mut self, fs: f32) -> Self {
        self.flex_shrink = fs;
        self
    }

    pub fn flex_wrap(mut self, fw: FlexWrap) -> Self {
        self.flex_wrap = fw;
        self
    }

    pub fn border(mut self, b: BorderStyle) -> Self {
        self.visual_mut().border = b;
        self
    }

    pub fn border_color(mut self, c: Color) -> Self {
        self.visual_mut().border_color = Some(c);
        self
    }

    pub fn opt_border_color(mut self, c: Option<Color>) -> Self {
        self.visual_mut().border_color = c;
        self
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.visual_mut().title = Some(t.into());
        self
    }

    pub fn opt_title(mut self, t: Option<String>) -> Self {
        self.visual_mut().title = t;
        self
    }

    pub fn cursor_offset(mut self, co: (u16, u16)) -> Self {
        self.visual_mut().cursor_offset = Some(co);
        self
    }

    pub fn opt_cursor_offset(mut self, co: Option<(u16, u16)>) -> Self {
        self.visual_mut().cursor_offset = co;
        self
    }

    pub fn scroll_x(mut self, sx: u16) -> Self {
        self.scroll_mut().scroll_x = sx;
        self
    }

    pub fn scroll_y(mut self, sy: u16) -> Self {
        self.scroll_mut().scroll_y = sy;
        self
    }

    pub fn scrollbar_visibility(mut self, sv: ScrollbarVisibility) -> Self {
        self.scroll_mut().scrollbar_visibility = sv;
        self
    }

    pub fn scrollbar_style(mut self, ss: ScrollbarStyle) -> Self {
        self.scroll_mut().scrollbar_style = ss;
        self
    }

    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = p.into();
        self
    }

    pub fn margin(mut self, m: impl Into<Margin>) -> Self {
        self.margin = m.into();
        self
    }

    pub fn align_items(mut self, align: impl Into<AlignItems>) -> Self {
        self.align_items = Some(align.into());
        self
    }

    pub fn justify_content(mut self, justify: impl Into<JustifyContent>) -> Self {
        self.justify_content = Some(justify.into());
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.visual_mut().background_color = Some(color);
        self
    }

    pub fn foreground_color(mut self, color: Color) -> Self {
        self.visual_mut().foreground_color = Some(color);
        self
    }

    pub fn overflow_x(mut self, o: Overflow) -> Self {
        self.overflow_x = o;
        self
    }

    pub fn overflow_y(mut self, o: Overflow) -> Self {
        self.overflow_y = o;
        self
    }

    pub fn overflow(mut self, o: Overflow) -> Self {
        self.overflow_x = o;
        self.overflow_y = o;
        self
    }

    pub fn scroll_id(mut self, id: FocusId) -> Self {
        self.scroll_mut().id = Some(id);
        self
    }

    pub fn focus_id(mut self, id: FocusId) -> Self {
        self.scroll_mut().id = Some(id);
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

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoNode>) -> Self {
        for c in children {
            match c.into_node() {
                VNode::Fragment(items) => self.children.extend(items),
                VNode::Empty => {}
                other => self.children.push(other),
            }
        }
        self
    }
}

#[derive(Clone)]
pub struct ViewProps {
    pub width: Size,
    pub height: Size,
    pub min_width: Option<Size>,
    pub max_width: Option<Size>,
    pub min_height: Option<Size>,
    pub max_height: Option<Size>,
    pub margin: Margin,
    pub padding: Padding,
    pub gap: u16,
    pub flex_direction: FlexDirection,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub align_self: Option<AlignSelf>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_wrap: FlexWrap,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub scroll: Option<Box<ScrollData>>,
    pub visual: Option<Box<VisualData>>,
    pub children: Vec<VNode>,
}

impl Default for ViewProps {
    fn default() -> Self {
        Self {
            width: Size::default(),
            height: Size::default(),
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            margin: Margin::default(),
            padding: Padding::default(),
            gap: 0,
            flex_direction: FlexDirection::Column,
            align_items: None,
            justify_content: None,
            align_self: None,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_wrap: FlexWrap::NoWrap,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            scroll: None,
            visual: None,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ViewPropsBuilder {
    props: ViewProps,
}

impl ViewProps {
    pub fn builder() -> ViewPropsBuilder {
        ViewPropsBuilder::default()
    }
}

impl ViewPropsBuilder {
    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.props.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.props.height = height.into();
        self
    }

    pub fn min_width(mut self, min_width: impl Into<Size>) -> Self {
        self.props.min_width = Some(min_width.into());
        self
    }

    pub fn opt_min_width(mut self, min_width: Option<Size>) -> Self {
        self.props.min_width = min_width;
        self
    }

    pub fn max_width(mut self, max_width: impl Into<Size>) -> Self {
        self.props.max_width = Some(max_width.into());
        self
    }

    pub fn opt_max_width(mut self, max_width: Option<Size>) -> Self {
        self.props.max_width = max_width;
        self
    }

    pub fn min_height(mut self, min_height: impl Into<Size>) -> Self {
        self.props.min_height = Some(min_height.into());
        self
    }

    pub fn opt_min_height(mut self, min_height: Option<Size>) -> Self {
        self.props.min_height = min_height;
        self
    }

    pub fn max_height(mut self, max_height: impl Into<Size>) -> Self {
        self.props.max_height = Some(max_height.into());
        self
    }

    pub fn opt_max_height(mut self, max_height: Option<Size>) -> Self {
        self.props.max_height = max_height;
        self
    }

    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.props.margin = margin.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.props.padding = padding.into();
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.props.gap = gap;
        self
    }

    pub fn flex_direction(mut self, dir: FlexDirection) -> Self {
        self.props.flex_direction = dir;
        self
    }

    pub fn align_items(mut self, align: impl Into<AlignItems>) -> Self {
        self.props.align_items = Some(align.into());
        self
    }

    pub fn opt_align_items(mut self, align: Option<AlignItems>) -> Self {
        self.props.align_items = align;
        self
    }

    pub fn justify_content(mut self, justify: impl Into<JustifyContent>) -> Self {
        self.props.justify_content = Some(justify.into());
        self
    }

    pub fn opt_justify_content(mut self, justify: Option<JustifyContent>) -> Self {
        self.props.justify_content = justify;
        self
    }

    pub fn align_self(mut self, align: impl Into<AlignSelf>) -> Self {
        self.props.align_self = Some(align.into());
        self
    }

    pub fn opt_align_self(mut self, align: Option<AlignSelf>) -> Self {
        self.props.align_self = align;
        self
    }

    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.props.flex_grow = grow;
        self
    }

    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.props.flex_shrink = shrink;
        self
    }

    pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.props.flex_wrap = wrap;
        self
    }

    pub fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.props.overflow_x = overflow;
        self
    }

    pub fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.props.overflow_y = overflow;
        self
    }

    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.props.overflow_x = overflow;
        self.props.overflow_y = overflow;
        self
    }

    fn visual_mut(&mut self) -> &mut VisualData {
        self.props
            .visual
            .get_or_insert_with(|| Box::new(VisualData::default()))
            .as_mut()
    }

    fn scroll_mut(&mut self) -> &mut ScrollData {
        self.props
            .scroll
            .get_or_insert_with(|| Box::new(ScrollData::default()))
            .as_mut()
    }

    pub fn scroll_x(mut self, x: u16) -> Self {
        self.scroll_mut().scroll_x = x;
        self
    }

    pub fn scroll_y(mut self, y: u16) -> Self {
        self.scroll_mut().scroll_y = y;
        self
    }

    pub fn scroll_id(mut self, id: FocusId) -> Self {
        self.scroll_mut().id = Some(id);
        self
    }

    pub fn focus_id(mut self, id: FocusId) -> Self {
        self.scroll_mut().id = Some(id);
        self
    }

    pub fn scrollbar_visibility(mut self, vis: ScrollbarVisibility) -> Self {
        self.scroll_mut().scrollbar_visibility = vis;
        self
    }

    pub fn scrollbar_style(mut self, style: ScrollbarStyle) -> Self {
        self.scroll_mut().scrollbar_style = style;
        self
    }

    pub fn on_scroll(mut self, handler: impl IntoScrollHandler) -> Self {
        self.scroll_mut().on_scroll = Some(handler.into_scroll_handler());
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.visual_mut().background_color = Some(color);
        self
    }

    pub fn opt_background_color(mut self, color: Option<Color>) -> Self {
        self.visual_mut().background_color = color;
        self
    }

    pub fn foreground_color(mut self, color: Color) -> Self {
        self.visual_mut().foreground_color = Some(color);
        self
    }

    pub fn opt_foreground_color(mut self, color: Option<Color>) -> Self {
        self.visual_mut().foreground_color = color;
        self
    }

    pub fn bg(self, color: Color) -> Self {
        self.background_color(color)
    }

    pub fn opt_bg(self, color: Option<Color>) -> Self {
        self.opt_background_color(color)
    }

    pub fn fg(self, color: Color) -> Self {
        self.foreground_color(color)
    }

    pub fn opt_fg(self, color: Option<Color>) -> Self {
        self.opt_foreground_color(color)
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        self.visual_mut().border = border;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.visual_mut().border_color = Some(color);
        self
    }

    pub fn opt_border_color(mut self, color: Option<Color>) -> Self {
        self.visual_mut().border_color = color;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.visual_mut().title = Some(title.into());
        self
    }

    pub fn opt_title(mut self, title: Option<String>) -> Self {
        self.visual_mut().title = title;
        self
    }

    pub fn cursor_offset(mut self, offset: (u16, u16)) -> Self {
        self.visual_mut().cursor_offset = Some(offset);
        self
    }

    pub fn opt_cursor_offset(mut self, offset: Option<(u16, u16)>) -> Self {
        self.visual_mut().cursor_offset = offset;
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

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoNode>) -> Self {
        for c in children {
            match c.into_node() {
                VNode::Fragment(items) => self.props.children.extend(items),
                VNode::Empty => {}
                other => self.props.children.push(other),
            }
        }
        self
    }

    pub fn build(self) -> ViewProps {
        self.props
    }
}

#[allow(non_snake_case)]
pub fn View(props: ViewProps) -> ViewElement {
    ViewElement {
        width: props.width,
        height: props.height,
        min_width: props.min_width,
        max_width: props.max_width,
        min_height: props.min_height,
        max_height: props.max_height,
        margin: props.margin,
        padding: props.padding,
        gap: props.gap,
        flex_direction: props.flex_direction,
        align_items: props.align_items,
        justify_content: props.justify_content,
        align_self: props.align_self,
        flex_grow: props.flex_grow,
        flex_shrink: props.flex_shrink,
        flex_wrap: props.flex_wrap,
        overflow_x: props.overflow_x,
        overflow_y: props.overflow_y,
        scroll: props.scroll,
        visual: props.visual,
        children: props.children,
    }
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

    pub fn focus_id(mut self, id: impl Into<FocusId>) -> Self {
        self.focus_id = Some(id.into());
        self
    }

    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        self.focus_group = Some(group);
        self
    }

    pub fn opt_focus_group(mut self, group: Option<FocusGroup>) -> Self {
        self.focus_group = group;
        self
    }

    pub fn focus_scope(mut self, scope: FocusScope) -> Self {
        self.focus_scope = Some(scope);
        self
    }

    pub fn opt_focus_scope(mut self, scope: Option<FocusScope>) -> Self {
        self.focus_scope = scope;
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

    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.semantics = Some(semantics);
        self
    }

    pub fn on_click(mut self, handler: impl IntoClickHandler) -> Self {
        self.on_click = Some(handler.into_click_handler());
        self
    }

    pub fn on_click_at(mut self, handler: impl IntoClickHandlerAt) -> Self {
        self.on_click_at = Some(handler.into_click_handler_at());
        self
    }

    pub fn on_key(mut self, handler: impl IntoKeyHandler) -> Self {
        self.on_key = Some(handler.into_key_handler());
        self
    }

    pub fn interaction(mut self, i: Interaction) -> Self {
        self.interaction = Some(Box::new(i));
        self
    }

    pub fn opt_interaction(mut self, i: Option<Interaction>) -> Self {
        self.interaction = i.map(Box::new);
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
    pub fn focus_id(mut self, id: impl Into<FocusId>) -> Self {
        self.props.focus_id = Some(id.into());
        self
    }

    pub fn opt_focus_id(mut self, id: Option<FocusId>) -> Self {
        self.props.focus_id = id;
        self
    }

    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        self.props.focus_group = Some(group);
        self
    }

    pub fn opt_focus_group(mut self, group: Option<FocusGroup>) -> Self {
        self.props.focus_group = group;
        self
    }

    pub fn focus_scope(mut self, scope: FocusScope) -> Self {
        self.props.focus_scope = Some(scope);
        self
    }

    pub fn opt_focus_scope(mut self, scope: Option<FocusScope>) -> Self {
        self.props.focus_scope = scope;
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

    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.props.semantics = Some(semantics);
        self
    }

    pub fn opt_semantics(mut self, semantics: Option<Semantics>) -> Self {
        self.props.semantics = semantics;
        self
    }

    pub fn on_click(mut self, handler: impl IntoClickHandler) -> Self {
        self.props.on_click = Some(handler.into_click_handler());
        self
    }

    pub fn opt_on_click(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.props.on_click = handler;
        self
    }

    pub fn on_click_at(mut self, handler: impl IntoClickHandlerAt) -> Self {
        self.props.on_click_at = Some(handler.into_click_handler_at());
        self
    }

    pub fn opt_on_click_at(mut self, handler: Option<Rc<dyn Fn(u16, u16)>>) -> Self {
        self.props.on_click_at = handler;
        self
    }

    pub fn on_key(mut self, handler: impl IntoKeyHandler) -> Self {
        self.props.on_key = Some(handler.into_key_handler());
        self
    }

    pub fn opt_on_key(mut self, handler: Option<KeyHandler>) -> Self {
        self.props.on_key = handler;
        self
    }

    pub fn interaction(mut self, i: Interaction) -> Self {
        self.props.interaction = Some(Box::new(i));
        self
    }

    pub fn opt_interaction(mut self, i: Option<Interaction>) -> Self {
        self.props.interaction = i.map(Box::new);
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
        interaction: props.interaction,
        children: props.children,
    }
}

#[derive(Clone, Default)]
pub struct TextSpanProps {
    pub content: Option<String>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
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
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.props.content = Some(content.into());
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.props.fg = Some(color);
        self
    }
    pub fn fg(mut self, fg: Color) -> Self {
        self.props.fg = Some(fg);
        self
    }
    pub fn opt_fg(mut self, fg: Option<Color>) -> Self {
        self.props.fg = fg;
        self
    }
    pub fn opt_color(self, color: Option<Color>) -> Self {
        self.opt_fg(color)
    }
    pub fn bg(mut self, bg: Color) -> Self {
        self.props.bg = Some(bg);
        self
    }
    pub fn opt_bg(mut self, bg: Option<Color>) -> Self {
        self.props.bg = bg;
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
    if let Some(fg) = props.fg {
        el = el.fg(fg);
    }
    if let Some(bg) = props.bg {
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
    pub fg: Option<Color>,
    pub bg: Option<Color>,
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
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.props.content = Some(content.into());
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
    pub fn color(mut self, color: Color) -> Self {
        self.props.fg = Some(color);
        self
    }
    pub fn fg(mut self, fg: Color) -> Self {
        self.props.fg = Some(fg);
        self
    }
    pub fn opt_fg(mut self, fg: Option<Color>) -> Self {
        self.props.fg = fg;
        self
    }
    pub fn opt_color(self, color: Option<Color>) -> Self {
        self.opt_fg(color)
    }
    pub fn bg(mut self, bg: Color) -> Self {
        self.props.bg = Some(bg);
        self
    }
    pub fn opt_bg(mut self, bg: Option<Color>) -> Self {
        self.props.bg = bg;
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
    if let Some(fg) = props.fg {
        el = el.fg(fg);
    }
    if let Some(bg) = props.bg {
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
    pub fn id(mut self, id: FocusId) -> Self {
        self.props.id = Some(id);
        self
    }
    pub fn opener(mut self, opener: FocusId) -> Self {
        self.props.opener = Some(opener);
        self
    }
    pub fn opt_opener(mut self, opener: Option<FocusId>) -> Self {
        self.props.opener = opener;
        self
    }
    pub fn anchor(mut self, id: FocusId) -> Self {
        self.props.anchor = Some(id);
        self
    }
    pub fn layer(mut self, layer: DismissLayer) -> Self {
        self.props.layer = Some(layer);
        self
    }
    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        self.props.focus_group = Some(group);
        self
    }
    pub fn managed(mut self, managed: bool) -> Self {
        self.props.managed = managed;
        self
    }
    pub fn semantics(mut self, semantics: Semantics) -> Self {
        self.props.semantics = Some(semantics);
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
    pub fn on_key(
        mut self,
        handler: impl Fn(KeyEvent, &mut EventContext) -> EventResult + 'static,
    ) -> Self {
        self.props.on_key = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_key(mut self, handler: Option<KeyHandler>) -> Self {
        self.props.on_key = handler;
        self
    }
    pub fn on_close(mut self, handler: impl Fn() + 'static) -> Self {
        self.props.on_close = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_close(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.props.on_close = handler;
        self
    }
    pub fn on_open(mut self, handler: impl Fn() + 'static) -> Self {
        self.props.on_open = Some(Rc::new(handler));
        self
    }
    pub fn opt_on_open(mut self, handler: Option<Rc<dyn Fn()>>) -> Self {
        self.props.on_open = handler;
        self
    }
    pub fn trigger(mut self, trigger: impl IntoNode) -> Self {
        self.props.trigger = Some(trigger.into_node());
        self
    }
    pub fn opt_trigger(mut self, trigger: Option<VNode>) -> Self {
        self.props.trigger = trigger;
        self
    }
    pub fn slot_trigger(mut self, trigger: impl IntoNode) -> Self {
        self.props.slot_trigger = Some(Snippet::from(trigger.into_node()));
        self
    }
    pub fn opt_slot_trigger(mut self, trigger: Option<Snippet<()>>) -> Self {
        self.props.slot_trigger = trigger;
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
        el = el.opt_on_key(Some(ok));
    }
    if let Some(oc) = props.on_close {
        el = el.opt_on_close(Some(oc));
    }
    if let Some(oo) = props.on_open {
        el = el.opt_on_open(Some(oo));
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
