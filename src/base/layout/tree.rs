use crate::base::font_atlas::FontAtlas;
use crate::base::layout::style::{Content, PanelStyle};
use taffy::prelude::*;

/// Wrapper around TaffyTree that associates PanelStyle with each node
pub struct LayoutTree {
    pub taffy: TaffyTree<PanelStyle>,
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
        }
    }

    /// Create a new leaf node (no children)
    pub fn new_leaf(&mut self, style: Style, panel_style: PanelStyle) -> NodeId {
        self.taffy
            .new_leaf_with_context(style, panel_style)
            .expect("Failed to create leaf node")
    }

    /// Create a new node with children
    pub fn new_with_children(
        &mut self,
        style: Style,
        panel_style: PanelStyle,
        children: &[NodeId],
    ) -> NodeId {
        self.taffy
            .new_with_children(style, children)
            .map(|id| {
                self.taffy.set_node_context(id, Some(panel_style)).ok();
                id
            })
            .expect("Failed to create node with children")
    }

    /// Add a child to a parent node
    #[allow(dead_code)]
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.taffy
            .add_child(parent, child)
            .expect("Failed to add child");
    }

    /// Compute layout for the tree starting at root (without text measurement)
    pub fn compute(&mut self, root: NodeId, width: f32, height: f32) {
        self.taffy
            .compute_layout(
                root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
            )
            .expect("Failed to compute layout");
    }

    /// Compute layout with text measurement support
    ///
    /// This method uses Taffy's measure function to calculate intrinsic sizes
    /// for text nodes, allowing text panels to size themselves based on content.
    pub fn compute_with_text(
        &mut self,
        root: NodeId,
        width: f32,
        height: f32,
        font_atlas: &FontAtlas,
    ) {
        self.taffy
            .compute_layout_with_measure(
                root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
                |known_dimensions, _available_space, _node_id, node_context, _style| {
                    // If dimensions are already known from style, use those
                    let width = known_dimensions.width;
                    let height = known_dimensions.height;

                    // If we have a node context with text content, measure it
                    if let Some(context) = node_context {
                        if let Content::Text {
                            ref text, scale, ..
                        } = context.content
                        {
                            let (text_width, text_height) = font_atlas.measure_text(text, scale);

                            // Use measured dimensions if not already specified
                            return Size {
                                width: width.unwrap_or(text_width),
                                height: height.unwrap_or(text_height),
                            };
                        }
                    }

                    // Default: use known dimensions or zero
                    Size {
                        width: width.unwrap_or(0.0),
                        height: height.unwrap_or(0.0),
                    }
                },
            )
            .expect("Failed to compute layout with measure");
    }

    /// Get the computed layout for a node
    pub fn get_layout(&self, node: NodeId) -> &Layout {
        self.taffy.layout(node).expect("Failed to get layout")
    }

    /// Get the panel style for a node
    pub fn get_panel_style(&self, node: NodeId) -> Option<&PanelStyle> {
        self.taffy.get_node_context(node)
    }

    /// Get children of a node
    pub fn children(&self, node: NodeId) -> Vec<NodeId> {
        self.taffy.children(node).unwrap_or_default()
    }

    /// Find all nodes with a panel_id matching the given prefix and return their absolute bounds
    ///
    /// Returns a Vec of (panel_id, x, y, width, height) tuples
    pub fn find_panels_by_prefix(&self, root: NodeId, prefix: &str) -> Vec<(String, f32, f32, f32, f32)> {
        let mut results = Vec::new();
        self.find_panels_recursive(root, 0.0, 0.0, prefix, &mut results);
        results
    }

    fn find_panels_recursive(
        &self,
        node: NodeId,
        parent_x: f32,
        parent_y: f32,
        prefix: &str,
        results: &mut Vec<(String, f32, f32, f32, f32)>,
    ) {
        let layout = self.get_layout(node);
        let abs_x = parent_x + layout.location.x;
        let abs_y = parent_y + layout.location.y;
        let width = layout.size.width;
        let height = layout.size.height;

        // Check if this node has a matching panel_id
        if let Some(style) = self.get_panel_style(node) {
            if let Some(ref id) = style.panel_id {
                if id.starts_with(prefix) {
                    results.push((id.clone(), abs_x, abs_y, width, height));
                }
            }
        }

        // Recurse into children
        for child in self.children(node) {
            self.find_panels_recursive(child, abs_x, abs_y, prefix, results);
        }
    }
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}
