//! Notification manager - handles rule checking and notification state

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::notification::{Notification, Severity};
use super::rules::{CrossDir, NotificationRule, ThresholdDir};
use crate::mock::CoinData;

const DEFAULT_MAX_NOTIFICATIONS: usize = 100;

/// Manages notification rules and triggered notifications
pub struct NotificationManager {
    pub rules: Vec<NotificationRule>,
    pub notifications: Vec<Notification>,
    pub unread_count: usize,
    pub selected_rule: usize,
    max_notifications: usize,
    cooldown_secs: u64,
    // State tracking for crossing detection
    prev_prices: HashMap<String, f64>,
    prev_ema_positions: HashMap<(String, u8), bool>, // (symbol, period) -> was_above_ema
    prev_rsi_positions: HashMap<(String, u8), bool>, // (symbol, period) -> was_above_threshold
    // Cooldown tracking: rule_key -> last_trigger_timestamp
    cooldowns: HashMap<String, u64>,
}

impl NotificationManager {
    /// Create a new notification manager with the given rules
    pub fn new(rules: Vec<NotificationRule>, cooldown_secs: u64, max_notifications: usize) -> Self {
        Self {
            rules,
            notifications: Vec::new(),
            unread_count: 0,
            selected_rule: 0,
            max_notifications: if max_notifications > 0 {
                max_notifications
            } else {
                DEFAULT_MAX_NOTIFICATIONS
            },
            cooldown_secs,
            prev_prices: HashMap::new(),
            prev_ema_positions: HashMap::new(),
            prev_rsi_positions: HashMap::new(),
            cooldowns: HashMap::new(),
        }
    }

    /// Create with default settings
    pub fn default() -> Self {
        Self::new(Vec::new(), 60, DEFAULT_MAX_NOTIFICATIONS)
    }

    /// Load existing notifications (from persistence)
    pub fn load_notifications(&mut self, notifications: Vec<Notification>) {
        self.notifications = notifications;
        self.unread_count = self.notifications.iter().filter(|n| !n.read).count();
        self.rotate_log();
    }

    /// Check all rules against current coin data, returns new notifications
    /// Only checks rules for coins that are checked/selected
    pub fn check_rules(&mut self, coins: &[CoinData], checked: &[bool]) -> Vec<Notification> {
        let mut new_notifications = Vec::new();
        let now = now_secs();

        // Clone rules to avoid borrow conflict
        let rules: Vec<NotificationRule> = self.rules.clone();

        for (i, coin) in coins.iter().enumerate() {
            // Skip unchecked coins
            if !checked.get(i).copied().unwrap_or(false) {
                continue;
            }

            for rule in &rules {
                if !rule.is_enabled() {
                    continue;
                }

                // Check cooldown
                let rule_key = format!("{}_{}", coin.symbol, rule.key());
                if let Some(&last_trigger) = self.cooldowns.get(&rule_key) {
                    if now - last_trigger < self.cooldown_secs {
                        continue;
                    }
                }

                if let Some(notif) = self.check_single_rule(rule, coin, now, rule.sound()) {
                    self.cooldowns.insert(rule_key, now);
                    new_notifications.push(notif);
                }
            }

            // Update previous state for next check
            self.prev_prices.insert(coin.symbol.clone(), coin.price);
        }

        // Add new notifications
        for notif in &new_notifications {
            self.notifications.push(notif.clone());
            self.unread_count += 1;
        }

        self.rotate_log();
        new_notifications
    }

    /// Check a single rule against a coin
    fn check_single_rule(
        &mut self,
        rule: &NotificationRule,
        coin: &CoinData,
        _now: u64,
        sound: Option<&str>,
    ) -> Option<Notification> {
        match rule {
            NotificationRule::Rsi {
                period,
                threshold,
                direction,
                ..
            } => self.check_rsi_rule(coin, *period, *threshold, *direction, sound),

            NotificationRule::EmaCross {
                period, direction, ..
            } => self.check_ema_cross_rule(coin, *period, *direction, sound),

            NotificationRule::PriceLevel {
                symbol,
                price,
                direction,
                ..
            } => {
                // Only check if symbol matches (strip USDT suffix)
                let coin_base = coin.symbol.trim_end_matches("USDT");
                if coin_base == symbol || coin.symbol == *symbol {
                    self.check_price_level_rule(coin, *price, *direction, sound)
                } else {
                    None
                }
            }
        }
    }

    /// Check RSI threshold rule
    fn check_rsi_rule(
        &mut self,
        coin: &CoinData,
        period: u8,
        threshold: f64,
        direction: ThresholdDir,
        sound: Option<&str>,
    ) -> Option<Notification> {
        let rsi = match period {
            6 => coin.indicators.rsi_6,
            12 => coin.indicators.rsi_12,
            24 => coin.indicators.rsi_24,
            _ => return None,
        };

        let key = (coin.symbol.clone(), period);
        let currently_triggered = match direction {
            ThresholdDir::Below => rsi < threshold,
            ThresholdDir::Above => rsi > threshold,
        };

        // Check if we just crossed the threshold
        let prev_triggered = self.prev_rsi_positions.get(&key).copied().unwrap_or(false);
        self.prev_rsi_positions.insert(key, currently_triggered);

        // Only notify on transition from not-triggered to triggered
        if currently_triggered && !prev_triggered {
            let severity = if rsi < 20.0 || rsi > 80.0 {
                Severity::Critical
            } else if rsi < 30.0 || rsi > 70.0 {
                Severity::Warning
            } else {
                Severity::Info
            };

            let dir_text = match direction {
                ThresholdDir::Below => "dropped below",
                ThresholdDir::Above => "rose above",
            };

            let message = format!(
                "{} RSI({}) {} {:.0} (now {:.1})",
                coin.symbol, period, dir_text, threshold, rsi
            );

            return Some(Notification::new(
                &coin.symbol,
                &message,
                "rsi",
                severity,
                sound,
            ));
        }

        None
    }

    /// Check EMA crossing rule
    fn check_ema_cross_rule(
        &mut self,
        coin: &CoinData,
        period: u8,
        direction: CrossDir,
        sound: Option<&str>,
    ) -> Option<Notification> {
        let ema = match period {
            7 => coin.indicators.ema_7,
            25 => coin.indicators.ema_25,
            99 => coin.indicators.ema_99,
            _ => return None,
        };

        // Skip if EMA is 0 (not calculated yet)
        if ema == 0.0 {
            return None;
        }

        let key = (coin.symbol.clone(), period);
        let currently_above = coin.price > ema;

        let prev_above = self.prev_ema_positions.get(&key).copied();
        self.prev_ema_positions.insert(key, currently_above);

        // Need previous state to detect crossing
        let prev_above = prev_above?;

        let crossed = match direction {
            CrossDir::CrossAbove => !prev_above && currently_above,
            CrossDir::CrossBelow => prev_above && !currently_above,
        };

        if crossed {
            let dir_text = match direction {
                CrossDir::CrossAbove => "crossed above",
                CrossDir::CrossBelow => "crossed below",
            };

            let message = format!(
                "{} {} EMA({}) at ${:.2}",
                coin.symbol, dir_text, period, coin.price
            );

            return Some(Notification::new(
                &coin.symbol,
                &message,
                "ema_cross",
                Severity::Info,
                sound,
            ));
        }

        None
    }

    /// Check price level rule
    fn check_price_level_rule(
        &mut self,
        coin: &CoinData,
        target_price: f64,
        direction: ThresholdDir,
        sound: Option<&str>,
    ) -> Option<Notification> {
        let prev_price = self.prev_prices.get(&coin.symbol).copied()?;

        let crossed = match direction {
            ThresholdDir::Above => prev_price < target_price && coin.price >= target_price,
            ThresholdDir::Below => prev_price > target_price && coin.price <= target_price,
        };

        if crossed {
            let dir_text = match direction {
                ThresholdDir::Above => "broke above",
                ThresholdDir::Below => "fell below",
            };

            let message = format!("{} {} ${:.0}", coin.symbol, dir_text, target_price);

            return Some(Notification::new(
                &coin.symbol,
                &message,
                "price_level",
                Severity::Warning,
                sound,
            ));
        }

        None
    }

    /// Mark all notifications as read
    pub fn mark_all_read(&mut self) {
        for notif in &mut self.notifications {
            notif.read = true;
        }
        self.unread_count = 0;
    }

    /// Toggle the selected rule's enabled state
    pub fn toggle_selected_rule(&mut self) {
        if let Some(rule) = self.rules.get_mut(self.selected_rule) {
            rule.toggle();
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_rule > 0 {
            self.selected_rule -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_rule + 1 < self.rules.len() {
            self.selected_rule += 1;
        }
    }

    /// Rotate log if over max entries
    fn rotate_log(&mut self) {
        if self.notifications.len() > self.max_notifications {
            let excess = self.notifications.len() - self.max_notifications;
            self.notifications.drain(0..excess);
        }
    }

    /// Get notifications slice for display
    pub fn get_notifications(&self) -> &[Notification] {
        &self.notifications
    }

    /// Get rules slice for display
    pub fn get_rules(&self) -> &[NotificationRule] {
        &self.rules
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
