//! News API client for fetching cryptocurrency news from NewsData.io

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

const NEWSDATA_URL: &str = "https://newsdata.io/api/1/crypto";

/// A news article
#[derive(Debug, Clone)]
pub struct NewsArticle {
    pub title: String,
    pub source: String,
    pub published_at: i64, // Unix timestamp
    pub description: String,
}

// NewsData.io API response structures
#[derive(Deserialize)]
struct NewsDataResponse {
    results: Option<Vec<NewsDataArticle>>,
}

#[derive(Deserialize)]
struct NewsDataArticle {
    title: Option<String>,
    source_name: Option<String>,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    description: Option<String>,
}

/// Fetch news from NewsData.io API
pub async fn fetch_newsdata_news(coins: &[String]) -> anyhow::Result<Vec<NewsArticle>> {
    let api_key = env::var("NEWSDATA_API_KEY")?;

    // Build coin parameter from selected coins (lowercase)
    let coin_param = if coins.is_empty() {
        "btc,eth,sol".to_string()
    } else {
        coins
            .iter()
            .map(|c| c.to_lowercase())
            .collect::<Vec<_>>()
            .join(",")
    };

    let url = format!("{}?apikey={}&coin={}", NEWSDATA_URL, api_key, coin_param);

    let resp = reqwest::get(&url).await?;
    let data: NewsDataResponse = resp.json().await?;

    let news = data
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(|a| {
            let title = a.title?;
            let source = a.source_name.unwrap_or_else(|| "Unknown".to_string());
            let published_at = a
                .pub_date
                .and_then(|d| parse_newsdata_datetime(&d))
                .unwrap_or(0);

            Some(NewsArticle {
                title,
                source,
                published_at,
                description: a.description.unwrap_or_default(),
            })
        })
        .collect();

    Ok(news)
}

/// Fetch news (wrapper for consistency)
pub async fn fetch_all_news(coins: &[String]) -> Vec<NewsArticle> {
    match fetch_newsdata_news(coins).await {
        Ok(articles) => articles,
        Err(e) => {
            eprintln!("NewsData news fetch failed: {}", e);
            Vec::new()
        }
    }
}

/// Check if news API key is configured
pub fn has_api_keys() -> bool {
    env::var("NEWSDATA_API_KEY").is_ok()
}

/// Parse NewsData datetime format to Unix timestamp
/// Format: "2024-01-15 10:30:00"
fn parse_newsdata_datetime(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.trim().split(' ').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<i64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<i64> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();

    if date_parts.len() < 3 || time_parts.len() < 3 {
        return None;
    }

    let year = date_parts[0];
    let month = date_parts[1];
    let day = date_parts[2];
    let hour = time_parts[0];
    let minute = time_parts[1];
    let second = time_parts[2];

    let days_since_epoch =
        (year - 1970) * 365 + (year - 1969) / 4 + days_before_month(month as u32) + day - 1;

    Some(days_since_epoch * 86400 + hour * 3600 + minute * 60 + second)
}

/// Days before a given month (1-indexed)
fn days_before_month(month: u32) -> i64 {
    match month {
        1 => 0,
        2 => 31,
        3 => 59,
        4 => 90,
        5 => 120,
        6 => 151,
        7 => 181,
        8 => 212,
        9 => 243,
        10 => 273,
        11 => 304,
        12 => 334,
        _ => 0,
    }
}

/// Format a timestamp as relative time (e.g., "5 min ago")
pub fn format_relative_time(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let diff = now - timestamp;

    if diff < 0 {
        return "just now".to_string();
    }

    if diff < 60 {
        return "just now".to_string();
    }

    if diff < 3600 {
        let mins = diff / 60;
        return format!("{} min ago", mins);
    }

    if diff < 86400 {
        let hours = diff / 3600;
        return format!("{} hr ago", hours);
    }

    let days = diff / 86400;
    if days == 1 {
        "yesterday".to_string()
    } else {
        format!("{} days ago", days)
    }
}
