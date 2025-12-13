use taffy::NodeId;

use crate::base::focus::FocusManager;
use crate::base::font_atlas::FontAtlas;
use crate::base::layout::{BorderStyle, Content, HAlign, LayoutTree, VAlign};
use crate::base::renderer::rect_renderer::{Rect, RectRenderer};
use crate::base::renderer::scissor_stack::ScissorStack;
use crate::base::text_renderer::TextRenderer;

/// Renders the layout tree to the screen
pub fn render(
    gl: &glow::Context,
    tree: &LayoutTree,
    root: NodeId,
    rect_renderer: &mut RectRenderer,
    text_renderer: &mut TextRenderer,
    font_atlas: &FontAtlas,
    scissor_stack: &mut ScissorStack,
    focus_manager: &FocusManager,
    screen_width: u32,
    screen_height: u32,
) {
    // Begin batching
    rect_renderer.begin();
    text_renderer.begin();

    // Render the tree recursively
    render_node(
        gl,
        tree,
        root,
        0.0,
        0.0,
        rect_renderer,
        text_renderer,
        font_atlas,
        scissor_stack,
        focus_manager,
        screen_width,
        screen_height,
    );

    // Flush remaining batches
    rect_renderer.end(gl, screen_width, screen_height);
    text_renderer.end(gl, font_atlas, screen_width, screen_height);

    // Clear scissor state
    scissor_stack.clear(gl);
}

fn render_node(
    gl: &glow::Context,
    tree: &LayoutTree,
    node: NodeId,
    parent_x: f32,
    parent_y: f32,
    rect_renderer: &mut RectRenderer,
    text_renderer: &mut TextRenderer,
    font_atlas: &FontAtlas,
    scissor_stack: &mut ScissorStack,
    focus_manager: &FocusManager,
    screen_width: u32,
    screen_height: u32,
) {
    let layout = tree.get_layout(node);
    let panel_style = tree.get_panel_style(node);

    // Calculate absolute position
    let abs_x = parent_x + layout.location.x;
    let abs_y = parent_y + layout.location.y;
    let width = layout.size.width;
    let height = layout.size.height;

    let bounds = Rect::new(abs_x, abs_y, width, height);

    if let Some(style) = panel_style {
        // 1. Draw background
        if let Some(bg_color) = style.background_color {
            rect_renderer.draw_rect(&bounds, bg_color);
        }

        // 2. Draw border (with focus highlight if focused)
        let is_focused = style
            .panel_id
            .as_ref()
            .map(|id| focus_manager.is_focused(id))
            .unwrap_or(false);

        let border_color = if is_focused {
            style.focus_border_color.unwrap_or([1.0, 0.8, 0.2, 1.0]) // Yellow focus
        } else {
            style.border.color
        };

        let border_width = if is_focused {
            style.border.width.max(2.0)
        } else {
            style.border.width
        };

        match style.border.style {
            BorderStyle::None => {
                // If focused but no border, draw focus indicator anyway
                if is_focused {
                    rect_renderer.draw_border_solid(&bounds, 2.0, border_color);
                }
            }
            BorderStyle::Solid => {
                rect_renderer.draw_border_solid(&bounds, border_width, border_color);
            }
            BorderStyle::Dashed => {
                rect_renderer.draw_border_dashed(&bounds, border_width, border_color);
            }
            BorderStyle::Dotted => {
                rect_renderer.draw_border_dotted(&bounds, border_width, border_color);
            }
        }

        // 3. Handle clipping
        if style.clip_overflow {
            // Flush current batches before changing scissor
            rect_renderer.end(gl, screen_width, screen_height);
            text_renderer.end(gl, font_atlas, screen_width, screen_height);
            rect_renderer.begin();
            text_renderer.begin();

            // Calculate content area (inside padding)
            let padding = &layout.padding;
            let content_rect = Rect::new(
                abs_x + padding.left,
                abs_y + padding.top,
                width - padding.left - padding.right,
                height - padding.top - padding.bottom,
            );
            scissor_stack.push(gl, content_rect);
        }

        // 4. Draw text content (with scroll offset if scrollable)
        if let Content::Text {
            ref text,
            color,
            scale,
        } = style.content
        {
            // Calculate content area for text positioning
            let padding = &layout.padding;
            let content_x = abs_x + padding.left;
            let content_y = abs_y + padding.top;
            let content_width = width - padding.left - padding.right;
            let content_height = height - padding.top - padding.bottom;

            // Measure text for alignment
            let (text_width, text_height) = text_renderer.measure_text(font_atlas, text, scale);

            // Calculate X position based on horizontal alignment
            let text_x = match style.text_align_h {
                HAlign::Left => content_x,
                HAlign::Center => content_x + (content_width - text_width) / 2.0,
                HAlign::Right => content_x + content_width - text_width,
            };

            // Calculate Y position based on vertical alignment
            // base_y is the baseline position - glyphs extend upward by text_height
            // text_height = ascent (height above baseline)
            // For most text, descent is minimal, so we center based on ascent
            let base_y = match style.text_align_v {
                VAlign::Top => content_y + text_height,
                VAlign::Center => {
                    // Center the visual text in the content area
                    // baseline = content_center + text_height/2 positions text visually centered
                    content_y + (content_height / 2.0) + (text_height / 2.0)
                }
                VAlign::Bottom => content_y + content_height,
            };

            let text_y = base_y - style.scroll_offset;

            text_renderer.draw_text(font_atlas, text, text_x, text_y, scale, color);
        }
    }

    // 5. Render children (with scroll offset if scrollable)
    let children = tree.children(node);
    let scroll_offset = panel_style.map(|s| s.scroll_offset).unwrap_or(0.0);

    for child in children {
        render_node(
            gl,
            tree,
            child,
            abs_x,
            abs_y - scroll_offset, // Apply scroll offset to children
            rect_renderer,
            text_renderer,
            font_atlas,
            scissor_stack,
            focus_manager,
            screen_width,
            screen_height,
        );
    }

    // 6. Pop scissor if we pushed it
    if let Some(style) = panel_style {
        if style.clip_overflow {
            // Flush before restoring scissor
            rect_renderer.end(gl, screen_width, screen_height);
            text_renderer.end(gl, font_atlas, screen_width, screen_height);
            rect_renderer.begin();
            text_renderer.begin();

            scissor_stack.pop(gl);
        }
    }
}
