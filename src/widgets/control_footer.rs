//! Control footer widget displaying keyboard shortcuts and selection info

use dashboard_system::{panel, taffy, HAlign, PanelBuilder, VAlign};
use taffy::prelude::*;

use super::theme::GlTheme;
use crate::app::View;

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
        // Selection count
        .child(panel().text(
            &format!("Selected: {}/{}", selected_count, total_count),
            theme.foreground,
            1.0,
        ))
        // Separator
        .child(panel().text("│", theme.foreground_muted, 1.0))
        // Space toggle
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Space]", theme.accent_secondary, 1.0))
                .child(panel().text("Toggle", theme.foreground, 1.0)),
        )
        // Enter for details
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Enter]", theme.accent_secondary, 1.0))
                .child(panel().text("View Details", theme.foreground, 1.0)),
        )
        // Arrow keys
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[↑↓]", theme.accent_secondary, 1.0))
                .child(panel().text("Navigate", theme.foreground, 1.0)),
        )
        // Quit
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[q]", theme.accent_secondary, 1.0))
                .child(panel().text("Quit", theme.foreground, 1.0)),
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
        // Tab to switch view
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Tab]", theme.accent_secondary, 1.0))
                .child(panel().text("Overview", theme.foreground, 1.0)),
        )
        // Separator
        .child(panel().text("│", theme.foreground_muted, 1.0))
        // Chart scroll
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[←→]", theme.accent_secondary, 1.0))
                .child(panel().text("Scroll Chart", theme.foreground, 1.0)),
        )
        // Reset scroll
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Home/r]", theme.accent_secondary, 1.0))
                .child(panel().text("Reset", theme.foreground, 1.0)),
        )
        // Window change
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[w]", theme.accent_secondary, 1.0))
                .child(panel().text("Window", theme.foreground, 1.0)),
        )
        // Chart type
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[c]", theme.accent_secondary, 1.0))
                .child(panel().text("Chart Type", theme.foreground, 1.0)),
        )
        // Quit
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[q]", theme.accent_secondary, 1.0))
                .child(panel().text("Quit", theme.foreground, 1.0)),
        )
}

/// Build footer based on current view
pub fn build_control_footer(
    view: View,
    selected_count: usize,
    total_count: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    match view {
        View::Overview => build_overview_footer(selected_count, total_count, theme),
        View::Details => build_details_footer(theme),
    }
}
