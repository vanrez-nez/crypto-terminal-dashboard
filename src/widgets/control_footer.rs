//! Control footer widget displaying keyboard shortcuts and selection info

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::theme::GlTheme;
/// Build the control footer panel for Overview view
pub fn build_overview_footer(
    selected_count: usize,
    total_count: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    let gap = theme.panel_gap;
    let footer_height = theme.font_size * 3.0; // Derived from font size

    panel()
        .width(percent(1.0))
        .height(length(footer_height))
        .background(theme.background_panel)
        .border_solid(1.0, theme.border)
        .padding_all(theme.panel_padding)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(gap * 2.0)
        .child(panel().text(
            &format!("Selected: {}/{}", selected_count, total_count),
            theme.foreground,
            theme.font_normal,
        ))
        .child(panel().text("│", theme.foreground_muted, theme.font_normal))
        // Space toggle
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Space]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Toggle", theme.foreground, theme.font_normal)),
        )
        // Enter for details
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Enter]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("View Details", theme.foreground, theme.font_normal)),
        )
        // Arrow keys
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[▲▼]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Navigate", theme.foreground, theme.font_normal)),
        )
        // Quit
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[q]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Quit", theme.foreground, theme.font_normal)),
        )
}

/// Build the control footer panel for Details view
pub fn build_details_footer(theme: &GlTheme) -> PanelBuilder {
    let gap = theme.panel_gap;
    let footer_height = theme.font_size * 3.0; // Derived from font size

    panel()
        .width(percent(1.0))
        .height(length(footer_height))
        .background(theme.background_panel)
        .border_solid(1.0, theme.border)
        .padding_all(theme.panel_padding)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(gap * 2.0)
        // Chart scroll
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[◄►]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Scroll Chart", theme.foreground, theme.font_normal)),
        )
        // Reset scroll
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Home/r]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Reset", theme.foreground, theme.font_normal)),
        )
        // Window change
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[w]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Window", theme.foreground, theme.font_normal)),
        )
        // Chart type
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[c]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Chart Type", theme.foreground, theme.font_normal)),
        )
        // Quit
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[q]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Quit", theme.foreground, theme.font_normal)),
        )
}
