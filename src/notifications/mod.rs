//! Notifications module - alerts for price and indicator conditions

pub mod audio;
pub mod manager;
pub mod notification;
pub mod persistence;
pub mod rules;
pub mod ticker_tones;

pub use manager::NotificationManager;
pub use notification::Severity;
pub use rules::NotificationRule;
pub use ticker_tones::process_ticker_tones;
