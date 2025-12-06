use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::{Candle, PriceUpdate};

const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/stream";
const BINANCE_REST_URL: &str = "https://api.binance.com";

/// Binance 24hr ticker message from WebSocket
#[derive(Deserialize, Debug)]
struct TickerData {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    close_price: String,
    #[serde(rename = "o")]
    open_price: String,
    #[serde(rename = "h")]
    high_price: String,
    #[serde(rename = "l")]
    low_price: String,
    #[serde(rename = "v")]
    base_volume: String,
    #[serde(rename = "q")]
    quote_volume: String,
    #[serde(rename = "P")]
    price_change_percent: String,
}

#[derive(Deserialize, Debug)]
struct StreamMessage {
    stream: String,
    data: TickerData,
}

pub struct BinanceProvider {
    pairs: Vec<String>,
}

impl BinanceProvider {
    pub fn new(pairs: Vec<String>) -> Self {
        Self { pairs }
    }

    /// Run the WebSocket connection and send updates through the channel
    pub async fn run(self, tx: mpsc::Sender<PriceUpdate>) {
        loop {
            match self.connect_and_stream(&tx).await {
                Ok(_) => {
                    let _ = tx.send(PriceUpdate::Disconnected).await;
                }
                Err(e) => {
                    let _ = tx.send(PriceUpdate::Error(e.to_string())).await;
                    let _ = tx.send(PriceUpdate::Disconnected).await;
                }
            }

            // Wait before reconnecting
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_stream(&self, tx: &mpsc::Sender<PriceUpdate>) -> anyhow::Result<()> {
        // Build combined stream URL: wss://stream.binance.com:9443/stream?streams=btcusdt@ticker/ethusdt@ticker
        let streams: Vec<String> = self
            .pairs
            .iter()
            .map(|p| format!("{}@ticker", p.to_lowercase()))
            .collect();
        let streams_param = streams.join("/");
        let url = format!("{}?streams={}", BINANCE_WS_URL, streams_param);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send connected status
        tx.send(PriceUpdate::Connected).await?;

        // Process incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Some(update) = self.parse_message(&text) {
                        if tx.send(update).await.is_err() {
                            break;
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(Message::Ping(data)) => {
                    let _ = write.send(Message::Pong(data)).await;
                }
                Err(e) => {
                    tx.send(PriceUpdate::Error(e.to_string())).await?;
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn parse_message(&self, text: &str) -> Option<PriceUpdate> {
        let msg: StreamMessage = serde_json::from_str(text).ok()?;
        let data = msg.data;

        let price: f64 = data.close_price.parse().ok()?;
        let open_24h: f64 = data.open_price.parse().ok().unwrap_or(0.0);
        let high_24h: f64 = data.high_price.parse().ok().unwrap_or(0.0);
        let low_24h: f64 = data.low_price.parse().ok().unwrap_or(0.0);
        let volume_base: f64 = data.base_volume.parse().ok().unwrap_or(0.0);
        let volume_quote: f64 = data.quote_volume.parse().ok().unwrap_or(0.0);
        let change_24h: f64 = data.price_change_percent.parse().ok().unwrap_or(0.0);

        // Extract symbol (e.g., "BTCUSDT" -> "BTC")
        let symbol = data.symbol.trim_end_matches("USDT").to_string();

        Some(PriceUpdate::Ticker {
            symbol,
            price,
            change_24h,
            volume_24h_usd: volume_quote, // Quote volume is in USDT
            volume_24h_base: volume_base,
            high_24h,
            low_24h,
        })
    }
}

/// Fetch historical candle data from Binance REST API
/// Returns candles in chronological order (oldest first)
pub async fn fetch_candles(symbol: &str, interval: &str) -> anyhow::Result<Vec<Candle>> {
    let url = format!(
        "{}/api/v3/klines?symbol={}&interval={}&limit=300",
        BINANCE_REST_URL, symbol, interval
    );

    let resp = reqwest::get(&url).await?;
    let data: Vec<Vec<serde_json::Value>> = resp.json().await?;

    // Binance returns: [open_time, open, high, low, close, volume, close_time, quote_volume, trades, ...]
    let candles: Vec<Candle> = data
        .iter()
        .filter_map(|c| {
            if c.len() < 6 {
                return None;
            }
            Some(Candle {
                time: c[0].as_i64().unwrap_or(0) / 1000, // Convert ms to seconds
                open: parse_string_number(&c[1]),
                high: parse_string_number(&c[2]),
                low: parse_string_number(&c[3]),
                close: parse_string_number(&c[4]),
                volume: parse_string_number(&c[5]),
            })
        })
        .collect();

    // Binance returns in chronological order already
    Ok(candles)
}

/// Map TimeWindow granularity to Binance interval string
pub fn granularity_to_interval(granularity: u32) -> &'static str {
    match granularity {
        900 => "15m",
        3600 => "1h",
        21600 => "6h",
        86400 => "1d",
        _ => "1h",
    }
}

/// Parse a JSON string value as f64
fn parse_string_number(val: &serde_json::Value) -> f64 {
    match val {
        serde_json::Value::String(s) => s.parse().unwrap_or(0.0),
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        _ => 0.0,
    }
}
