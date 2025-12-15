//! Notification rule definitions
//!
//! Rules are configured in config.json and define conditions that trigger notifications.

use serde::{Deserialize, Serialize};

/// Direction for threshold-based rules (RSI, price levels)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdDir {
    Above,
    Below,
}

/// Direction for crossing-based rules (EMA crossings)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrossDir {
    CrossAbove,
    CrossBelow,
}

/// Notification rule types - tagged enum for config.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationRule {
    /// RSI threshold alert (e.g., RSI < 30 = oversold)
    Rsi {
        period: u8,
        threshold: f64,
        direction: ThresholdDir,
        #[serde(default = "default_enabled")]
        enabled: bool,
        /// Custom sound file (e.g., "oversold.wav") in sounds/ directory
        #[serde(default)]
        sound: Option<String>,
    },
    /// Price crosses EMA line
    EmaCross {
        period: u8,
        direction: CrossDir,
        #[serde(default = "default_enabled")]
        enabled: bool,
        /// Custom sound file (e.g., "ema_cross.wav") in sounds/ directory
        #[serde(default)]
        sound: Option<String>,
    },
    /// Price crosses a specific level
    PriceLevel {
        symbol: String,
        price: f64,
        direction: ThresholdDir,
        #[serde(default = "default_enabled")]
        enabled: bool,
        /// Custom sound file (e.g., "price_alert.wav") in sounds/ directory
        #[serde(default)]
        sound: Option<String>,
    },
}

fn default_enabled() -> bool {
    true
}

impl NotificationRule {
    /// Check if this rule is enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            NotificationRule::Rsi { enabled, .. } => *enabled,
            NotificationRule::EmaCross { enabled, .. } => *enabled,
            NotificationRule::PriceLevel { enabled, .. } => *enabled,
        }
    }

    /// Toggle the enabled state of this rule
    pub fn toggle(&mut self) {
        match self {
            NotificationRule::Rsi { enabled, .. } => *enabled = !*enabled,
            NotificationRule::EmaCross { enabled, .. } => *enabled = !*enabled,
            NotificationRule::PriceLevel { enabled, .. } => *enabled = !*enabled,
        }
    }

    /// Get the custom sound file for this rule (if any)
    pub fn sound(&self) -> Option<&str> {
        match self {
            NotificationRule::Rsi { sound, .. } => sound.as_deref(),
            NotificationRule::EmaCross { sound, .. } => sound.as_deref(),
            NotificationRule::PriceLevel { sound, .. } => sound.as_deref(),
        }
    }

    /// Get a human-readable description of this rule
    pub fn description(&self) -> String {
        match self {
            NotificationRule::Rsi {
                period,
                threshold,
                direction,
                ..
            } => {
                let dir = match direction {
                    ThresholdDir::Above => ">",
                    ThresholdDir::Below => "<",
                };
                format!("RSI({}) {} {:.0}", period, dir, threshold)
            }
            NotificationRule::EmaCross {
                period, direction, ..
            } => {
                let dir = match direction {
                    CrossDir::CrossAbove => "Cross Above",
                    CrossDir::CrossBelow => "Cross Below",
                };
                format!("EMA({}) {}", period, dir)
            }
            NotificationRule::PriceLevel {
                symbol,
                price,
                direction,
                ..
            } => {
                let dir = match direction {
                    ThresholdDir::Above => ">",
                    ThresholdDir::Below => "<",
                };
                format!("{} {} ${:.0}", symbol, dir, price)
            }
        }
    }

    /// Get a unique key for this rule (for cooldown tracking)
    pub fn key(&self) -> String {
        match self {
            NotificationRule::Rsi {
                period,
                threshold,
                direction,
                ..
            } => format!("rsi_{}_{:?}_{}", period, direction, *threshold as i64),
            NotificationRule::EmaCross {
                period, direction, ..
            } => format!("ema_cross_{}_{:?}", period, direction),
            NotificationRule::PriceLevel {
                symbol,
                price,
                direction,
                ..
            } => format!("price_{}_{}_{:?}", symbol, *price as i64, direction),
        }
    }
}
