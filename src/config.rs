use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::notifications::NotificationRule;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub api: Option<ApiConfig>,
    #[serde(default)]
    pub pairs: Option<Vec<String>>,
    #[serde(default)]
    pub notifications: Option<NotificationsConfig>,
}

/// Notification system configuration
#[derive(Deserialize, Clone)]
pub struct NotificationsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub audio_enabled: bool,
    #[serde(default = "default_cooldown")]
    pub cooldown_secs: u64,
    #[serde(default = "default_log_file")]
    pub log_file: String,
    #[serde(default = "default_max_entries")]
    pub max_log_entries: usize,
    #[serde(default)]
    pub rules: Vec<NotificationRule>,
    #[serde(default)]
    pub ticker_tones: TickerTonesConfig,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            audio_enabled: true,
            cooldown_secs: 60,
            log_file: "notifications.json".to_string(),
            max_log_entries: 100,
            rules: Vec::new(),
            ticker_tones: TickerTonesConfig::default(),
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_cooldown() -> u64 {
    60
}
fn default_log_file() -> String {
    "notifications.json".to_string()
}
fn default_max_entries() -> usize {
    100
}

/// Ticker tone configuration - audio feedback for price movements
#[derive(Deserialize, Clone)]
pub struct TickerTonesConfig {
    /// Enable/disable ticker tones (default: false)
    #[serde(default)]
    pub enabled: bool,
    /// Base frequency for UP price movements in Hz (default: 400)
    #[serde(default = "default_base_freq_up")]
    pub base_freq_up: f32,
    /// Base frequency for DOWN price movements in Hz (default: 300)
    #[serde(default = "default_base_freq_down")]
    pub base_freq_down: f32,
    /// Maximum frequency for big UP moves in Hz (default: 1200)
    #[serde(default = "default_max_freq")]
    pub max_freq: f32,
    /// Minimum frequency for big DOWN moves in Hz (default: 150)
    #[serde(default = "default_min_freq")]
    pub min_freq: f32,
    /// Tone duration in milliseconds (default: 50)
    #[serde(default = "default_tone_duration_ms")]
    pub duration_ms: u32,
}

impl Default for TickerTonesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_freq_up: 400.0,
            base_freq_down: 300.0,
            max_freq: 1200.0,
            min_freq: 150.0,
            duration_ms: 50,
        }
    }
}

fn default_base_freq_up() -> f32 {
    400.0
}
fn default_base_freq_down() -> f32 {
    300.0
}
fn default_max_freq() -> f32 {
    1200.0
}
fn default_min_freq() -> f32 {
    150.0
}
fn default_tone_duration_ms() -> u32 {
    50
}

#[derive(Deserialize, Clone)]
pub struct ApiConfig {
    pub provider: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct ThemeConfig {
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

impl ThemeConfig {
    /// Get a color value by key, returns None if not found or empty
    pub fn get(&self, key: &str) -> Option<&str> {
        self.colors
            .get(key)
            .map(|s| s.as_str())
            .filter(|s| !s.is_empty())
    }

    /// Load a theme by name from the themes directory
    pub fn load_by_name(name: &str) -> Option<Self> {
        let filename = format!("{}.json", name);

        // Try themes directory next to executable
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let theme_path = exe_dir.join("themes").join(&filename);
                if let Some(config) = Self::load_from_path(&theme_path) {
                    return Some(config);
                }
            }
        }

        // Try themes directory in current working directory
        let cwd_path = PathBuf::from("themes").join(&filename);
        if let Some(config) = Self::load_from_path(&cwd_path) {
            return Some(config);
        }

        None
    }

    fn load_from_path(path: &PathBuf) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let raw: RawThemeFile = serde_json::from_str(&content).ok()?;
        if raw.colors.is_empty() {
            None
        } else {
            Some(Self { colors: raw.colors })
        }
    }
}

/// Raw theme file structure (themes/*.json) - just colors directly
#[derive(Deserialize)]
struct RawThemeFile {
    #[serde(default)]
    colors: HashMap<String, String>,
}

#[derive(Deserialize)]
struct RawConfig {
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    api: Option<ApiConfig>,
    #[serde(default)]
    pairs: Option<Vec<String>>,
    #[serde(default)]
    notifications: Option<NotificationsConfig>,
}

impl Config {
    /// Find config file path. Search order:
    /// 1. Next to the executable
    /// 2. Current working directory
    fn find_config_path(filename: &str) -> Option<PathBuf> {
        // Try next to the executable first
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let config_path = exe_dir.join(filename);
                if config_path.exists() {
                    return Some(config_path);
                }
            }
        }

        // Fall back to current working directory
        let cwd_path = PathBuf::from(filename);
        if cwd_path.exists() {
            return Some(cwd_path);
        }

        None
    }

    pub fn load(filename: &str) -> Self {
        let path = match Self::find_config_path(filename) {
            Some(p) => p,
            None => return Self::default(),
        };

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        match serde_json::from_str::<RawConfig>(&content) {
            Ok(raw) => Self {
                theme: raw.theme,
                api: raw.api,
                pairs: raw.pairs,
                notifications: raw.notifications,
            },
            Err(_) => Self::default(),
        }
    }

    pub fn provider(&self) -> &str {
        self.api
            .as_ref()
            .map(|a| a.provider.as_str())
            .unwrap_or("mock")
    }

    pub fn pairs(&self) -> Vec<String> {
        self.pairs.clone().unwrap_or_else(|| {
            vec![
                "BTC-USD".to_string(),
                "ETH-USD".to_string(),
                "SOL-USD".to_string(),
            ]
        })
    }

    /// Load theme configuration by name, returns None if not found
    pub fn theme_config(&self) -> Option<ThemeConfig> {
        self.theme
            .as_ref()
            .and_then(|name| ThemeConfig::load_by_name(name))
    }

    /// Get notifications config or default
    pub fn notifications_config(&self) -> NotificationsConfig {
        self.notifications.clone().unwrap_or_default()
    }

    /// Check if notifications are enabled
    pub fn notifications_enabled(&self) -> bool {
        self.notifications
            .as_ref()
            .map(|n| n.enabled)
            .unwrap_or(false)
    }

    /// Check if audio alerts are enabled
    pub fn audio_enabled(&self) -> bool {
        self.notifications
            .as_ref()
            .map(|n| n.audio_enabled)
            .unwrap_or(false)
    }

    /// Get the notification log file path
    pub fn log_file(&self) -> String {
        self.notifications
            .as_ref()
            .map(|n| n.log_file.clone())
            .unwrap_or_else(|| "notifications.json".to_string())
    }

    /// Get ticker tones config
    pub fn ticker_tones_config(&self) -> TickerTonesConfig {
        self.notifications
            .as_ref()
            .map(|n| n.ticker_tones.clone())
            .unwrap_or_default()
    }
}
