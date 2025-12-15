//! Status header widget for displaying connection status, provider, and controls
//!
//! Shows: [Overview] [Details] | Provider: Binance | [w] Window: 15m | [c] Chart: Candle | ● Live

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::format::capitalize;
use super::theme::GlTheme;
use crate::app::{ChartType, ConnectionStatus, TimeWindow, View};
use crate::base::view::header_height;

/// Build the status header panel
pub fn build_status_header(
    view: View,
    provider: &str,
    time_window: TimeWindow,
    chart_type: ChartType,
    connection_status: ConnectionStatus,
    unread_count: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    let gap = theme.panel_gap;
    let header_height = header_height(theme); // Derived from theme sizing

    // View tabs
    let (overview_color, details_color, alerts_color, news_color) = match view {
        View::Overview => (
            theme.accent,
            theme.foreground_inactive,
            theme.foreground_inactive,
            theme.foreground_inactive,
        ),
        View::Details => (
            theme.foreground_inactive,
            theme.accent,
            theme.foreground_inactive,
            theme.foreground_inactive,
        ),
        View::Notifications => (
            theme.foreground_inactive,
            theme.foreground_inactive,
            theme.accent,
            theme.foreground_inactive,
        ),
        View::News => (
            theme.foreground_inactive,
            theme.foreground_inactive,
            theme.foreground_inactive,
            theme.accent,
        ),
    };

    // Connection status
    let (status_text, status_color) = match connection_status {
        ConnectionStatus::Connected => ("● Live", theme.status_live),
        ConnectionStatus::Connecting => ("◐ Connecting", theme.status_connecting),
        ConnectionStatus::Disconnected => ("○ Disconnected", theme.status_disconnected),
        ConnectionStatus::Mock => ("◆ Mock", theme.status_mock),
    };

    let provider_display = capitalize(provider);

    // Suppress unused warnings - these are now shown in the footer
    let _ = time_window;
    let _ = chart_type;

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
                .child(panel().text("[Overview]", overview_color, theme.font_normal))
                .child(panel().text("[Details]", details_color, theme.font_normal))
                .child(build_alerts_tab(alerts_color, unread_count, theme))
                .child(panel().text("[News]", news_color, theme.font_normal)),
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
        // Connection status
        .child(panel().text(status_text, status_color, theme.font_normal))
        // Quit
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[q]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Quit", theme.foreground, theme.font_normal)),
        )
}

/// Build the Alerts tab with optional unread badge
fn build_alerts_tab(color: [f32; 4], unread_count: usize, theme: &GlTheme) -> PanelBuilder {
    let gap = theme.panel_gap;

    if unread_count > 0 {
        panel()
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .gap(gap / 2.0)
            .child(panel().text("[Alerts", color, theme.font_normal))
            .child(
                panel()
                    .background(theme.negative)
                    .padding(1.0, 4.0, 1.0, 4.0)
                    .child(panel().text(
                        &format!("{}", unread_count),
                        theme.foreground,
                        theme.font_small,
                    )),
            )
            .child(panel().text("]", color, theme.font_normal))
    } else {
        panel().text("[Alerts]", color, theme.font_normal)
    }
}
