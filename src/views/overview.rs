//! Overview view - coin table with selection

use dashboard_system::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use crate::app::App;
use crate::widgets::{
    build_coin_table, build_overview_footer, build_status_header, titled_panel, GlTheme,
};

/// Centralizes spacing for the overview layout so tweaks happen in one place.
#[derive(Clone, Copy)]
struct OverviewSpacing {
    outer_padding: f32,
    section_gap: f32,
    footer_gap: f32,
}

impl OverviewSpacing {
    fn new(theme: &GlTheme) -> Self {
        let base = theme.panel_gap;
        Self {
            outer_padding: base,
            section_gap: base,
            footer_gap: base * 2.0, // Provide extra breathing room before the footer
        }
    }

    fn footer_margin(&self) -> f32 {
        (self.footer_gap - self.section_gap).max(0.0)
    }
}

pub fn build_overview_view(app: &App, theme: &GlTheme, width: f32, height: f32) -> PanelBuilder {
    let selected_count = app.checked.iter().filter(|&&c| c).count();
    let total_count = app.coins.len();
    let spacing = OverviewSpacing::new(theme);

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
            theme,
        ))
        // Coin table - grows to fill space, wrapped in titled panel
        .child(
            titled_panel(
                "Coins",
                theme,
                panel().flex_grow(1.0).child(build_coin_table(
                    &app.coins,
                    app.selected_index,
                    &app.checked,
                    theme,
                )),
            )
            .flex_grow(1.0),
        )
        // Footer - fixed height
        .child(
            build_overview_footer(selected_count, total_count, theme).margin(
                spacing.footer_margin(),
                0.0,
                0.0,
                0.0,
            ),
        )
}
