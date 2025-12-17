//! Binance Margin API integration with authenticated requests

use anyhow::Result;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the appropriate Binance API URL based on environment
fn get_binance_api_url() -> String {
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

/// Margin position with actual API data
#[derive(Debug, Clone)]
pub struct MarginPosition {
    pub asset: String,
    pub borrowed: f64,
    pub free: f64,
    pub interest: f64,
    pub locked: f64,
    pub net_asset: f64,

    // Market data
    pub current_price: f64,
    pub total_value_usd: f64,
    pub borrowed_value_usd: f64,
    pub net_value_usd: f64,
}

/// Margin account summary
#[derive(Debug, Clone)]
pub struct MarginAccount {
    pub margin_level: f64,
    pub total_asset_usd: f64,
    pub total_liability_usd: f64,
    pub total_net_usd: f64,
    pub positions: Vec<MarginPosition>,
    pub account_type: String, // "Cross Margin"
}

/// Binance API response structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceMarginAccount {
    borrow_enabled: bool,
    margin_level: String,
    total_asset_of_btc: String,
    total_liability_of_btc: String,
    total_net_asset_of_btc: String,
    user_assets: Vec<BinanceUserAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BinanceUserAsset {
    asset: String,
    borrowed: String,
    free: String,
    interest: String,
    locked: String,
    net_asset: String,
}

/// Sign a request using HMAC-SHA256
fn sign_request(query_string: &str, secret: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(query_string.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Parse string to f64, defaulting to 0.0 on error
fn parse_f64(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

/// Fetch current prices for assets from Binance ticker
async fn fetch_asset_prices(assets: &[String]) -> Result<std::collections::HashMap<String, f64>> {
    let mut prices = std::collections::HashMap::new();

    // Stablecoins are always 1.0 USD
    let stablecoins = ["USDT", "BUSD", "USDC", "TUSD", "USDP", "DAI"];

    // Fetch ticker prices for all trading pairs
    let url = format!("{}/api/v3/ticker/price", get_binance_api_url());
    let response: Vec<TickerPrice> = reqwest::get(&url).await?.json().await?;

    // Map asset to USDT price
    for asset in assets {
        // Stablecoins are always 1.0
        if stablecoins.contains(&asset.as_str()) {
            prices.insert(asset.clone(), 1.0);
            continue;
        }

        // Try to find {ASSET}USDT pair
        let symbol = format!("{}USDT", asset);
        if let Some(ticker) = response.iter().find(|t| t.symbol == symbol) {
            if let Ok(price) = ticker.price.parse::<f64>() {
                prices.insert(asset.clone(), price);
            }
        }
    }

    Ok(prices)
}

#[derive(Debug, Deserialize)]
struct TickerPrice {
    symbol: String,
    price: String,
}

/// Calculate position metrics from raw API data
fn calculate_position_metrics(
    asset: &BinanceUserAsset,
    current_price: f64,
) -> MarginPosition {
    let borrowed = parse_f64(&asset.borrowed);
    let free = parse_f64(&asset.free);
    let interest = parse_f64(&asset.interest);
    let locked = parse_f64(&asset.locked);
    let net_asset = parse_f64(&asset.net_asset);

    // Total holdings in this asset
    let total = free + locked;

    // USD values
    let total_value_usd = total * current_price;
    let borrowed_value_usd = (borrowed + interest) * current_price;
    let net_value_usd = net_asset * current_price;

    MarginPosition {
        asset: asset.asset.clone(),
        borrowed,
        free,
        interest,
        locked,
        net_asset,
        current_price,
        total_value_usd,
        borrowed_value_usd,
        net_value_usd,
    }
}

/// Fetch margin account data from Binance API
pub async fn fetch_margin_account(api_key: &str, api_secret: &str) -> Result<MarginAccount> {
    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();

    // Build query string
    let query_string = format!("timestamp={}&recvWindow=5000", timestamp);

    // Sign the request
    let signature = sign_request(&query_string, api_secret);

    // Build final URL
    let url = format!(
        "{}/sapi/v1/margin/account?{}&signature={}",
        get_binance_api_url(),
        query_string,
        signature
    );

    // Make authenticated request
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?;

    // Check for errors
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Binance API error {}: {}",
            status,
            body
        ));
    }

    // Parse response
    let data: BinanceMarginAccount = response.json().await?;

    // Parse account-level metrics
    let margin_level = parse_f64(&data.margin_level);

    // Get list of assets with any balance (borrowed or owned)
    let active_assets: Vec<String> = data
        .user_assets
        .iter()
        .filter(|a| {
            let total = parse_f64(&a.free) + parse_f64(&a.locked);
            let borrowed = parse_f64(&a.borrowed);
            total > 0.0001 || borrowed > 0.0001
        })
        .map(|a| a.asset.clone())
        .collect();

    // Fetch current prices for active assets
    let prices = fetch_asset_prices(&active_assets).await?;

    // Calculate position metrics for each asset
    let mut positions: Vec<MarginPosition> = data
        .user_assets
        .iter()
        .filter(|a| {
            let total = parse_f64(&a.free) + parse_f64(&a.locked);
            let borrowed = parse_f64(&a.borrowed);
            total > 0.0001 || borrowed > 0.0001
        })
        .filter_map(|asset| {
            let current_price = prices.get(&asset.asset).copied().unwrap_or(0.0);
            if current_price > 0.0 {
                Some(calculate_position_metrics(asset, current_price))
            } else {
                None
            }
        })
        .collect();

    // Calculate USD totals
    let total_asset_usd: f64 = positions.iter().map(|p| p.total_value_usd).sum();
    let total_liability_usd: f64 = positions.iter().map(|p| p.borrowed_value_usd).sum();
    let total_net_usd: f64 = positions.iter().map(|p| p.net_value_usd).sum();

    // Sort positions by net value (largest first)
    positions.sort_by(|a, b| b.net_value_usd.abs().partial_cmp(&a.net_value_usd.abs()).unwrap());

    Ok(MarginAccount {
        margin_level,
        total_asset_usd,
        total_liability_usd,
        total_net_usd,
        positions,
        account_type: "Cross Margin".to_string(),
    })
}
