use crate::base::layout::style::{Border, Content, HAlign, PanelStyle, VAlign};
use crate::base::layout::tree::LayoutTree;
use taffy::prelude::*;
use taffy::{Overflow, Point as TaffyPoint};

/// Builder for creating panels with a fluent API
pub struct PanelBuilder {
    taffy_style: Style,
    panel_style: PanelStyle,
    children: Vec<PanelBuilder>,
}

impl PanelBuilder {
    pub fn new() -> Self {
        Self {
            taffy_style: Style::default(),
            panel_style: PanelStyle::default(),
            children: Vec::new(),
        }
    }

    // === Layout properties (Taffy) ===

    /// Set fixed size in pixels
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.taffy_style.size = Size {
            width: length(width),
            height: length(height),
        };
        self
    }

    /// Set width only
    pub fn width(mut self, width: Dimension) -> Self {
        self.taffy_style.size.width = width;
        self
    }

    /// Set height only
    pub fn height(mut self, height: Dimension) -> Self {
        self.taffy_style.size.height = height;
        self
    }

    /// Set size using Taffy Size
    #[allow(dead_code)]
    pub fn size_taffy(mut self, size: Size<Dimension>) -> Self {
        self.taffy_style.size = size;
        self
    }

    /// Set minimum size
    #[allow(dead_code)]
    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.taffy_style.min_size = Size {
            width: length(width),
            height: length(height),
        };
        self
    }

    /// Set maximum size
    #[allow(dead_code)]
    pub fn max_size(mut self, width: f32, height: f32) -> Self {
        self.taffy_style.max_size = Size {
            width: length(width),
            height: length(height),
        };
        self
    }

    /// Set flex direction
    pub fn flex_direction(mut self, dir: FlexDirection) -> Self {
        self.taffy_style.flex_direction = dir;
        self
    }

    /// Set flex wrap
    #[allow(dead_code)]
    pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.taffy_style.flex_wrap = wrap;
        self
    }

    /// Set flex grow
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.taffy_style.flex_grow = grow;
        self
    }

    /// Set flex shrink
    #[allow(dead_code)]
    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.taffy_style.flex_shrink = shrink;
        self
    }

    /// Set flex basis
    #[allow(dead_code)]
    pub fn flex_basis(mut self, basis: Dimension) -> Self {
        self.taffy_style.flex_basis = basis;
        self
    }

    /// Set proportional size using flex-grow ratio
    ///
    /// DESIGN: This is the correct way to do proportional layouts that account for gaps.
    /// Unlike flex_basis(percent()), which calculates percentage of container size (ignoring gaps),
    /// proportion() uses flex-grow to distribute remaining space AFTER gaps are subtracted.
    ///
    /// Example: Two panels with proportion(60) and proportion(40) will split available space
    /// 60:40 regardless of gap size or number of siblings.
    ///
    /// For 5 equal panels: each gets proportion(1) or proportion(20)
    /// For 70/30 split: proportion(70) and proportion(30)
    pub fn proportion(mut self, ratio: f32) -> Self {
        self.taffy_style.flex_basis = Dimension::Length(0.0);
        self.taffy_style.flex_grow = ratio;
        self.taffy_style.flex_shrink = 0.0;
        self
    }

    /// Set justify content (main axis alignment)
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.taffy_style.justify_content = Some(justify);
        self
    }

    /// Set align items (cross axis alignment)
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.taffy_style.align_items = Some(align);
        self
    }

    /// Set align self
    #[allow(dead_code)]
    pub fn align_self(mut self, align: AlignSelf) -> Self {
        self.taffy_style.align_self = Some(align);
        self
    }

    /// Set align content
    #[allow(dead_code)]
    pub fn align_content(mut self, align: AlignContent) -> Self {
        self.taffy_style.align_content = Some(align);
        self
    }

    /// Set padding (all sides)
    pub fn padding_all(mut self, value: f32) -> Self {
        self.taffy_style.padding = Rect {
            left: length(value),
            right: length(value),
            top: length(value),
            bottom: length(value),
        };
        self
    }

    /// Set padding (individual sides)
    pub fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.taffy_style.padding = Rect {
            left: length(left),
            right: length(right),
            top: length(top),
            bottom: length(bottom),
        };
        self
    }

    /// Set margin (all sides)
    #[allow(dead_code)]
    pub fn margin_all(mut self, value: f32) -> Self {
        self.taffy_style.margin = Rect {
            left: length(value),
            right: length(value),
            top: length(value),
            bottom: length(value),
        };
        self
    }

    /// Set margin (individual sides)
    #[allow(dead_code)]
    pub fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.taffy_style.margin = Rect {
            left: length(left),
            right: length(right),
            top: length(top),
            bottom: length(bottom),
        };
        self
    }

    /// Set gap between children
    pub fn gap(mut self, value: f32) -> Self {
        self.taffy_style.gap = Size {
            width: length(value),
            height: length(value),
        };
        self
    }

    /// Set row gap
    #[allow(dead_code)]
    pub fn row_gap(mut self, value: f32) -> Self {
        self.taffy_style.gap.height = length(value);
        self
    }

    /// Set column gap
    #[allow(dead_code)]
    pub fn column_gap(mut self, value: f32) -> Self {
        self.taffy_style.gap.width = length(value);
        self
    }

    /// Set position type (relative or absolute)
    #[allow(dead_code)]
    pub fn position(mut self, position: Position) -> Self {
        self.taffy_style.position = position;
        self
    }

    /// Set inset (for absolute positioning)
    #[allow(dead_code)]
    pub fn inset(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.taffy_style.inset = Rect {
            left: length(left),
            right: length(right),
            top: length(top),
            bottom: length(bottom),
        };
        self
    }

    /// Set absolute position (shorthand for position + inset)
    pub fn absolute(mut self, left: f32, top: f32) -> Self {
        self.taffy_style.position = Position::Absolute;
        self.taffy_style.inset = Rect {
            left: length(left),
            right: LengthPercentageAuto::Auto,
            top: length(top),
            bottom: LengthPercentageAuto::Auto,
        };
        self
    }

    /// Set overflow behavior (affects how children contribute to intrinsic size)
    /// Overflow::Scroll tells Taffy that children can overflow without expanding the container
    pub fn overflow(mut self, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        self.taffy_style.overflow = TaffyPoint {
            x: overflow_x,
            y: overflow_y,
        };
        self
    }

    /// Set overflow to scroll (children don't expand container)
    /// DESIGN: Use this on scrollable containers to prevent child content from affecting size
    pub fn overflow_scroll(mut self) -> Self {
        self.taffy_style.overflow = TaffyPoint {
            x: Overflow::Scroll,
            y: Overflow::Scroll,
        };
        self
    }

    // === Visual properties (PanelStyle) ===

    /// Set background color
    pub fn background(mut self, color: [f32; 4]) -> Self {
        self.panel_style.background_color = Some(color);
        self
    }

    /// Set border
    #[allow(dead_code)]
    pub fn border(mut self, border: Border) -> Self {
        self.panel_style.border = border;
        self
    }

    /// Set solid border
    pub fn border_solid(mut self, width: f32, color: [f32; 4]) -> Self {
        self.panel_style.border = Border::solid(width, color);
        self
    }

    /// Set dashed border
    #[allow(dead_code)]
    pub fn border_dashed(mut self, width: f32, color: [f32; 4]) -> Self {
        self.panel_style.border = Border::dashed(width, color);
        self
    }

    /// Set dotted border
    #[allow(dead_code)]
    pub fn border_dotted(mut self, width: f32, color: [f32; 4]) -> Self {
        self.panel_style.border = Border::dotted(width, color);
        self
    }

    /// Set text content
    pub fn text(mut self, text: impl Into<String>, color: [f32; 4], scale: f32) -> Self {
        self.panel_style.content = Content::Text {
            text: text.into(),
            color,
            scale,
        };
        self
    }

    /// Enable overflow clipping
    pub fn clip(mut self, clip: bool) -> Self {
        self.panel_style.clip_overflow = clip;
        self
    }

    /// Set text horizontal alignment
    #[allow(dead_code)]
    pub fn text_align_h(mut self, align: HAlign) -> Self {
        self.panel_style.text_align_h = align;
        self
    }

    /// Set text vertical alignment
    #[allow(dead_code)]
    pub fn text_align_v(mut self, align: VAlign) -> Self {
        self.panel_style.text_align_v = align;
        self
    }

    /// Set text alignment (horizontal and vertical)
    pub fn text_align(mut self, h: HAlign, v: VAlign) -> Self {
        self.panel_style.text_align_h = h;
        self.panel_style.text_align_v = v;
        self
    }

    // === Focus properties ===

    /// Make this panel focusable
    pub fn focusable(mut self, id: impl Into<String>) -> Self {
        self.panel_style.focusable = true;
        self.panel_style.panel_id = Some(id.into());
        self
    }

    /// Set a marker ID for this panel (without making it focusable)
    /// Used to identify special panels like chart areas after layout
    pub fn marker_id(mut self, id: impl Into<String>) -> Self {
        self.panel_style.panel_id = Some(id.into());
        self
    }

    /// Set border color when focused
    pub fn focus_border(mut self, color: [f32; 4]) -> Self {
        self.panel_style.focus_border_color = Some(color);
        self
    }

    /// Make this panel scrollable
    pub fn scrollable(mut self) -> Self {
        self.panel_style.scrollable = true;
        self.panel_style.clip_overflow = true;
        self
    }

    /// Set scroll offset
    pub fn scroll_offset(mut self, offset: f32) -> Self {
        self.panel_style.scroll_offset = offset;
        self
    }

    // === Children ===

    /// Add a child panel
    pub fn child(mut self, child: PanelBuilder) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: impl IntoIterator<Item = PanelBuilder>) -> Self {
        self.children.extend(children);
        self
    }

    // === Build ===

    /// Build this panel and all its children into the layout tree
    pub fn build(self, tree: &mut LayoutTree) -> NodeId {
        // First, build all children
        let child_ids: Vec<NodeId> = self
            .children
            .into_iter()
            .map(|child| child.build(tree))
            .collect();

        // Then create this node
        if child_ids.is_empty() {
            tree.new_leaf(self.taffy_style, self.panel_style)
        } else {
            tree.new_with_children(self.taffy_style, self.panel_style, &child_ids)
        }
    }
}

impl Default for PanelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience function for creating a new panel
pub fn panel() -> PanelBuilder {
    PanelBuilder::new()
}
