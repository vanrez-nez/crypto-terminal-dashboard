//! Overview view - coin table with selection

use dashboard_system::{panel, PanelBuilder, taffy};
use taffy::prelude::*;

use crate::app::App;
use crate::widgets::{
    build_coin_table,
    build_overview_footer,
    build_status_header,
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

    panel()
        .width(length(width))
        .height(length(height))
        .flex_direction(FlexDirection::Column)
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
        // Coin table - grows to fill space
        .child(
            build_coin_table(
                &app.coins,
                app.selected_index,
                &app.checked,
                theme,
            )
        )
        // Footer - fixed height
        .child(
            build_overview_footer(selected_count, total_count, theme)
        )
}
