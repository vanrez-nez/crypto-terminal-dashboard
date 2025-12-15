//! Per-candle indicator calculations for chart overlays

use crate::api::Candle;

/// Per-candle indicator values computed from historical candles
pub struct CandleIndicators {
    /// RSI value per candle (0-100), indexed same as candles
    pub rsi: Vec<f64>,
    /// EMA 7 values per candle
    pub ema_7: Vec<f64>,
    /// EMA 25 values per candle
    pub ema_25: Vec<f64>,
    /// EMA 99 values per candle
    pub ema_99: Vec<f64>,
}

impl Default for CandleIndicators {
    fn default() -> Self {
        Self {
            rsi: Vec::new(),
            ema_7: Vec::new(),
            ema_25: Vec::new(),
            ema_99: Vec::new(),
        }
    }
}

impl CandleIndicators {
    /// Compute all indicators from a slice of candles
    pub fn from_candles(candles: &[Candle], rsi_period: usize) -> Self {
        if candles.is_empty() {
            return Self {
                rsi: Vec::new(),
                ema_7: Vec::new(),
                ema_25: Vec::new(),
                ema_99: Vec::new(),
            };
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        Self {
            rsi: Self::calculate_rsi_series(&closes, rsi_period),
            ema_7: Self::calculate_ema_series(&closes, 7),
            ema_25: Self::calculate_ema_series(&closes, 25),
            ema_99: Self::calculate_ema_series(&closes, 99),
        }
    }

    /// Calculate RSI for each candle (returns Vec same length as input)
    fn calculate_rsi_series(prices: &[f64], period: usize) -> Vec<f64> {
        let mut result = vec![50.0; prices.len()]; // Default neutral RSI

        if prices.len() < period + 1 {
            return result;
        }

        // Calculate price changes
        let changes: Vec<f64> = prices.windows(2).map(|w| w[1] - w[0]).collect();

        // Initialize with first period average
        let mut avg_gain: f64 =
            changes[..period].iter().filter(|&&c| c > 0.0).sum::<f64>() / period as f64;

        let mut avg_loss: f64 = changes[..period]
            .iter()
            .filter(|&&c| c < 0.0)
            .map(|c| c.abs())
            .sum::<f64>()
            / period as f64;

        // First RSI value
        result[period] = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
        };

        // Calculate remaining RSI values with Wilder smoothing
        for i in period..changes.len() {
            let change = changes[i];
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { change.abs() } else { 0.0 };

            avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
            avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;

            result[i + 1] = if avg_loss == 0.0 {
                100.0
            } else {
                100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
            };
        }

        result
    }

    /// Calculate EMA for each candle (returns Vec same length as input)
    fn calculate_ema_series(prices: &[f64], period: usize) -> Vec<f64> {
        let mut result = vec![0.0; prices.len()];

        if prices.is_empty() {
            return result;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);

        if prices.len() >= period {
            // Calculate initial SMA for first 'period' values
            let sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;

            // Fill initial values with running average
            for i in 0..period {
                result[i] = prices[..=i].iter().sum::<f64>() / (i + 1) as f64;
            }

            // Calculate EMA from period onwards
            let mut ema = sma;
            for i in period..prices.len() {
                ema = (prices[i] - ema) * multiplier + ema;
                result[i] = ema;
            }
        } else {
            // Not enough data, use running average
            for i in 0..prices.len() {
                result[i] = prices[..=i].iter().sum::<f64>() / (i + 1) as f64;
            }
        }

        result
    }
}
