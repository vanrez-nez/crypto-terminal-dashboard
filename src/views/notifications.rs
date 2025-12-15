//! Notifications view - alert rules and history log

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use crate::app::App;
use crate::notifications::{NotificationRule, Severity};
use crate::widgets::{status_header::build_status_header, theme::GlTheme, titled_panel::titled_panel};

/// Build the notifications view
pub fn build_notifications_view(app: &App, theme: &GlTheme, width: f32, height: f32) -> PanelBuilder {
    let gap = theme.panel_gap;

    panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
        .gap(gap)
        .padding_all(gap)
        .background(theme.background)
        // Header
        .child(build_status_header(
            app.view,
            &app.provider,
            app.time_window,
            app.chart_type,
            app.connection_status,
            app.notification_manager.unread_count,
            theme,
        ))
        // Main content: two columns
        .child(
            panel()
                .flex_grow(1.0)
                .flex_direction(FlexDirection::Row)
                .gap(gap)
                // Left column: Rule toggles (35%)
                .child(
                    titled_panel(
                        "Alert Rules",
                        theme,
                        build_rules_list(
                            app.notification_manager.get_rules(),
                            app.notification_manager.selected_rule,
                            theme,
                        ),
                    )
                    .width(percent(0.35)),
                )
                // Right column: Notification log (65%)
                .child(
                    titled_panel(
                        "Alert History",
                        theme,
                        build_notification_list(app, theme),
                    )
                    .flex_grow(1.0),
                ),
        )
        // Footer with controls
        .child(build_notifications_footer(theme))
}

/// Build the list of notification rules with toggle checkboxes
fn build_rules_list(
    rules: &[NotificationRule],
    selected: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    let gap = theme.panel_gap;

    let mut container = panel()
        .flex_direction(FlexDirection::Column)
        .gap(gap / 2.0)
        .padding_all(gap / 2.0);

    if rules.is_empty() {
        container = container.child(
            panel().text("No rules configured", theme.foreground_muted, theme.font_normal),
        );
    } else {
        for (i, rule) in rules.iter().enumerate() {
            let is_selected = i == selected;
            let checkbox = if rule.is_enabled() { "[x]" } else { "[ ]" };
            let description = rule.description();

            let bg_color = if is_selected {
                theme.selection_bg
            } else {
                theme.background_panel
            };

            let text_color = if rule.is_enabled() {
                theme.foreground
            } else {
                theme.foreground_muted
            };

            container = container.child(
                panel()
                    .flex_direction(FlexDirection::Row)
                    .gap(gap / 2.0)
                    .padding(gap / 4.0, gap / 2.0, gap / 4.0, gap / 2.0)
                    .background(bg_color)
                    .child(panel().text(checkbox, theme.accent, theme.font_normal))
                    .child(panel().text(&description, text_color, theme.font_normal)),
            );
        }
    }

    container
}

/// Build the notification history list
fn build_notification_list(app: &App, theme: &GlTheme) -> PanelBuilder {
    let gap = theme.panel_gap;
    let notifications = app.notification_manager.get_notifications();
    let scroll_offset = app.notification_scroll;

    let mut container = panel()
        .flex_direction(FlexDirection::Column)
        .gap(gap / 4.0)
        .padding_all(gap / 2.0);

    if notifications.is_empty() {
        container = container.child(
            panel().text("No alerts yet", theme.foreground_muted, theme.font_normal),
        );
    } else {
        // Show most recent first, apply scroll offset
        let visible_count = 15; // Show up to 15 notifications
        let start = scroll_offset;
        let end = (start + visible_count).min(notifications.len());

        // Iterate in reverse (newest first)
        for notif in notifications.iter().rev().skip(start).take(end - start) {
            let severity_color = match notif.severity {
                Severity::Info => theme.foreground_muted,
                Severity::Warning => theme.accent,
                Severity::Critical => theme.negative,
            };

            let read_indicator = if notif.read { " " } else { "*" };
            let time_str = notif.time_str();

            container = container.child(
                panel()
                    .flex_direction(FlexDirection::Row)
                    .gap(gap / 2.0)
                    .child(panel().text(read_indicator, theme.negative, theme.font_small))
                    .child(panel().text(&time_str, theme.foreground_muted, theme.font_small))
                    .child(panel().text(&notif.message, severity_color, theme.font_small)),
            );
        }

        // Show scroll indicator if there are more
        if notifications.len() > visible_count {
            let shown = end - start;
            let total = notifications.len();
            container = container.child(
                panel().text(
                    &format!("({}/{} alerts)", shown, total),
                    theme.foreground_muted,
                    theme.font_small,
                ),
            );
        }
    }

    container
}

/// Build the footer with keyboard controls
fn build_notifications_footer(theme: &GlTheme) -> PanelBuilder {
    let gap = theme.panel_gap;

    panel()
        .flex_direction(FlexDirection::Row)
        .gap(gap * 2.0)
        .padding(gap / 2.0, gap, gap / 2.0, gap)
        .background(theme.background_panel)
        .border_solid(1.0, theme.border)
        // View switch
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Tab]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Switch view", theme.foreground, theme.font_normal)),
        )
        // Toggle rule
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[Space]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Toggle rule", theme.foreground, theme.font_normal)),
        )
        // Scroll
        .child(
            panel()
                .flex_direction(FlexDirection::Row)
                .gap(gap / 2.0)
                .child(panel().text("[j/k]", theme.accent_secondary, theme.font_normal))
                .child(panel().text("Navigate", theme.foreground, theme.font_normal)),
        )
}
