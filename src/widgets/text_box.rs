//! Text box widget with wrapping and scrolling handled at render time

use super::theme::GlTheme;
use crate::base::layout::{Content, HAlign, VAlign};
use crate::base::{panel, PanelBuilder};
use fontdue::Font;
use std::sync::OnceLock;

/// Build a scrollable text box panel. Wrapping and line fitting are handled in the renderer
/// based on the panel's computed size, so callers don't need to pass dimensions.
pub fn build_text_box(text: &str, scroll_offset: usize, theme: &GlTheme) -> PanelBuilder {
    use taffy::prelude::*;

    let line_gap = 2.0;

    // Handle empty text
    if text.is_empty() {
        return panel()
            .flex_direction(FlexDirection::Column)
            .child(panel().text(
                "No content available",
                theme.foreground_muted,
                theme.font_normal,
            ));
    }

    panel()
        .flex_direction(FlexDirection::Column)
        .gap(line_gap)
        .align_items(AlignItems::FlexStart)
        .width(percent(1.0))
        .flex_grow(1.0)
        .clip(true)
        .content(Content::WrappedTextBox {
            text: text.to_string(),
            color: theme.foreground,
            // Use the same scale multipliers used elsewhere; atlas font size sets the base size.
            scale: theme.font_normal,
            scroll_offset,
            line_gap,
            indicator_color: theme.foreground_muted,
            indicator_scale: theme.font_small,
        })
        .text_align(HAlign::Left, VAlign::Top)
}

/// Get the advance width for a single character at the given pixel size
pub fn char_width_px(ch: char, font_size_px: f32) -> f32 {
    let metrics = font().metrics(ch, font_size_px);
    metrics.advance_width.max(metrics.width as f32)
}

fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(
            include_bytes!("../../fonts/CascadiaMonoPL.ttf") as &[u8],
            Default::default(),
        )
        .expect("failed to load bundled font")
    })
}
