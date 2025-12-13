//! Titled panel widget - creates panels with a title on the border (like HTML fieldset legend)

use crate::base::layout::{HAlign, VAlign};
use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::theme::GlTheme;

/// Build a titled panel with the title sitting on the top border
///
/// The title is positioned at the top-left corner, centered vertically on the border line.
/// It has a background that masks the border behind it.
///
/// Structure:
/// - Outer container
///   - Title (naturally sized, with background to mask border)
///   - Content panel with border, pulled up by half title height so border aligns with title center
pub fn titled_panel(title: &str, theme: &GlTheme, content: PanelBuilder) -> PanelBuilder {
    // Calculate offset to align border with title center
    // Title text uses the theme's small font scale
    let text_scale = theme.font_small;
    let font_height = theme.font_size * text_scale;
    let title_padding_v = 2.0; // Vertical padding for title background
                               // Border should align with center of title text
    let title_center_offset = title_padding_v + font_height / 2.0;
    let title_left = theme.panel_padding + 4.0;

    panel()
        .flex_direction(FlexDirection::Column)
        // Content panel with border first
        .child(
            panel()
                .flex_grow(1.0)
                .background(theme.background_panel)
                .border_solid(1.0, theme.border)
                // Extra top padding to account for title overlap
                .padding(
                    theme.panel_padding + title_center_offset,
                    theme.panel_padding,
                    theme.panel_padding,
                    theme.panel_padding,
                )
                .margin(title_center_offset, 0.0, 0.0, 0.0) // Top margin for title area
                .flex_direction(FlexDirection::Column)
                .child(content),
        )
        // Title - absolute positioned, renders ON TOP of border
        .child(
            panel()
                .absolute(title_left, 0.0)
                .background(theme.background) // Theme background masks border
                .padding(title_padding_v, 6.0, title_padding_v, 6.0)
                .text(&title.to_uppercase(), theme.accent, text_scale)
                .text_align(HAlign::Left, VAlign::Center),
        )
}
