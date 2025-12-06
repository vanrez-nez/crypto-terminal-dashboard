use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::{Candle, PriceUpdate};

const COINBASE_WS_URL: &str = "wss://ws-feed.exchange.coinbase.com";
const COINBASE_REST_URL: &str = "https://api.exchange.coinbase.com";

#[derive(Serialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    product_ids: Vec<String>,
    channels: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct TickerMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    product_id: String,
    #[serde(default)]
    price: Option<String>,
    #[serde(default)]
    open_24h: Option<String>,
    #[serde(default)]
    volume_24h: Option<String>,
    #[serde(default)]
    high_24h: Option<String>,
    #[serde(default)]
    low_24h: Option<String>,
}

pub struct CoinbaseProvider {
    pairs: Vec<String>,
}

impl CoinbaseProvider {
    pub fn new(pairs: Vec<String>) -> Self {
        Self { pairs }
    }

    /// Run the WebSocket connection and send updates through the channel
    pub async fn run(self, tx: mpsc::Sender<PriceUpdate>) {
        loop {
            match self.connect_and_stream(&tx).await {
                Ok(_) => {
                    // Connection closed normally
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
        let (ws_stream, _) = connect_async(COINBASE_WS_URL).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send connected status
        tx.send(PriceUpdate::Connected).await?;

        // Subscribe to ticker channel
        let subscribe = SubscribeMessage {
            msg_type: "subscribe".to_string(),
            product_ids: self.pairs.clone(),
            channels: vec!["ticker".to_string()],
        };

        let subscribe_msg = serde_json::to_string(&subscribe)?;
        write.send(Message::Text(subscribe_msg)).await?;

        // Process incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Some(update) = self.parse_message(&text) {
                        if tx.send(update).await.is_err() {
                            // Receiver dropped, exit
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
        let msg: TickerMessage = serde_json::from_str(text).ok()?;

        if msg.msg_type != "ticker" {
            return None;
        }

        let price: f64 = msg.price?.parse().ok()?;
        let open_24h: f64 = msg.open_24h.and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let volume_24h_base: f64 = msg.volume_24h.and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let volume_24h_usd = volume_24h_base * price;
        let high_24h = msg.high_24h.and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let low_24h = msg.low_24h.and_then(|v| v.parse().ok()).unwrap_or(0.0);

        // Calculate 24h percentage change
        let change_24h = if open_24h > 0.0 {
            ((price - open_24h) / open_24h) * 100.0
        } else {
            0.0
        };

        // Extract symbol from product_id (e.g., "BTC-USD" -> "BTC")
        let symbol = msg.product_id.split('-').next()?.to_string();

        Some(PriceUpdate::Ticker {
            symbol,
            price,
            change_24h,
            volume_24h_usd,
            volume_24h_base,
            high_24h,
            low_24h,
        })
    }
}

/// Fetch historical candle data from Coinbase REST API
/// Returns candles in chronological order (oldest first)
pub async fn fetch_candles(product_id: &str, granularity: u32) -> anyhow::Result<Vec<Candle>> {
    let url = format!(
        "{}/products/{}/candles?granularity={}",
        COINBASE_REST_URL, product_id, granularity
    );

    let resp = reqwest::get(&url).await?;
    let data: Vec<Vec<serde_json::Value>> = resp.json().await?;

    // Coinbase returns [time, low, high, open, close, volume] arrays
    // Data comes in reverse chronological order, so we reverse it
    let mut candles: Vec<Candle> = data
        .iter()
        .filter_map(|c| {
            if c.len() < 6 {
                return None;
            }
            Some(Candle {
                time: c[0].as_i64().unwrap_or(0),
                low: parse_number(&c[1]),
                high: parse_number(&c[2]),
                open: parse_number(&c[3]),
                close: parse_number(&c[4]),
                volume: parse_number(&c[5]),
            })
        })
        .collect();

    // Reverse to get chronological order (oldest first)
    candles.reverse();
    Ok(candles)
}

/// Parse a JSON value as f64 (handles both string and number formats)
fn parse_number(val: &serde_json::Value) -> f64 {
    match val {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}
