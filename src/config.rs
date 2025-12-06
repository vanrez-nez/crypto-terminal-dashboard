use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::theme::Theme;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Option<ThemeConfig>,
    #[serde(default)]
    pub api: Option<ApiConfig>,
    #[serde(default)]
    pub pairs: Option<Vec<String>>,
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
        self.colors.get(key).map(|s| s.as_str()).filter(|s| !s.is_empty())
    }
}

#[derive(Deserialize)]
struct RawConfig {
    #[serde(default)]
    theme: Option<ThemeConfig>,
    #[serde(default)]
    api: Option<ApiConfig>,
    #[serde(default)]
    pairs: Option<Vec<String>>,
}

impl Config {
    pub fn load(path: &str) -> Self {
        if !Path::new(path).exists() {
            return Self::default();
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        match serde_json::from_str::<RawConfig>(&content) {
            Ok(raw) => Self {
                theme: raw.theme,
                api: raw.api,
                pairs: raw.pairs,
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

    pub fn build_theme(&self) -> Theme {
        match &self.theme {
            Some(config) => Theme::from_config(config),
            None => Theme::default(),
        }
    }
}
