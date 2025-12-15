//! Ticker tones - audio feedback for price movements
//!
//! Plays generated tones on price ticks:
//! - Different base frequencies for UP vs DOWN movements
//! - Higher pitch = bigger change magnitude (relative to historical average)
//! - Rate-limited to prevent overlapping tones
//! - Only plays once per actual price change

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::audio;
use crate::config::TickerTonesConfig;
use crate::mock::CoinData;
use crate::widgets::format::round_to_display;

/// Minimum interval between tones in milliseconds.
/// 100ms allows up to 10 tones per second.
const MIN_TONE_INTERVAL_MS: u64 = 100;

/// Track when the last tone was played (for rate limiting)
static LAST_TONE_TIME: Mutex<Option<Instant>> = Mutex::new(None);

/// Track the price at which we last played a tone for each coin
/// This prevents playing multiple tones for the same price change
static LAST_TONE_PRICES: Mutex<Option<HashMap<String, f64>>> = Mutex::new(None);

/// Check if enough time has passed since the last tone
fn can_play_tone(duration_ms: u32) -> bool {
    let min_interval = Duration::from_millis(MIN_TONE_INTERVAL_MS.max(duration_ms as u64));

    let last = LAST_TONE_TIME.lock().unwrap();
    match *last {
        Some(last_time) => last_time.elapsed() >= min_interval,
        None => true,
    }
}

/// Record that a tone was just played
fn record_tone_played() {
    let mut last = LAST_TONE_TIME.lock().unwrap();
    *last = Some(Instant::now());
}

/// Check if we should play a tone for this coin's price
/// Returns Some((price_delta, is_up)) if we should play, None otherwise
fn check_price_change(coin: &CoinData) -> Option<(f64, bool)> {
    let mut prices_guard = LAST_TONE_PRICES.lock().unwrap();
    let prices = prices_guard.get_or_insert_with(HashMap::new);

    // Round to display precision - tone plays only when visible price changes
    let current_rounded = round_to_display(coin.price);
    let symbol = &coin.symbol;

    // Get the last rounded price we played a tone for
    let last_tone_price = prices.get(symbol).copied();

    match last_tone_price {
        Some(ltp) => {
            if current_rounded != ltp {
                // Visible price changed - update and signal to play
                let delta = current_rounded - ltp;
                prices.insert(symbol.clone(), current_rounded);
                Some((delta, delta > 0.0))
            } else {
                // Visible price hasn't changed
                None
            }
        }
        None => {
            // First time seeing this coin - store price but don't play
            prices.insert(symbol.clone(), current_rounded);
            None
        }
    }
}

/// Calculate and play a ticker tone based on price change
fn play_tone_for_change(
    price_delta: f64,
    is_up: bool,
    avg_change: f64,
    config: &TickerTonesConfig,
) {
    let abs_change = price_delta.abs();

    // Calculate magnitude ratio (1.0 = average change, up to 4.0 = big move)
    let ratio = if avg_change > 0.0 {
        (abs_change / avg_change).min(4.0)
    } else {
        1.0
    };
    let ratio = ratio as f32;

    // Scale factor: 0.0 at ratio=1, 1.0 at ratio=4
    let scale = (ratio - 1.0) / 3.0;

    // Use logarithmic scaling for natural pitch perception
    let frequency = if is_up {
        // UP: logarithmic scale from base_freq_up to max_freq
        // At scale=0: base_freq_up, at scale=1: max_freq
        let freq_ratio = config.max_freq / config.base_freq_up;
        config.base_freq_up * freq_ratio.powf(scale)
    } else {
        // DOWN: logarithmic scale from base_freq_down to min_freq
        // At scale=0: base_freq_down, at scale=1: min_freq
        let freq_ratio = config.min_freq / config.base_freq_down;
        config.base_freq_down * freq_ratio.powf(scale)
    };

    audio::play_tone(frequency, config.duration_ms);
    record_tone_played();
}

/// Process ticker tone for checked coins only.
/// Only plays one tone per actual price change.
pub fn process_ticker_tones(coins: &[CoinData], checked: &[bool], config: &TickerTonesConfig) {
    if !config.enabled {
        return;
    }

    // Rate limit check
    if !can_play_tone(config.duration_ms) {
        return;
    }

    // Find a checked coin with an actual price change
    for (i, coin) in coins.iter().enumerate() {
        // Skip unchecked coins
        if !checked.get(i).copied().unwrap_or(false) {
            continue;
        }

        // Skip if no previous price history
        if coin.prev_price <= 0.0 {
            continue;
        }

        // Skip if not enough change history for magnitude calculation
        let avg_change = coin.avg_change();
        if avg_change <= 0.0 {
            continue;
        }

        // Check if price actually changed since last tone
        if let Some((price_delta, is_up)) = check_price_change(coin) {
            play_tone_for_change(price_delta, is_up, avg_change, config);
            // Only play one tone per tick
            break;
        }
    }
}
