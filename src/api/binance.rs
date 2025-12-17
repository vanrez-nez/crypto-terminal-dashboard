use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::env;
use std::sync::RwLock;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::{Candle, PriceUpdate};

/// Get the appropriate Binance WebSocket URL based on environment
fn get_binance_ws_url() -> String {
    if env::var("BINANCE_USE_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true"
    {
        "wss://stream.testnet.binance.vision/stream".to_string()
    } else {
        "wss://stream.binance.com:9443/stream".to_string()
    }
}

/// Get the appropriate Binance REST API URL based on environment
fn get_binance_rest_url() -> String {
    if env::var("BINANCE_USE_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true"
    {
        "https://testnet.binance.vision".to_string()
    } else {
        "https://api.binance.com".to_string()
    }
}

/// Check if testnet mode is enabled (for logging/debugging)
pub fn is_testnet_mode() -> bool {
    env::var("BINANCE_USE_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true"
}

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

/// Binance kline data from WebSocket
#[derive(Deserialize, Debug)]
struct KlineData {
    #[serde(rename = "t")]
    start_time: i64,
    #[serde(rename = "o")]
    open: String,
    #[serde(rename = "h")]
    high: String,
    #[serde(rename = "l")]
    low: String,
    #[serde(rename = "c")]
    close: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "x")]
    is_closed: bool,
}

#[derive(Deserialize, Debug)]
struct KlineStreamData {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "k")]
    kline: KlineData,
}

/// Raw stream message for initial parsing
#[derive(Deserialize, Debug)]
struct RawStreamMessage {
    stream: String,
    data: serde_json::Value,
}

pub struct BinanceProvider {
    pairs: Vec<String>,
    current_interval: RwLock<String>,
}

impl BinanceProvider {
    pub fn new(pairs: Vec<String>, initial_interval: &str) -> Self {
        Self {
            pairs,
            current_interval: RwLock::new(initial_interval.to_string()),
        }
    }

    /// Run the WebSocket connection and send updates through the channel
    pub async fn run(self, tx: mpsc::Sender<PriceUpdate>, mut interval_rx: mpsc::Receiver<String>) {
        loop {
            match self.connect_and_stream(&tx, &mut interval_rx).await {
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

    async fn connect_and_stream(
        &self,
        tx: &mpsc::Sender<PriceUpdate>,
        interval_rx: &mut mpsc::Receiver<String>,
    ) -> anyhow::Result<()> {
        // Build combined stream: ticker + kline for all pairs
        let current = self.current_interval.read().unwrap().clone();
        let streams: Vec<String> = self
            .pairs
            .iter()
            .flat_map(|p| {
                let lower = p.to_lowercase();
                vec![
                    format!("{}@ticker", lower),
                    format!("{}@kline_{}", lower, current),
                ]
            })
            .collect();
        let streams_param = streams.join("/");
        let url = format!("{}?streams={}", get_binance_ws_url(), streams_param);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send connected status
        tx.send(PriceUpdate::Connected).await?;

        // Process incoming messages and interval changes
        loop {
            tokio::select! {
                // Handle WebSocket messages
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Some(update) = self.parse_message(&text) {
                                if tx.send(update).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => break,
                        Some(Ok(Message::Ping(data))) => {
                            let _ = write.send(Message::Pong(data)).await;
                        }
                        Some(Err(e)) => {
                            tx.send(PriceUpdate::Error(e.to_string())).await?;
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                }

                // Handle interval change requests
                new_interval = interval_rx.recv() => {
                    if let Some(interval) = new_interval {
                        let old_interval = self.current_interval.read().unwrap().clone();

                        // Unsubscribe from old kline streams
                        let unsubscribe: Vec<String> = self
                            .pairs
                            .iter()
                            .map(|p| format!("{}@kline_{}", p.to_lowercase(), old_interval))
                            .collect();

                        let unsub_msg = serde_json::json!({
                            "method": "UNSUBSCRIBE",
                            "params": unsubscribe,
                            "id": 1
                        });

                        // Subscribe to new kline streams
                        let subscribe: Vec<String> = self
                            .pairs
                            .iter()
                            .map(|p| format!("{}@kline_{}", p.to_lowercase(), interval))
                            .collect();

                        let sub_msg = serde_json::json!({
                            "method": "SUBSCRIBE",
                            "params": subscribe,
                            "id": 2
                        });

                        println!("[DEBUG] Unsubscribing from: {:?}", unsubscribe);
                        println!("[DEBUG] Subscribing to: {:?}", subscribe);

                        write.send(Message::Text(unsub_msg.to_string())).await?;
                        write.send(Message::Text(sub_msg.to_string())).await?;

                        // Update current interval after successful subscribe
                        *self.current_interval.write().unwrap() = interval.clone();
                        println!("[DEBUG] Interval updated to: {}", interval);
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_message(&self, text: &str) -> Option<PriceUpdate> {
        // Parse raw message to check stream type
        let raw: RawStreamMessage = serde_json::from_str(text).ok()?;

        // Log the stream for debugging
        println!("[DEBUG] Received message on stream: {}", raw.stream);

        // Determine message type based on stream name
        if raw.stream.contains("@ticker") {
            // Parse as ticker message
            let data: TickerData = serde_json::from_value(raw.data).ok()?;

            let price: f64 = data.close_price.parse().ok()?;
            let _open_24h: f64 = data.open_price.parse().ok().unwrap_or(0.0);
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
        } else if raw.stream.contains("@kline_") {
            // Parse as kline message
            let data: KlineStreamData = serde_json::from_value(raw.data).ok()?;
            let k = data.kline;

            println!("[DEBUG] Kline message: symbol={}, time={}, is_closed={}", data.symbol, k.start_time, k.is_closed);

            let candle = Candle {
                time: k.start_time / 1000, // Convert ms to seconds
                open: parse_string_number(&serde_json::Value::String(k.open)),
                high: parse_string_number(&serde_json::Value::String(k.high)),
                low: parse_string_number(&serde_json::Value::String(k.low)),
                close: parse_string_number(&serde_json::Value::String(k.close)),
                volume: parse_string_number(&serde_json::Value::String(k.volume)),
            };

            let symbol = data.symbol.trim_end_matches("USDT").to_string();

            Some(PriceUpdate::Kline {
                symbol,
                candle,
                is_closed: k.is_closed,
            })
        } else {
            // Unknown stream type
            println!("[WARN] Unknown stream type: {}", raw.stream);
            None
        }
    }
}

/// Fetch historical candle data from Binance REST API
/// Returns candles in chronological order (oldest first)
pub async fn fetch_candles(symbol: &str, interval: &str) -> anyhow::Result<Vec<Candle>> {
    let url = format!(
        "{}/api/v3/klines?symbol={}&interval={}&limit=300",
        get_binance_rest_url(),
        symbol,
        interval
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
        14400 => "4h",
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
