//! Price panel widget displaying current price, 24h change, and range bar

use crate::base::layout::{HAlign, VAlign};
use crate::base::{panel, taffy, PanelBuilder};
use taffy::prelude::*;

use super::format::{format_change, format_price, format_price_short, price_change_color};
use super::theme::GlTheme;
use crate::mock::CoinData;

/// Build the price panel for a coin
pub fn build_price_panel(coin: &CoinData, theme: &GlTheme) -> PanelBuilder {
    let price_text = format_price(coin.price);
    let change_text = format_change(coin.change_24h);
    let gap = theme.panel_gap;

    // Calculate price color based on tick direction
    let avg_change = coin.avg_change();
    let price_color = price_change_color(coin.price, coin.prev_price, avg_change, theme);

    let price_delta = coin.price - coin.prev_price;
    let arrow_panel = if price_delta.abs() > f64::EPSILON {
        let arrow = if price_delta > 0.0 { "▲" } else { "▼" };
        Some(
            panel()
                .text(arrow, price_color, theme.font_big)
                .text_align(HAlign::Left, VAlign::Center),
        )
    } else {
        None
    };

    let change_color = if coin.change_24h > 0.0 {
        theme.positive
    } else if coin.change_24h < 0.0 {
        theme.negative
    } else {
        theme.foreground_muted
    };

    // Calculate range bar position (0.0 to 1.0)
    let range = coin.high_24h - coin.low_24h;
    let range_pos = if range > 0.0 {
        ((coin.price - coin.low_24h) / range).clamp(0.0, 1.0)
    } else {
        0.5
    };

    panel()
        .width(percent(1.0))
        .flex_direction(FlexDirection::Column)
        .gap(gap / 2.0)
        .child({
            let mut row = panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Baseline)
                .gap(gap / 2.0)
                .child(panel().text(&price_text, price_color, theme.font_big));
            if let Some(arrow_panel) = arrow_panel {
                row = row.child(arrow_panel);
            }
            row
        })
        // Change percentage row
        .child(
            panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                .gap(gap)
                .child(panel().text("24h:", theme.foreground_muted, theme.font_medium))
                .child(panel().text(&change_text, change_color, theme.font_medium)),
        )
        // Range bar row
        .child(build_range_bar(
            coin.low_24h,
            coin.high_24h,
            range_pos,
            theme,
        ))
}

/// Build a range bar showing price position within 24h range
fn build_range_bar(low: f64, high: f64, position: f64, theme: &GlTheme) -> PanelBuilder {
    let low_text = format!("L:{}", format_price_short(low));
    let high_text = format!("H:{}", format_price_short(high));
    let gap = theme.panel_gap;

    // Create bar segments: left filled, marker, right empty
    let left_pct = (position * 100.0) as f32;

    panel()
        .width(percent(1.0))
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .gap(gap / 2.0)
        // Low label
        .child(
            panel()
                .width(length(60.0))
                .text(&low_text, theme.foreground_muted, theme.font_small)
                .text_align(HAlign::Left, VAlign::Center),
        )
        // Bar container
        .child(
            panel()
                .flex_grow(1.0)
                .height(length(12.0))
                .background([0.2, 0.2, 0.2, 1.0])
                .flex_direction(FlexDirection::Row)
                // Filled portion
                .child(
                    panel()
                        .width(percent(left_pct / 100.0))
                        .height(percent(1.0))
                        .background(theme.accent),
                )
                // Marker
                .child(
                    panel()
                        .width(length(4.0))
                        .height(percent(1.0))
                        .background(theme.foreground),
                ),
        )
        // High label
        .child(
            panel()
                .width(length(60.0))
                .text(&high_text, theme.foreground_muted, theme.font_small)
                .text_align(HAlign::Right, VAlign::Center),
        )
}
