//! Positions view - display margin account positions

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use crate::api::margin::MarginAccount;
use crate::app::App;
use crate::base::view::ViewSpacing;
use crate::widgets::{
    control_footer::build_positions_footer, format::format_price,
    positions_table::build_positions_table, status_header::build_status_header,
    theme::GlTheme, titled_panel::titled_panel,
};

pub fn build_positions_view(
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> PanelBuilder {
    let spacing = ViewSpacing::new(theme);

    // Build content based on state
    let content = if !app.positions_available {
        build_unavailable_state(theme)
    } else if app.positions_loading {
        build_loading_state(theme)
    } else if let Some(account) = &app.margin_account {
        build_positions_content(account, app.positions_selected, theme)
    } else {
        build_empty_state(theme)
    };

    panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
        .gap(spacing.section_gap)
        .padding_all(spacing.outer_padding)
        .background(theme.background)
        // Header - fixed height
        .child(build_status_header(
            app.view,
            &app.provider,
            app.time_window,
            app.chart_type,
            app.connection_status,
            app.notification_manager.unread_count,
            theme,
        ))
        // Content - grows to fill space
        .child(content.flex_grow(1.0))
        // Footer - fixed height
        .child(
            build_positions_footer(
                app.margin_account.as_ref().map(|a| a.margin_level),
                theme,
            )
            .margin(spacing.footer_margin(), 0.0, 0.0, 0.0),
        )
}

fn build_positions_content(
    account: &MarginAccount,
    selected_index: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    // Account summary panel
    let summary = build_account_summary(account, theme);

    // Positions table (scrollable)
    let table = titled_panel(
        &format!("Active Positions ({})", account.positions.len()),
        theme,
        panel().flex_grow(1.0).child(build_positions_table(
            &account.positions,
            selected_index,
            theme,
        )),
    )
    .flex_grow(1.0);

    panel()
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Column)
        .gap(theme.panel_gap)
        .child(summary)
        .child(table)
}

fn build_account_summary(account: &MarginAccount, theme: &GlTheme) -> PanelBuilder {
    // Color-code margin level (green > 2.0, yellow > 1.5, red < 1.5)
    let margin_color = if account.margin_level > 2.0 {
        theme.positive
    } else if account.margin_level > 1.5 {
        theme.neutral
    } else {
        theme.negative
    };

    // Color for net equity
    let net_color = if account.total_net_usd > 0.0 {
        theme.positive
    } else if account.total_net_usd < 0.0 {
        theme.negative
    } else {
        theme.foreground
    };

    titled_panel(
        &format!("{} - Account Summary", account.account_type),
        theme,
        panel()
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::SpaceAround)
            .gap(theme.panel_gap * 2.0)
            .padding_all(theme.panel_gap)
            .child(
                panel()
                    .flex_direction(FlexDirection::Column)
                    .gap(theme.panel_gap / 2.0)
                    .child(
                        panel().text(
                            "Margin Level",
                            theme.accent_secondary,
                            theme.font_small,
                        ),
                    )
                    .child(
                        panel().text(
                            &format!("{:.2}", account.margin_level),
                            margin_color,
                            theme.font_big,
                        ),
                    ),
            )
            .child(
                panel()
                    .flex_direction(FlexDirection::Column)
                    .gap(theme.panel_gap / 2.0)
                    .child(
                        panel().text("Total Assets", theme.accent_secondary, theme.font_small),
                    )
                    .child(panel().text(
                        &format_price(account.total_asset_usd),
                        theme.foreground,
                        theme.font_big,
                    )),
            )
            .child(
                panel()
                    .flex_direction(FlexDirection::Column)
                    .gap(theme.panel_gap / 2.0)
                    .child(
                        panel().text(
                            "Total Borrowed",
                            theme.accent_secondary,
                            theme.font_small,
                        ),
                    )
                    .child(panel().text(
                        &format_price(account.total_liability_usd),
                        theme.negative,
                        theme.font_big,
                    )),
            )
            .child(
                panel()
                    .flex_direction(FlexDirection::Column)
                    .gap(theme.panel_gap / 2.0)
                    .child(
                        panel().text(
                            "Net Equity",
                            theme.accent_secondary,
                            theme.font_small,
                        ),
                    )
                    .child(panel().text(
                        &format_price(account.total_net_usd),
                        net_color,
                        theme.font_big,
                    )),
            ),
    )
}

fn build_loading_state(theme: &GlTheme) -> PanelBuilder {
    titled_panel(
        "Positions",
        theme,
        panel()
            .flex_grow(1.0)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .text(
                "Loading positions...",
                theme.accent_secondary,
                theme.font_normal,
            ),
    )
}

fn build_empty_state(theme: &GlTheme) -> PanelBuilder {
    titled_panel(
        "Positions",
        theme,
        panel()
            .flex_grow(1.0)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .text(
                "No positions data. Press 'r' to refresh.",
                theme.accent_secondary,
                theme.font_normal,
            ),
    )
}

fn build_unavailable_state(theme: &GlTheme) -> PanelBuilder {
    titled_panel(
        "Positions",
        theme,
        panel()
            .flex_grow(1.0)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .text(
                "Positions unavailable - API keys not configured",
                theme.foreground_muted,
                theme.font_normal,
            ),
    )
}
