//! Notification instance - a triggered alert

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Notification severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// A notification instance - represents a triggered alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: u64,
    pub timestamp: u64,
    pub symbol: String,
    pub message: String,
    pub rule_type: String,
    pub severity: Severity,
    pub read: bool,
    /// Custom sound file to play (from rule config)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
}

impl Notification {
    /// Create a new notification with current timestamp
    pub fn new(
        symbol: &str,
        message: &str,
        rule_type: &str,
        severity: Severity,
        sound: Option<&str>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: timestamp * 1000 + (rand_u64() % 1000), // Simple unique ID
            timestamp,
            symbol: symbol.to_string(),
            message: message.to_string(),
            rule_type: rule_type.to_string(),
            severity,
            read: false,
            sound: sound.map(|s| s.to_string()),
        }
    }

    /// Format timestamp as HH:MM
    pub fn time_str(&self) -> String {
        let secs = self.timestamp % 86400; // seconds in day
        let hours = (secs / 3600) % 24;
        let minutes = (secs % 3600) / 60;
        format!("{:02}:{:02}", hours, minutes)
    }
}

/// Simple pseudo-random number (no external deps)
fn rand_u64() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_nanos() as u64
}
