use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Option<String>,
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

    /// Load theme configuration by name, returns None if not found
    pub fn theme_config(&self) -> Option<ThemeConfig> {
        self.theme.as_ref().and_then(|name| ThemeConfig::load_by_name(name))
    }
}
