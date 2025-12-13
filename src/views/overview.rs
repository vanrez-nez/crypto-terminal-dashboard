//! Overview view - coin table with selection

use dashboard_system::{panel, PanelBuilder, taffy};
use taffy::prelude::*;

use crate::app::App;
use crate::widgets::{
    build_coin_table,
    build_overview_footer,
    build_status_header,
    titled_panel,
    GlTheme,
};

pub fn build_overview_view(
    app: &App,
    theme: &GlTheme,
    width: f32,
    height: f32,
) -> PanelBuilder {
    let selected_count = app.checked.iter().filter(|&&c| c).count();
    let total_count = app.coins.len();
    let gap = theme.panel_gap;

    panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
        .gap(gap)
        .padding_all(gap)
        .background(theme.background)
        // Header - fixed height
        .child(
            build_status_header(
                app.view,
                &app.provider,
                app.time_window,
                app.chart_type,
                app.connection_status,
                theme,
            )
        )
        // Coin table - grows to fill space, wrapped in titled panel
        .child(
            titled_panel(
                "Coins",
                theme,
                panel()
                    .flex_grow(1.0)
                    .child(
                        build_coin_table(
                            &app.coins,
                            app.selected_index,
                            &app.checked,
                            theme,
                        )
                    )
            )
            .flex_grow(1.0)
        )
        // Footer - fixed height
        .child(
            build_overview_footer(selected_count, total_count, theme)
        )
}
