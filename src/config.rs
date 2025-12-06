use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::theme::Theme;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Option<ThemeColors>,
    #[serde(default)]
    pub api: Option<ApiConfig>,
    #[serde(default)]
    pub pairs: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
pub struct ApiConfig {
    pub provider: String,
}

#[derive(Deserialize, Default)]
pub struct ThemeColors {
    #[serde(default)]
    pub foreground: String,
    #[serde(default, rename = "foreground.muted")]
    pub foreground_muted: String,
    #[serde(default, rename = "foreground.inactive")]
    pub foreground_inactive: String,
    #[serde(default)]
    pub accent: String,
    #[serde(default, rename = "accent.secondary")]
    pub accent_secondary: String,
    #[serde(default)]
    pub positive: String,
    #[serde(default)]
    pub negative: String,
    #[serde(default)]
    pub neutral: String,
    #[serde(default, rename = "selection.background")]
    pub selection_background: String,
    #[serde(default, rename = "statusBar.live")]
    pub status_live: String,
}

#[derive(Deserialize)]
struct RawConfig {
    #[serde(default)]
    theme: Option<RawThemeConfig>,
    #[serde(default)]
    api: Option<ApiConfig>,
    #[serde(default)]
    pairs: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawThemeConfig {
    colors: ThemeColors,
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

        let raw: RawConfig = match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        Self {
            theme: raw.theme.map(|t| t.colors),
            api: raw.api,
            pairs: raw.pairs,
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
            Some(colors) => Theme::from_colors(colors),
            None => Theme::default(),
        }
    }
}
