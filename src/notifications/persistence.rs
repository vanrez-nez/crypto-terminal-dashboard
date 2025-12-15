//! Notification persistence - load/save to JSON file

use super::notification::Notification;
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_LOG_FILE: &str = "notifications.json";

/// Find the log file path (same logic as config.json)
fn find_log_path(filename: &str) -> PathBuf {
    let filename = if filename.is_empty() {
        DEFAULT_LOG_FILE
    } else {
        filename
    };

    // Try next to the executable first
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let log_path = exe_dir.join(filename);
            // Use this path even if file doesn't exist yet (for writing)
            return log_path;
        }
    }

    // Fall back to current working directory
    PathBuf::from(filename)
}

/// Load notifications from JSON file
pub fn load_notifications(filename: &str) -> Vec<Notification> {
    let path = find_log_path(filename);

    if !path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(notifications) => notifications,
            Err(e) => {
                eprintln!("Failed to parse notifications log: {}", e);
                Vec::new()
            }
        },
        Err(e) => {
            eprintln!("Failed to read notifications log: {}", e);
            Vec::new()
        }
    }
}

/// Save notifications to JSON file
pub fn save_notifications(notifications: &[Notification], filename: &str) {
    let path = find_log_path(filename);

    match serde_json::to_string_pretty(notifications) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                eprintln!("Failed to write notifications log: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize notifications: {}", e);
        }
    }
}
