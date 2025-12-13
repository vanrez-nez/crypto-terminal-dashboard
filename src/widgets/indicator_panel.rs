//! Indicator panel widget displaying RSI and EMA values in aligned columns

use crate::base::layout::{HAlign, VAlign};
use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::theme::GlTheme;
use crate::mock::IndicatorData;

/// Build the indicator panel displaying technical indicators
pub fn build_indicator_panel(indicators: &IndicatorData, theme: &GlTheme) -> PanelBuilder {
    let gap = theme.panel_gap;
    let freq_colors = [
        theme.indicator_primary,
        theme.indicator_secondary,
        theme.indicator_tertiary,
    ];

    panel()
        .width(percent(1.0))
        .flex_direction(FlexDirection::Column)
        .gap(gap / 2.0)
        .child(build_three_column_row(
            "RSI",
            [
                ("6", indicators.rsi_6),
                ("12", indicators.rsi_12),
                ("24", indicators.rsi_24),
            ],
            freq_colors,
            theme,
        ))
        .child(build_three_column_row(
            "EMA",
            [
                ("7", indicators.ema_7),
                ("25", indicators.ema_25),
                ("99", indicators.ema_99),
            ],
            freq_colors,
            theme,
        ))
}

fn build_three_column_row(
    prefix: &str,
    values: [(&str, f64); 3],
    freq_colors: [[f32; 4]; 3],
    theme: &GlTheme,
) -> PanelBuilder {
    let gap = theme.panel_gap;
    panel()
        .width(percent(1.0))
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(gap / 2.0)
        .children(
            values
                .iter()
                .zip(freq_colors.iter())
                .map(|((label, value), color)| {
                    build_indicator_column(prefix, label, *value, *color, theme)
                })
                .collect::<Vec<_>>(),
        )
}

fn build_indicator_column(
    prefix: &str,
    label: &str,
    value: f64,
    column_color: [f32; 4],
    theme: &GlTheme,
) -> PanelBuilder {
    let value_text = format!("{:.1}", value);
    panel()
        .flex_grow(1.0)
        .flex_direction(FlexDirection::Row)
        .gap(theme.panel_gap / 4.0)
        .child(
            panel()
                .text(
                    &format!("{}({}):", prefix, label),
                    column_color,
                    theme.font_medium,
                )
                .text_align(HAlign::Left, VAlign::Center),
        )
        .child(
            panel()
                .text(&format!(" {}", value_text), column_color, theme.font_medium)
                .text_align(HAlign::Left, VAlign::Center),
        )
}
