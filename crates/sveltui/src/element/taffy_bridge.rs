use crate::element::node::{TextElement, VNode, ViewElement};
use crate::layout::geometry::Rect;
use crate::layout::{BorderStyle, Size};
use taffy::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct ComputedLayoutNode {
    pub rect: Rect,
    pub content_size: (u16, u16),
    pub scroll_size: (u16, u16),
    pub children: Vec<ComputedLayoutNode>,
}

pub struct TextMeasureData {
    pub content: String,
    pub wrap: bool,
    pub max_lines: Option<usize>,
}

pub fn view_to_taffy_style(view: &ViewElement, parent_direction: Option<FlexDirection>) -> taffy::style::Style {
    let border_inset = if view
        .visual
        .as_ref()
        .is_some_and(|v| v.border != BorderStyle::None)
    {
        1.0
    } else {
        0.0
    };

    let mut grow = view.flex_grow;
    let mut shrink = view.flex_shrink;
    let mut width_dim = view.width.to_taffy_dimension();
    let mut height_dim = view.height.to_taffy_dimension();
    let mut align_self = view.align_self;

    if let Some(dir) = parent_direction {
        match dir {
            FlexDirection::Row | FlexDirection::RowReverse => {
                if view.width == Size::Fill {
                    if grow == 0.0 {
                        grow = 1.0;
                    }
                    if shrink == 0.0 {
                        shrink = 1.0;
                    }
                    width_dim = Dimension::Length(0.0);
                }
                if view.height == Size::Fill {
                    if align_self.is_none() {
                        align_self = Some(AlignSelf::Stretch);
                    }
                    height_dim = Dimension::Auto;
                }
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                if view.height == Size::Fill {
                    if grow == 0.0 {
                        grow = 1.0;
                    }
                    if shrink == 0.0 {
                        shrink = 1.0;
                    }
                    height_dim = Dimension::Length(0.0);
                }
                if view.width == Size::Fill {
                    if align_self.is_none() {
                        align_self = Some(AlignSelf::Stretch);
                    }
                    width_dim = Dimension::Auto;
                }
            }
        }
    }

    taffy::style::Style {
        display: Display::Flex,
        flex_direction: view.flex_direction,
        align_items: view.align_items.or(Some(AlignItems::Stretch)),
        justify_content: view.justify_content,
        align_self,
        flex_grow: grow,
        flex_shrink: shrink,
        flex_wrap: view.flex_wrap,
        size: taffy::geometry::Size {
            width: width_dim,
            height: height_dim,
        },
        min_size: taffy::geometry::Size {
            width: view
                .min_width
                .map_or(Dimension::Auto, |s| s.to_taffy_dimension()),
            height: view
                .min_height
                .map_or(Dimension::Auto, |s| s.to_taffy_dimension()),
        },
        max_size: taffy::geometry::Size {
            width: view
                .max_width
                .map_or(Dimension::Auto, |s| s.to_taffy_dimension()),
            height: view
                .max_height
                .map_or(Dimension::Auto, |s| s.to_taffy_dimension()),
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
        scrollbar_width: if view.overflow_x == taffy::style::Overflow::Scroll || view.overflow_y == taffy::style::Overflow::Scroll { 1.0 } else { 0.0 },
        ..Default::default()
    }
}

pub struct LayoutEngine {
    tree: TaffyTree<TextMeasureData>,
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
        self.tree.clear();
        let taffy_root = self.build_node(root, None);

        let available = taffy::geometry::Size {
            width: AvailableSpace::Definite(available_w as f32),
            height: AvailableSpace::Definite(available_h as f32),
        };

        let _ = self.tree.compute_layout_with_measure(
            taffy_root,
            available,
            |known_dimensions, available_space, _node_id, context, _style| {
                if let Some(ctx) = context {
                    let max_width = match available_space.width {
                        AvailableSpace::Definite(w) => Some(w.max(0.0) as u16),
                        _ => None,
                    };
                    let text_elem = TextElement {
                        content: ctx.content.clone(),
                        wrap: ctx.wrap,
                        max_lines: ctx.max_lines,
                        ..Default::default()
                    };
                    let lines = crate::element::text::lines(&text_elem, max_width);
                    let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
                    let line_count = lines.len();
                    taffy::geometry::Size {
                        width: known_dimensions.width.unwrap_or(max_len as f32),
                        height: known_dimensions.height.unwrap_or(line_count as f32),
                    }
                } else {
                    taffy::geometry::Size {
                        width: known_dimensions.width.unwrap_or(0.0),
                        height: known_dimensions.height.unwrap_or(0.0),
                    }
                }
            },
        );

        self.extract_node_layout(taffy_root, root)
    }

    fn build_node(&mut self, node: &VNode, parent_direction: Option<FlexDirection>) -> NodeId {
        match node {
            VNode::View(view) => {
                let style = view_to_taffy_style(view, parent_direction);
                let dir = view.flex_direction;
                let child_ids: Vec<NodeId> =
                    view.children.iter().map(|c| self.build_node(c, Some(dir))).collect();
                self.tree
                    .new_with_children(style, &child_ids)
                    .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
            }
            VNode::Pressable(pressable) => {
                if pressable.children.len() == 1 {
                    self.build_node(&pressable.children[0], parent_direction)
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
                        .map(|c| self.build_node(c, Some(FlexDirection::Column)))
                        .collect();
                    self.tree
                        .new_with_children(style, &child_ids)
                        .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
                }
            }
            VNode::Text(text) => {
                let style = Style {
                    display: Display::Flex,
                    size: taffy::geometry::Size {
                        width: Dimension::Auto,
                        height: Dimension::Auto,
                    },
                    align_self: if text.fill.is_some() {
                        Some(AlignSelf::Stretch)
                    } else {
                        None
                    },
                    flex_grow: if text.fill.is_some() { 1.0 } else { 0.0 },
                    ..Default::default()
                };
                let context = TextMeasureData {
                    content: text.content.clone(),
                    wrap: text.wrap,
                    max_lines: text.max_lines,
                };
                self.tree
                    .new_leaf_with_context(style, context)
                    .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
            }
            VNode::Overlay(overlay) => {
                if let Some(trigger) = &overlay.trigger {
                    self.build_node(trigger, parent_direction)
                } else {
                    self.tree.new_leaf(Style::default()).unwrap()
                }
            }
            VNode::Fragment(children) => {
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
                    children.iter().map(|c| self.build_node(c, parent_direction.or(Some(FlexDirection::Column)))).collect();
                self.tree
                    .new_with_children(style, &child_ids)
                    .unwrap_or_else(|_| self.tree.new_leaf(Style::default()).unwrap())
            }
            VNode::FlexBox(_) | VNode::Empty => self.tree.new_leaf(Style::default()).unwrap(),
        }
    }

    fn extract_node_layout(
        &self,
        taffy_node: NodeId,
        vnode: &VNode,
    ) -> ComputedLayoutNode {
        let l = self
            .tree
            .layout(taffy_node)
            .cloned()
            .unwrap_or_default();

        let x = l.location.x.max(0.0).round() as u16;
        let y = l.location.y.max(0.0).round() as u16;
        let width = l.size.width.max(0.0).round() as u16;
        let height = l.size.height.max(0.0).round() as u16;
        let content_w = l.content_size.width.max(0.0).round() as u16;
        let content_h = l.content_size.height.max(0.0).round() as u16;

        let scroll_w = content_w.saturating_sub(width);
        let scroll_h = content_h.saturating_sub(height);

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
            content_size: (content_w, content_h),
            scroll_size: (scroll_w, scroll_h),
            children: computed_children,
        }
    }
}
