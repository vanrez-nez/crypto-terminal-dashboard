pub struct CoinData {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_24h: f64,
    pub volume_usd: f64,
    pub volume_base: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub indicators: IndicatorData,
    pub sparkline: Vec<u64>,
    pub price_history: Vec<f64>,
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
            change_24h: 0.0,
            volume_usd: 0.0,
            volume_base: 0.0,
            high_24h: 0.0,
            low_24h: 0.0,
            indicators: IndicatorData::default(),
            sparkline: vec![50; 20],
            price_history: Vec::with_capacity(200),
        }
    }

    /// Add a price to history and recalculate indicators
    pub fn update_price(&mut self, price: f64) {
        self.price = price;
        self.price_history.push(price);

        // Keep last 200 prices
        if self.price_history.len() > 200 {
            self.price_history.remove(0);
        }

        // Update sparkline
        if self.price_history.len() >= 2 {
            let prev = self.price_history[self.price_history.len() - 2];
            if prev > 0.0 {
                let normalized = ((price / prev) * 50.0) as u64;
                self.sparkline.remove(0);
                self.sparkline.push(normalized.clamp(20, 80));
            }
        }

        self.recalculate_indicators();
    }

    fn recalculate_indicators(&mut self) {
        let prices = &self.price_history;
        if prices.len() < 2 {
            return;
        }

        // Calculate EMAs
        self.indicators.ema_9 = Self::calculate_ema(prices, 9);
        self.indicators.ema_21 = Self::calculate_ema(prices, 21);

        // Calculate RSI (14 periods)
        self.indicators.rsi = Self::calculate_rsi(prices, 14);

        // Calculate MACD (12, 26, 9)
        let ema_12 = Self::calculate_ema(prices, 12);
        let ema_26 = Self::calculate_ema(prices, 26);
        self.indicators.macd_line = ema_12 - ema_26;

        // For signal line, we'd need MACD history - approximate with current
        // In a real implementation, we'd track MACD history separately
        let macd_smoothing = 2.0 / 10.0; // 9-period smoothing
        self.indicators.macd_signal = self.indicators.macd_signal * (1.0 - macd_smoothing)
            + self.indicators.macd_line * macd_smoothing;
        self.indicators.macd_histogram = self.indicators.macd_line - self.indicators.macd_signal;
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

    fn calculate_rsi(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0; // Neutral RSI when not enough data
        }

        let mut gains = 0.0;
        let mut losses = 0.0;
        let start = prices.len().saturating_sub(period + 1);

        for i in (start + 1)..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change; // Make positive
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }
}

pub fn generate_mock_coins() -> Vec<CoinData> {
    vec![
        CoinData {
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            price: 67432.10,
            change_24h: 2.34,
            volume_usd: 28_400_000_000.0,
            volume_base: 421_234.0,
            high_24h: 68102.00,
            low_24h: 65201.00,
            indicators: IndicatorData {
                rsi: 58.3,
                ema_9: 67102.00,
                ema_21: 66430.00,
                ema_50: 64200.00,
                sma_20: 66800.00,
                sma_50: 65500.00,
                sma_200: 61000.00,
                macd_line: 12.4,
                macd_signal: 8.2,
                macd_histogram: 4.2,
            },
            sparkline: vec![65, 66, 64, 67, 68, 70, 69, 71, 72, 70, 68, 69, 71, 73, 72, 70, 68, 69, 70, 72],
            price_history: vec![65000.0, 65500.0, 66000.0, 66200.0, 66800.0, 67000.0, 67200.0, 67432.10],
        },
        CoinData {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            price: 3521.45,
            change_24h: -0.82,
            volume_usd: 14_200_000_000.0,
            volume_base: 4_032_150.0,
            high_24h: 3612.30,
            low_24h: 3480.10,
            indicators: IndicatorData {
                rsi: 42.1,
                ema_9: 3540.00,
                ema_21: 3580.00,
                ema_50: 3620.00,
                sma_20: 3560.00,
                sma_50: 3600.00,
                sma_200: 3400.00,
                macd_line: -5.1,
                macd_signal: -3.2,
                macd_histogram: -1.9,
            },
            sparkline: vec![72, 70, 68, 66, 65, 64, 62, 63, 65, 67, 69, 71, 73, 72, 70, 68, 66, 64, 65, 67],
            price_history: vec![3600.0, 3580.0, 3560.0, 3540.0, 3520.0, 3510.0, 3515.0, 3521.45],
        },
        CoinData {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            price: 142.33,
            change_24h: 5.21,
            volume_usd: 2_100_000_000.0,
            volume_base: 14_753_000.0,
            high_24h: 145.00,
            low_24h: 135.00,
            indicators: IndicatorData {
                rsi: 65.2,
                ema_9: 140.50,
                ema_21: 138.00,
                ema_50: 132.00,
                sma_20: 139.00,
                sma_50: 135.00,
                sma_200: 120.00,
                macd_line: 3.2,
                macd_signal: 2.1,
                macd_histogram: 1.1,
            },
            sparkline: vec![55, 58, 60, 63, 65, 68, 70, 72, 75, 73, 71, 74, 76, 78, 80, 82, 80, 78, 76, 75],
            price_history: vec![135.0, 137.0, 139.0, 140.0, 141.0, 142.0, 142.5, 142.33],
        },
        CoinData {
            symbol: "XRP".to_string(),
            name: "Ripple".to_string(),
            price: 0.5234,
            change_24h: 1.02,
            volume_usd: 1_800_000_000.0,
            volume_base: 3_439_816_000.0,
            high_24h: 0.53,
            low_24h: 0.51,
            indicators: IndicatorData {
                rsi: 48.7,
                ema_9: 0.52,
                ema_21: 0.51,
                ema_50: 0.50,
                sma_20: 0.52,
                sma_50: 0.51,
                sma_200: 0.48,
                macd_line: 0.005,
                macd_signal: 0.003,
                macd_histogram: 0.002,
            },
            sparkline: vec![50, 51, 52, 51, 50, 49, 50, 51, 52, 53, 52, 51, 50, 51, 52, 53, 54, 53, 52, 51],
            price_history: vec![0.51, 0.515, 0.52, 0.518, 0.522, 0.525, 0.523, 0.5234],
        },
        CoinData {
            symbol: "ADA".to_string(),
            name: "Cardano".to_string(),
            price: 0.4521,
            change_24h: -0.34,
            volume_usd: 890_000_000.0,
            volume_base: 1_968_368_000.0,
            high_24h: 0.46,
            low_24h: 0.44,
            indicators: IndicatorData {
                rsi: 51.2,
                ema_9: 0.45,
                ema_21: 0.45,
                ema_50: 0.44,
                sma_20: 0.45,
                sma_50: 0.44,
                sma_200: 0.42,
                macd_line: -0.002,
                macd_signal: -0.001,
                macd_histogram: -0.001,
            },
            sparkline: vec![46, 45, 44, 45, 46, 45, 44, 43, 44, 45, 46, 45, 44, 45, 46, 47, 46, 45, 44, 45],
            price_history: vec![0.455, 0.453, 0.451, 0.452, 0.450, 0.451, 0.452, 0.4521],
        },
    ]
}

/// Create coins from pairs list (e.g., ["BTC-USD", "ETH-USD"])
pub fn coins_from_pairs(pairs: &[String]) -> Vec<CoinData> {
    pairs
        .iter()
        .map(|pair| {
            let symbol = pair.split('-').next().unwrap_or(pair);
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
