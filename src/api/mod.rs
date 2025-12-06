pub mod coinbase;

/// Price update message from WebSocket
#[derive(Debug, Clone)]
pub enum PriceUpdate {
    /// Real-time price update
    Ticker {
        symbol: String,
        price: f64,
        change_24h: f64,
        volume_24h_usd: f64,
        volume_24h_base: f64,
        high_24h: f64,
        low_24h: f64,
    },
    /// Connection status change
    Connected,
    Disconnected,
    /// Error message
    Error(String),
}
