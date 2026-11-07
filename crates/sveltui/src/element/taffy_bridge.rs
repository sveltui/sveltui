use crate::element::node::{TextElement, VNode, ViewElement};
use crate::layout::BorderStyle;
use crate::layout::geometry::Rect;
use taffy::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct ComputedLayoutNode {
    pub rect: Rect,
    pub viewport_size: (u16, u16),
    pub content_size: (u16, u16),
    pub scroll_size: (u16, u16),
    pub children: Vec<ComputedLayoutNode>,
}

/// Measured sizes for one text node, keyed by the width it was measured
/// against. Taffy asks about the same text several times per pass (min,
/// max and definite widths), and wrapping is the expensive part.
#[derive(Default)]
struct TextSizes(Vec<(Option<u16>, (f32, f32))>);

impl TextSizes {
    fn get(&mut self, text: &TextElement, width: Option<u16>) -> (f32, f32) {
        if let Some(&(_, size)) = self.0.iter().find(|(w, _)| *w == width) {
            return size;
        }
        let (widest, count) = crate::element::text::size(text, width);
        let size = (widest as f32, count as f32);
        self.0.push((width, size));
        size
    }
}

pub fn view_to_taffy_style(view: &ViewElement) -> taffy::style::Style {
    let border_inset = if view
        .visual
        .as_ref()
        .is_some_and(|v| v.border != BorderStyle::None)
    {
        1.0
    } else {
        0.0
    };

    taffy::style::Style {
        display: Display::Flex,
        flex_direction: view.flex_direction,
        align_items: view.align_items.or(Some(AlignItems::Stretch)),
        justify_content: view.justify_content,
        align_self: view.align_self,
        flex_grow: view.flex_grow,
        flex_shrink: view.flex_shrink,
        flex_basis: view.flex_basis,
        flex_wrap: view.flex_wrap,
        size: taffy::geometry::Size {
            width: view.width,
            height: view.height,
        },
        min_size: taffy::geometry::Size {
            width: view.min_width.unwrap_or(Dimension::Auto),
            height: view.min_height.unwrap_or(Dimension::Auto),
        },
        max_size: taffy::geometry::Size {
            width: view.max_width.unwrap_or(Dimension::Auto),
            height: view.max_height.unwrap_or(Dimension::Auto),
        },
        gap: taffy::geometry::Size {
            width: LengthPercentage::Length(view.gap as f32),
            height: LengthPercentage::Length(view.gap as f32),
        },
        padding: taffy::geometry::Rect {
            left: LengthPercentage::Length(view.padding.left as f32),
            right: LengthPercentage::Length(view.padding.right as f32),
            top: LengthPercentage::Length(view.padding.top as f32),
            bottom: LengthPercentage::Length(view.padding.bottom as f32),
        },
        margin: taffy::geometry::Rect {
            left: LengthPercentageAuto::Length(view.margin.left as f32),
            right: LengthPercentageAuto::Length(view.margin.right as f32),
            top: LengthPercentageAuto::Length(view.margin.top as f32),
            bottom: LengthPercentageAuto::Length(view.margin.bottom as f32),
        },
        border: taffy::geometry::Rect {
            left: LengthPercentage::Length(border_inset),
            right: LengthPercentage::Length(border_inset),
            top: LengthPercentage::Length(border_inset),
            bottom: LengthPercentage::Length(border_inset),
        },
        overflow: taffy::geometry::Point {
            x: view.overflow_x,
            y: view.overflow_y,
        },
        scrollbar_width: if (view.overflow_y == taffy::style::Overflow::Scroll
            || view.overflow_x == taffy::style::Overflow::Scroll)
            && (view.width != Dimension::Auto || view.max_width.is_some())
            && (view.height != Dimension::Auto || view.max_height.is_some())
            && view
                .scroll
                .as_ref()
                .is_none_or(|s| s.scrollbar_visibility != crate::layout::ScrollbarVisibility::Never)
        {
            1.0
        } else {
            0.0
        },
        ..Default::default()
    }
}

/// Builds a taffy tree from a VNode tree and lays it out. The tree's
/// storage is kept between calls, so reusing one engine saves allocating
/// it again every frame.
pub struct LayoutEngine {
    /// Text leaves carry their index into the `texts` list built for each pass.
    tree: TaffyTree<usize>,
}

#[cfg(test)]
thread_local! {
    /// How many layout passes have run on this thread, so tests can check a
    /// frame doesn't lay out more than it needs to.
    pub(crate) static LAYOUT_PASSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

thread_local! {
    static SHARED_ENGINE: std::cell::RefCell<LayoutEngine> =
        std::cell::RefCell::new(LayoutEngine::new());
}

/// Lays out `root` with this thread's shared engine, so frames reuse its
/// storage. If the shared engine is already busy (a layout running inside
/// another), we fall back to a fresh one.
pub fn layout(root: &VNode, available_w: u16, available_h: u16) -> ComputedLayoutNode {
    SHARED_ENGINE.with(|engine| match engine.try_borrow_mut() {
        Ok(mut engine) => engine.compute_layout(root, available_w, available_h),
        Err(_) => LayoutEngine::new().compute_layout(root, available_w, available_h),
    })
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
        }
    }

    /// Computes full layout for a VNode tree within given available dimensions
    pub fn compute_layout(
        &mut self,
        root: &VNode,
        available_w: u16,
        available_h: u16,
    ) -> ComputedLayoutNode {
        #[cfg(test)]
        LAYOUT_PASSES.with(|passes| passes.set(passes.get() + 1));
        self.tree.clear();
        let mut texts = Vec::new();
        let taffy_root = self.build_node(root, &mut texts);

        let available = taffy::geometry::Size {
            width: AvailableSpace::Definite(available_w as f32),
            height: AvailableSpace::Definite(available_h as f32),
        };

        self.compute_taffy_layout(taffy_root, available, &texts);

        self.extract_node_layout(taffy_root, root)
    }

    fn compute_taffy_layout(
        &mut self,
        root: NodeId,
        available: taffy::geometry::Size<AvailableSpace>,
        texts: &[&TextElement],
    ) {
        let mut sizes: Vec<TextSizes> = texts.iter().map(|_| TextSizes::default()).collect();
        let _ = self.tree.compute_layout_with_measure(
            root,
            available,
            |known_dimensions, available_space, _node_id, context, _style| {
                let (width, height) = match context {
                    Some(&mut index) => {
                        let max_width = match available_space.width {
                            AvailableSpace::Definite(w) => Some(w.max(0.0) as u16),
                            _ => None,
                        };
                        sizes[index].get(texts[index], max_width)
                    }
                    None => (0.0, 0.0),
                };
                taffy::geometry::Size {
                    width: known_dimensions.width.unwrap_or(width),
                    height: known_dimensions.height.unwrap_or(height),
                }
            },
        );
    }

    fn build_node<'a>(&mut self, node: &'a VNode, texts: &mut Vec<&'a TextElement>) -> NodeId {
        match node {
            VNode::View(view) => {
                let style = view_to_taffy_style(view);
                let child_ids: Vec<NodeId> = view
                    .children
                    .iter()
                    .map(|child| self.build_node(child, texts))
                    .collect();
                self.tree
                    .new_with_children(style, &child_ids)
                    .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
            }
            VNode::Pressable(pressable) => {
                if pressable.children.len() == 1 {
                    self.build_node(&pressable.children[0], texts)
                } else if pressable.children.is_empty() {
                    self.tree.new_leaf(Style::default()).unwrap()
                } else {
                    let style = Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        size: taffy::geometry::Size {
                            width: Dimension::Auto,
                            height: Dimension::Auto,
                        },
                        ..Default::default()
                    };
                    let child_ids: Vec<NodeId> = pressable
                        .children
                        .iter()
                        .map(|c| self.build_node(c, texts))
                        .collect();
                    self.tree
                        .new_with_children(style, &child_ids)
                        .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
                }
            }
            VNode::Text(text) => {
                // Unwrapped text keeps its intrinsic minimum so stretching a
                // narrow parent doesn't silently truncate visible overflow.
                let min_size = if !text.wrap && text.fill.is_none() {
                    let lines = crate::element::text::lines(text, None);
                    taffy::geometry::Size {
                        width: Dimension::Length(
                            lines
                                .iter()
                                .map(|line| line.chars().count())
                                .max()
                                .unwrap_or(0) as f32,
                        ),
                        height: Dimension::Length(lines.len() as f32),
                    }
                } else {
                    taffy::geometry::Size::auto()
                };
                let style = Style {
                    display: Display::Flex,
                    min_size,
                    size: taffy::geometry::Size {
                        width: Dimension::Auto,
                        height: Dimension::Auto,
                    },
                    align_self: if text.fill.is_some() {
                        Some(AlignSelf::Stretch)
                    } else {
                        // Inherit the container's align_items just like View
                        // children; forcing FlexStart disables center/end/stretch.
                        None
                    },
                    flex_grow: if text.fill.is_some() { 1.0 } else { 0.0 },
                    flex_shrink: 0.0,
                    ..Default::default()
                };
                texts.push(text);
                self.tree
                    .new_leaf_with_context(style, texts.len() - 1)
                    .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
            }
            VNode::Overlay(overlay) => {
                if let Some(trigger) = &overlay.trigger {
                    self.build_node(trigger, texts)
                } else {
                    self.tree
                        .new_leaf(Style {
                            display: Display::None,
                            ..Default::default()
                        })
                        .unwrap()
                }
            }
            VNode::Fragment(children) => {
                if children.is_empty() {
                    self.tree
                        .new_leaf(Style {
                            display: Display::None,
                            ..Default::default()
                        })
                        .unwrap()
                } else {
                    let style = Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        size: taffy::geometry::Size {
                            width: Dimension::Auto,
                            height: Dimension::Auto,
                        },
                        ..Default::default()
                    };
                    let child_ids: Vec<NodeId> =
                        children.iter().map(|c| self.build_node(c, texts)).collect();
                    self.tree
                        .new_with_children(style, &child_ids)
                        .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
                }
            }
            VNode::Empty => self
                .tree
                .new_leaf(Style {
                    display: Display::None,
                    ..Default::default()
                })
                .unwrap(),
        }
    }

    fn extract_node_layout(&self, taffy_node: NodeId, vnode: &VNode) -> ComputedLayoutNode {
        // A single-child Pressable shares its child's Taffy node and geometry.
        if let VNode::Pressable(p) = vnode
            && p.children.len() == 1
        {
            return self.extract_node_layout(taffy_node, &p.children[0]);
        }
        let l = self.tree.layout(taffy_node).cloned().unwrap_or_default();

        let x = l.location.x.max(0.0).round() as u16;
        let y = l.location.y.max(0.0).round() as u16;
        let width = l.size.width.max(0.0).round() as u16;
        let height = l.size.height.max(0.0).round() as u16;
        let content_w = l.content_size.width.max(0.0).round() as u16;
        let content_h = l.content_size.height.max(0.0).round() as u16;

        let (content_w, content_h, viewport_w, viewport_h) = if let VNode::View(v) = vnode {
            let border = u16::from(
                v.visual
                    .as_ref()
                    .is_some_and(|v| v.border != BorderStyle::None),
            );
            let padding_w = v.padding.left.saturating_add(v.padding.right);
            let padding_h = v.padding.top.saturating_add(v.padding.bottom);
            // Taffy's content extent includes the leading border and both
            // padding edges. Scroll offsets and clipping use content coordinates.
            (
                content_w.saturating_sub(border.saturating_add(padding_w)),
                content_h.saturating_sub(border.saturating_add(padding_h)),
                width
                    .saturating_sub(border * 2)
                    .saturating_sub(padding_w)
                    .saturating_sub(l.scrollbar_size.width as u16),
                height
                    .saturating_sub(border * 2)
                    .saturating_sub(padding_h)
                    .saturating_sub(l.scrollbar_size.height as u16),
            )
        } else {
            (content_w, content_h, width, height)
        };

        let scroll_w = content_w.saturating_sub(viewport_w);
        let scroll_h = content_h.saturating_sub(viewport_h);

        let children_nodes = self.tree.children(taffy_node).unwrap_or_default();
        let child_vnodes = match vnode {
            VNode::View(view) => view.children.as_slice(),
            VNode::Pressable(pressable) => {
                if pressable.children.len() == 1 {
                    if let VNode::View(v) = &pressable.children[0] {
                        v.children.as_slice()
                    } else {
                        &[]
                    }
                } else {
                    pressable.children.as_slice()
                }
            }
            VNode::Fragment(children) => children.as_slice(),
            _ => &[],
        };

        let mut computed_children = Vec::new();
        for (idx, &child_taffy_id) in children_nodes.iter().enumerate() {
            let child_vnode = child_vnodes.get(idx).unwrap_or(&VNode::Empty);
            computed_children.push(self.extract_node_layout(child_taffy_id, child_vnode));
        }

        ComputedLayoutNode {
            rect: Rect::new(x, y, width, height),
            viewport_size: (viewport_w, viewport_h),
            content_size: (content_w, content_h),
            scroll_size: (scroll_w, scroll_h),
            children: computed_children,
        }
    }
}
