pub mod binance;
pub mod coinbase;
pub mod margin;
pub mod news;

/// OHLC candle data
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Candle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Price update message from WebSocket or REST API
#[derive(Debug, Clone)]
pub enum PriceUpdate {
    /// Real-time price update from WebSocket
    Ticker {
        symbol: String,
        price: f64,
        change_24h: f64,
        volume_24h_usd: f64,
        volume_24h_base: f64,
        high_24h: f64,
        low_24h: f64,
    },
    /// Historical candle data from REST API
    Candles {
        symbol: String,
        candles: Vec<Candle>,
    },
    /// Connection status change
    Connected,
    Disconnected,
    /// Error message
    #[allow(dead_code)]
    Error(String),
    /// Margin account positions update
    MarginPositions {
        account: margin::MarginAccount,
    },
}
