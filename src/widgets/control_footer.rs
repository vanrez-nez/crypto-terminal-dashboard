//! Control footer widget displaying keyboard shortcuts and selection info

use crate::app::{ChartType, TimeWindow};
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
pub fn build_details_footer(
    time_window: TimeWindow,
    chart_type: ChartType,
    ticker_muted: bool,
    theme: &GlTheme,
) -> PanelBuilder {
    let gap = theme.panel_gap;
    let footer_height = theme.font_size * 3.0; // Derived from font size

    let window_display = time_window.as_str();
    let chart_display = match chart_type {
        ChartType::Polygonal => "Poly",
        ChartType::Candlestick => "Candle",
    };
    let mute_display = if ticker_muted { "Muted" } else { "On" };
    let mute_color = if ticker_muted {
        theme.foreground_muted
    } else {
        theme.accent
    };

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
        // Zoom
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[▲▼]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Zoom", theme.foreground, theme.font_normal)),
        )
        // Window change with current value
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[w]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Window:", theme.foreground_muted, theme.font_normal))
                .child(panel().text(window_display, theme.accent, theme.font_normal)),
        )
        // Chart type with current value
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[c]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Chart:", theme.foreground_muted, theme.font_normal))
                .child(panel().text(chart_display, theme.accent, theme.font_normal)),
        )
        // Mute toggle with current state
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[m]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Sound:", theme.foreground_muted, theme.font_normal))
                .child(panel().text(mute_display, mute_color, theme.font_normal)),
        )
}
