//! Overview view - coin table with selection

use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use crate::app::App;
use crate::base::view::ViewSpacing;
use crate::widgets::{
    coin_table::build_coin_table, control_footer::build_overview_footer,
    status_header::build_status_header, theme::GlTheme, titled_panel::titled_panel,
};

pub fn build_overview_view(app: &App, theme: &GlTheme, width: f32, height: f32) -> PanelBuilder {
    let selected_count = app.selected_count();
    let total_count = app.coins.len();
    let spacing = ViewSpacing::new(theme);

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
