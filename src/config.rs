use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

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
