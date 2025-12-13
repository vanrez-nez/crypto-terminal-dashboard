//! Indicator panel widget displaying RSI, EMA, and MACD values

use dashboard_system::{panel, HAlign, PanelBuilder, VAlign, taffy};
use taffy::prelude::*;

use super::theme::GlTheme;
use crate::mock::IndicatorData;

/// Build the indicator panel displaying technical indicators
pub fn build_indicator_panel(indicators: &IndicatorData, theme: &GlTheme) -> PanelBuilder {
    panel()
        .width(percent(1.0))
        .flex_direction(FlexDirection::Column)
        .gap(4.0)
        // RSI row
        .child(
            panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                .gap(12.0)
                .child(
                    panel()
                        .width(length(40.0))
                        .text("RSI", theme.indicator_primary, 0.9)
                )
                .child(build_indicator_value("6", indicators.rsi_6, theme))
                .child(build_indicator_value("12", indicators.rsi_12, theme))
                .child(build_indicator_value("24", indicators.rsi_24, theme))
        )
        // EMA row
        .child(
            panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                .gap(12.0)
                .child(
                    panel()
                        .width(length(40.0))
                        .text("EMA", theme.indicator_secondary, 0.9)
                )
                .child(build_indicator_value("7", indicators.ema_7, theme))
                .child(build_indicator_value("25", indicators.ema_25, theme))
                .child(build_indicator_value("99", indicators.ema_99, theme))
        )
        // MACD row (optional, only if available)
        .child(
            panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                .gap(12.0)
                .child(
                    panel()
                        .width(length(40.0))
                        .text("MACD", theme.indicator_tertiary, 0.9)
                )
                .child(build_macd_value("Line", indicators.macd_line, theme))
                .child(build_macd_value("Sig", indicators.macd_signal, theme))
                .child(build_macd_histogram(indicators.macd_histogram, theme))
        )
}

fn build_indicator_value(label: &str, value: f64, theme: &GlTheme) -> PanelBuilder {
    let value_text = format!("{:.1}", value);
    let value_color = get_rsi_color(value, theme);

    panel()
        .flex_direction(FlexDirection::Row)
        .gap(4.0)
        .child(
            panel().text(label, theme.foreground_muted, 0.8)
        )
        .child(
            panel().text(&value_text, value_color, 0.9)
        )
}

fn build_macd_value(label: &str, value: f64, theme: &GlTheme) -> PanelBuilder {
    let value_text = format!("{:.2}", value);
    let value_color = if value >= 0.0 {
        theme.positive
    } else {
        theme.negative
    };

    panel()
        .flex_direction(FlexDirection::Row)
        .gap(4.0)
        .child(
            panel().text(label, theme.foreground_muted, 0.8)
        )
        .child(
            panel().text(&value_text, value_color, 0.9)
        )
}

fn build_macd_histogram(value: f64, theme: &GlTheme) -> PanelBuilder {
    let value_text = format!("{:.2}", value);
    let value_color = if value >= 0.0 {
        theme.positive
    } else {
        theme.negative
    };

    panel()
        .flex_direction(FlexDirection::Row)
        .gap(4.0)
        .child(
            panel().text("Hist", theme.foreground_muted, 0.8)
        )
        .child(
            panel().text(&value_text, value_color, 0.9)
        )
}

/// Get color for RSI value (oversold < 30, overbought > 70)
fn get_rsi_color(rsi: f64, theme: &GlTheme) -> [f32; 4] {
    if rsi <= 30.0 {
        theme.positive // Oversold - potential buy
    } else if rsi >= 70.0 {
        theme.negative // Overbought - potential sell
    } else {
        theme.foreground // Neutral
    }
}
