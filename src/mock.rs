use crate::api::Candle;
use std::collections::VecDeque;

const CHANGE_HISTORY_SIZE: usize = 50; // Number of samples to average

pub struct CoinData {
    pub symbol: String,
    #[allow(dead_code)]
    pub name: String,
    pub price: f64,
    pub prev_price: f64,               // Previous price for change detection
    pub change_history: VecDeque<f64>, // History of absolute price changes
    pub change_24h: f64,
    pub volume_usd: f64,
    pub volume_base: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub indicators: IndicatorData,
    pub sparkline: Vec<u64>,
    pub candles: Vec<Candle>,
}

pub struct IndicatorData {
    // RSI values
    pub rsi_6: f64,
    pub rsi_12: f64,
    pub rsi_24: f64,
    // EMA values
    pub ema_7: f64,
    pub ema_25: f64,
    pub ema_99: f64,
    // MACD
    pub macd_line: f64,
    pub macd_signal: f64,
    pub macd_histogram: f64,
}

impl Default for IndicatorData {
    fn default() -> Self {
        Self {
            rsi_6: 50.0,
            rsi_12: 50.0,
            rsi_24: 50.0,
            ema_7: 0.0,
            ema_25: 0.0,
            ema_99: 0.0,
            macd_line: 0.0,
            macd_signal: 0.0,
            macd_histogram: 0.0,
        }
    }
}

impl CoinData {
    pub fn new(symbol: &str, name: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            name: name.to_string(),
            price: 0.0,
            prev_price: 0.0,
            change_history: VecDeque::with_capacity(CHANGE_HISTORY_SIZE),
            change_24h: 0.0,
            volume_usd: 0.0,
            volume_base: 0.0,
            high_24h: 0.0,
            low_24h: 0.0,
            indicators: IndicatorData::default(),
            sparkline: vec![50; 20],
            candles: Vec::new(),
        }
    }

    /// Calculate average absolute change from history
    pub fn avg_change(&self) -> f64 {
        if self.change_history.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.change_history.iter().sum();
        sum / self.change_history.len() as f64
    }

    /// Update current price from WebSocket ticker and recalculate indicators
    pub fn update_price(&mut self, price: f64) {
        // Track change history for dynamic color gradient
        if self.price > 0.0 {
            let abs_change = (price - self.price).abs();
            if abs_change > 0.0 {
                // Add to history, remove oldest if at capacity
                if self.change_history.len() >= CHANGE_HISTORY_SIZE {
                    self.change_history.pop_front();
                }
                self.change_history.push_back(abs_change);
            }
        }

        // Track previous price
        self.prev_price = self.price;
        self.price = price;

        // Update the last candle's close price to current live price
        // This makes indicators reflect "what if the candle closed now"
        if let Some(last_candle) = self.candles.last_mut() {
            last_candle.close = price;
            // Also update high/low if price exceeds them
            if price > last_candle.high {
                last_candle.high = price;
            }
            if price < last_candle.low {
                last_candle.low = price;
            }
            // Recalculate indicators with updated candle
            self.recalculate_indicators();
        }
    }

    /// Set candles from historical data and recalculate indicators
    pub fn set_candles(&mut self, candles: Vec<Candle>) {
        self.candles = candles;
        self.recalculate_indicators();
        self.update_sparkline();

        // Update current price from latest candle if available
        if let Some(last) = self.candles.last() {
            if self.price == 0.0 {
                self.price = last.close;
            }
        }
    }

    fn recalculate_indicators(&mut self) {
        // Extract close prices from candles
        let closes: Vec<f64> = self.candles.iter().map(|c| c.close).collect();

        if closes.len() < 2 {
            return;
        }

        // Calculate EMAs (7, 25, 99)
        self.indicators.ema_7 = Self::calculate_ema(&closes, 7);
        self.indicators.ema_25 = Self::calculate_ema(&closes, 25);
        self.indicators.ema_99 = Self::calculate_ema(&closes, 99);

        // Calculate RSIs (6, 12, 24)
        self.indicators.rsi_6 = Self::calculate_rsi(&closes, 6);
        self.indicators.rsi_12 = Self::calculate_rsi(&closes, 12);
        self.indicators.rsi_24 = Self::calculate_rsi(&closes, 24);

        // Calculate MACD (12, 26, 9)
        let ema_12 = Self::calculate_ema(&closes, 12);
        let ema_26 = Self::calculate_ema(&closes, 26);
        self.indicators.macd_line = ema_12 - ema_26;

        // Calculate MACD signal line (9-period EMA of MACD line)
        // For proper calculation, we'd need MACD history, but approximate with smoothing
        let macd_smoothing = 2.0 / 10.0;
        self.indicators.macd_signal = self.indicators.macd_signal * (1.0 - macd_smoothing)
            + self.indicators.macd_line * macd_smoothing;
        self.indicators.macd_histogram = self.indicators.macd_line - self.indicators.macd_signal;
    }

    fn update_sparkline(&mut self) {
        if self.candles.len() < 2 {
            return;
        }

        // Take last 20 candles for sparkline
        let candles_to_use: Vec<&Candle> = self.candles.iter().rev().take(20).collect();
        if candles_to_use.len() < 2 {
            return;
        }

        // Find min and max close prices
        let closes: Vec<f64> = candles_to_use.iter().map(|c| c.close).collect();
        let min = closes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = closes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = if max > min { max - min } else { 1.0 };

        // Normalize to 20-80 range
        self.sparkline = closes
            .iter()
            .rev() // Reverse back to chronological order
            .map(|&price| {
                let normalized = ((price - min) / range) * 60.0 + 20.0;
                normalized.clamp(20.0, 80.0) as u64
            })
            .collect();

        // Ensure we have exactly 20 points
        while self.sparkline.len() < 20 {
            self.sparkline.insert(0, 50);
        }
        if self.sparkline.len() > 20 {
            self.sparkline = self.sparkline[self.sparkline.len() - 20..].to_vec();
        }
    }

    fn calculate_ema(prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }
        if prices.len() < period {
            // Not enough data, return SMA
            return prices.iter().sum::<f64>() / prices.len() as f64;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);

        // Start with SMA of first 'period' prices
        let sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;

        // Calculate EMA from there
        let mut ema = sma;
        for price in &prices[period..] {
            ema = (price - ema) * multiplier + ema;
        }
        ema
    }

    /// Calculate RSI using Wilder's smoothed moving average
    /// This matches the RSI calculation used by Binance and other trading platforms
    fn calculate_rsi(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0; // Neutral RSI when not enough data
        }

        // Calculate price changes
        let changes: Vec<f64> = prices.windows(2).map(|w| w[1] - w[0]).collect();

        if changes.len() < period {
            return 50.0;
        }

        // First average: SMA of first `period` changes
        let mut avg_gain: f64 =
            changes[..period].iter().filter(|&&c| c > 0.0).sum::<f64>() / period as f64;

        let mut avg_loss: f64 = changes[..period]
            .iter()
            .filter(|&&c| c < 0.0)
            .map(|c| c.abs())
            .sum::<f64>()
            / period as f64;

        // Apply Wilder's smoothing for remaining changes
        // Formula: avg = (prev_avg * (period - 1) + current) / period
        for change in &changes[period..] {
            let gain = if *change > 0.0 { *change } else { 0.0 };
            let loss = if *change < 0.0 { change.abs() } else { 0.0 };

            avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
            avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        }

        // Calculate RSI
        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// Get price data points for chart: Vec<(x, y)> where x is index, y is close price
    pub fn chart_data(&self) -> Vec<(f64, f64)> {
        self.candles
            .iter()
            .enumerate()
            .map(|(i, c)| (i as f64, c.close))
            .collect()
    }

    /// Get min/max price for Y-axis bounds with padding
    pub fn price_bounds(&self) -> (f64, f64) {
        if self.candles.is_empty() {
            return (0.0, 100.0);
        }
        let closes: Vec<f64> = self.candles.iter().map(|c| c.close).collect();
        let min = closes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = closes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        // Add 0.5% padding
        (min * 0.995, max * 1.005)
    }

    /// Get the timestamp of the last candle (for countdown timer)
    pub fn last_candle_time(&self) -> Option<i64> {
        self.candles.last().map(|c| c.time)
    }
}

pub fn generate_mock_coins() -> Vec<CoinData> {
    vec![
        CoinData {
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            price: 67432.10,
            prev_price: 67432.10,
            change_history: VecDeque::new(),
            change_24h: 2.34,
            volume_usd: 28_400_000_000.0,
            volume_base: 421_234.0,
            high_24h: 68102.00,
            low_24h: 65201.00,
            indicators: IndicatorData {
                rsi_6: 62.5,
                rsi_12: 58.3,
                rsi_24: 55.1,
                ema_7: 67200.00,
                ema_25: 66430.00,
                ema_99: 64200.00,
                macd_line: 12.4,
                macd_signal: 8.2,
                macd_histogram: 4.2,
            },
            sparkline: vec![
                65, 66, 64, 67, 68, 70, 69, 71, 72, 70, 68, 69, 71, 73, 72, 70, 68, 69, 70, 72,
            ],
            candles: Vec::new(),
        },
        CoinData {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            price: 3521.45,
            prev_price: 3521.45,
            change_history: VecDeque::new(),
            change_24h: -0.82,
            volume_usd: 14_200_000_000.0,
            volume_base: 4_032_150.0,
            high_24h: 3612.30,
            low_24h: 3480.10,
            indicators: IndicatorData {
                rsi_6: 38.2,
                rsi_12: 42.1,
                rsi_24: 45.5,
                ema_7: 3530.00,
                ema_25: 3560.00,
                ema_99: 3480.00,
                macd_line: -5.1,
                macd_signal: -3.2,
                macd_histogram: -1.9,
            },
            sparkline: vec![
                72, 70, 68, 66, 65, 64, 62, 63, 65, 67, 69, 71, 73, 72, 70, 68, 66, 64, 65, 67,
            ],
            candles: Vec::new(),
        },
        CoinData {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            price: 142.33,
            prev_price: 142.33,
            change_history: VecDeque::new(),
            change_24h: 5.21,
            volume_usd: 2_100_000_000.0,
            volume_base: 14_753_000.0,
            high_24h: 145.00,
            low_24h: 135.00,
            indicators: IndicatorData {
                rsi_6: 72.1,
                rsi_12: 65.2,
                rsi_24: 60.8,
                ema_7: 141.50,
                ema_25: 138.00,
                ema_99: 128.00,
                macd_line: 3.2,
                macd_signal: 2.1,
                macd_histogram: 1.1,
            },
            sparkline: vec![
                55, 58, 60, 63, 65, 68, 70, 72, 75, 73, 71, 74, 76, 78, 80, 82, 80, 78, 76, 75,
            ],
            candles: Vec::new(),
        },
        CoinData {
            symbol: "XRP".to_string(),
            name: "Ripple".to_string(),
            price: 0.5234,
            prev_price: 0.5234,
            change_history: VecDeque::new(),
            change_24h: 1.02,
            volume_usd: 1_800_000_000.0,
            volume_base: 3_439_816_000.0,
            high_24h: 0.53,
            low_24h: 0.51,
            indicators: IndicatorData {
                rsi_6: 52.3,
                rsi_12: 48.7,
                rsi_24: 50.2,
                ema_7: 0.522,
                ema_25: 0.518,
                ema_99: 0.505,
                macd_line: 0.005,
                macd_signal: 0.003,
                macd_histogram: 0.002,
            },
            sparkline: vec![
                50, 51, 52, 51, 50, 49, 50, 51, 52, 53, 52, 51, 50, 51, 52, 53, 54, 53, 52, 51,
            ],
            candles: Vec::new(),
        },
        CoinData {
            symbol: "ADA".to_string(),
            name: "Cardano".to_string(),
            price: 0.4521,
            prev_price: 0.4521,
            change_history: VecDeque::new(),
            change_24h: -0.34,
            volume_usd: 890_000_000.0,
            volume_base: 1_968_368_000.0,
            high_24h: 0.46,
            low_24h: 0.44,
            indicators: IndicatorData {
                rsi_6: 48.5,
                rsi_12: 51.2,
                rsi_24: 49.8,
                ema_7: 0.452,
                ema_25: 0.450,
                ema_99: 0.445,
                macd_line: -0.002,
                macd_signal: -0.001,
                macd_histogram: -0.001,
            },
            sparkline: vec![
                46, 45, 44, 45, 46, 45, 44, 43, 44, 45, 46, 45, 44, 45, 46, 47, 46, 45, 44, 45,
            ],
            candles: Vec::new(),
        },
    ]
}

/// Create coins from pairs list
/// Supports both formats: "BTC-USD" (Coinbase) and "BTCUSDT" (Binance)
pub fn coins_from_pairs(pairs: &[String]) -> Vec<CoinData> {
    pairs
        .iter()
        .map(|pair| {
            // Handle both "BTC-USD" and "BTCUSDT" formats
            let symbol = if pair.contains('-') {
                pair.split('-').next().unwrap_or(pair)
            } else {
                pair.trim_end_matches("USDT")
            };
            let name = symbol_to_name(symbol);
            CoinData::new(symbol, &name)
        })
        .collect()
}

fn symbol_to_name(symbol: &str) -> String {
    match symbol {
        "BTC" => "Bitcoin".to_string(),
        "ETH" => "Ethereum".to_string(),
        "SOL" => "Solana".to_string(),
        "XRP" => "Ripple".to_string(),
        "ADA" => "Cardano".to_string(),
        "DOGE" => "Dogecoin".to_string(),
        "DOT" => "Polkadot".to_string(),
        "AVAX" => "Avalanche".to_string(),
        "MATIC" => "Polygon".to_string(),
        "LINK" => "Chainlink".to_string(),
        _ => symbol.to_string(),
    }
}
