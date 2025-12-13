//! Price panel widget displaying current price, 24h change, and range bar

use dashboard_system::{panel, taffy, HAlign, PanelBuilder, VAlign};
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

    let (change_color, arrow) = if coin.change_24h >= 0.0 {
        (theme.positive, "▲")
    } else {
        (theme.negative, "▼")
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
        // Symbol and price row
        .child(
            panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .gap(gap)
                .child(panel().text(&coin.symbol, theme.accent, 1.2))
                .child(panel().text(&price_text, price_color, 1.1))
                .child(panel().text(arrow, change_color, 1.0)),
        )
        // Change percentage row
        .child(
            panel()
                .width(percent(1.0))
                .flex_direction(FlexDirection::Row)
                .gap(gap)
                .child(panel().text("24h:", theme.foreground_muted, 0.9))
                .child(panel().text(&change_text, change_color, 0.9)),
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
                .text(&low_text, theme.foreground_muted, 0.8)
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
                .text(&high_text, theme.foreground_muted, 0.8)
                .text_align(HAlign::Right, VAlign::Center),
        )
}
