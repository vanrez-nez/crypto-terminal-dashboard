//! Positions table widget for displaying margin positions

use crate::api::margin::MarginPosition;
use crate::base::layout::HAlign;
use crate::base::PanelBuilder;

use super::format::format_price;
use super::table::{build_table_styled, estimate_column_widths, CellBuilder, ColumnConfig, ColumnWidth, RowStyle, TableRow};
use super::theme::GlTheme;

/// Build the positions table widget using the reusable table component
pub fn build_positions_table(
    positions: &[MarginPosition],
    selected_index: usize,
    theme: &GlTheme,
) -> PanelBuilder {
    // Start with proportional column definitions that will be calculated
    // These proportions represent the relative importance/typical size of each column
    let mut columns = vec![
        ColumnConfig::auto("ASSET", 0.0).with_align(HAlign::Left),
        ColumnConfig::auto("AMOUNT", 0.0).with_align(HAlign::Left),
        ColumnConfig::auto("PRICE", 0.0).with_align(HAlign::Left),
        ColumnConfig::auto("BORROWED VAL", 0.0).with_align(HAlign::Left),
        ColumnConfig::auto("NET VALUE", 0.0).with_align(HAlign::Left),
    ];

    // Convert positions to table rows
    let rows: Vec<TableRow> = positions
        .iter()
        .map(|pos| {
            // Color for borrowed (red if borrowed, normal if not)
            let borrowed_color = if pos.borrowed > 0.0001 {
                theme.negative
            } else {
                theme.foreground
            };

            // Color for net value (green if positive, red if negative)
            let net_color = if pos.net_value_usd > 0.0 {
                theme.positive
            } else if pos.net_value_usd < 0.0 {
                theme.negative
            } else {
                theme.foreground
            };

            vec![
                CellBuilder::text(&pos.asset, theme.foreground),
                CellBuilder::text(&format!("{:.4}", pos.free + pos.locked), theme.foreground),
                CellBuilder::text(&format_price(pos.current_price), theme.foreground),
                CellBuilder::text(&format_price(pos.borrowed_value_usd), borrowed_color),
                CellBuilder::text(&format_price(pos.net_value_usd), net_color),
            ]
        })
        .collect();

    // Calculate minimum column widths based on content
    columns = estimate_column_widths(
        &columns,
        &rows,
        theme.font_size,
        theme.font_normal,
        theme.panel_gap * 2.0,
    );

    // Convert calculated minimum widths to flex-grow values for proportional expansion
    // Using proportion() (flex_basis=0) ensures columns align across all rows
    columns = columns
        .into_iter()
        .map(|mut col| {
            if let ColumnWidth::Auto(min_width) = col.width {
                // Use minimum width as flex-grow ratio
                col.width = ColumnWidth::Flex(min_width);
            }
            col
        })
        .collect();

    // Create row styles for selection
    let row_styles: Vec<RowStyle> = (0..rows.len())
        .map(|i| RowStyle {
            background: if i == selected_index {
                Some(theme.selection_bg)
            } else {
                None
            },
            height: None,
        })
        .collect();

    build_table_styled(&columns, &rows, &row_styles, theme)
}
