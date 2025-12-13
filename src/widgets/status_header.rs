//! Status header widget for displaying connection status, provider, and controls
//!
//! Shows: [Overview] [Details] | Provider: Binance | [w] Window: 15m | [c] Chart: Candle | ● Live

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::format::capitalize;
use super::theme::GlTheme;
use crate::app::{ChartType, ConnectionStatus, TimeWindow, View};

/// Build the status header panel
pub fn build_status_header(
    view: View,
    provider: &str,
    time_window: TimeWindow,
    chart_type: ChartType,
    connection_status: ConnectionStatus,
    theme: &GlTheme,
) -> PanelBuilder {
    let gap = theme.panel_gap;
    let header_height = theme.font_size * 3.0; // Derived from font size

    // View tabs
    let (overview_color, details_color) = match view {
        View::Overview => (theme.accent, theme.foreground_inactive),
        View::Details => (theme.foreground_inactive, theme.accent),
    };

    // Connection status
    let (status_text, status_color) = match connection_status {
        ConnectionStatus::Connected => ("● Live", theme.status_live),
        ConnectionStatus::Connecting => ("◐ Connecting", theme.status_connecting),
        ConnectionStatus::Disconnected => ("○ Disconnected", theme.status_disconnected),
        ConnectionStatus::Mock => ("◆ Mock", theme.status_mock),
    };

    let provider_display = capitalize(provider);
    let window_display = time_window.as_str();
    let chart_display = match chart_type {
        ChartType::Line => "Line",
        ChartType::Candlestick => "Candle",
    };

    panel()
        .width(percent(1.0))
        .height(length(header_height))
        .background(theme.background_panel)
        .border_solid(1.0, theme.border)
        .padding_all(theme.panel_padding)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(gap * 2.0)
        // View tabs
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Tab:", theme.foreground_inactive, theme.font_normal))
                .child(panel().text("Overview", overview_color, theme.font_normal))
                .child(panel().text("]", theme.foreground_inactive, theme.font_normal))
                .child(panel().text("[", theme.foreground_inactive, theme.font_normal))
                .child(panel().text("Details", details_color, theme.font_normal))
                .child(panel().text("]", theme.foreground_inactive, theme.font_normal)),
        )
        // Spacer
        .child(panel().flex_grow(1.0))
        // Provider
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("Provider:", theme.foreground_muted, theme.font_normal))
                .child(panel().text(&provider_display, theme.foreground, theme.font_normal)),
        )
        // Time window
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[w]", theme.foreground_muted, theme.font_normal))
                .child(panel().text("Window:", theme.foreground_muted, theme.font_normal))
                .child(panel().text(window_display, theme.accent, theme.font_normal)),
        )
        // Chart type (only in Details view)
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[c]", theme.foreground_muted, theme.font_normal))
                .child(panel().text("Chart:", theme.foreground_muted, theme.font_normal))
                .child(panel().text(chart_display, theme.accent, theme.font_normal)),
        )
        // Connection status
        .child(panel().text(status_text, status_color, theme.font_normal))
}
