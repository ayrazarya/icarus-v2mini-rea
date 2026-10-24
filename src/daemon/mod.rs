//! Background daemon for ICARUS V2 MINI mouse.
//!
//! Provides native desktop battery integration (Waybar, Polybar, notifications),
//! real-time auto-profile switching per game/application, and systemd user service management.

pub mod config;
pub mod export;
pub mod notification;
pub mod scanner;
pub mod service;
pub mod systemd;

pub use config::DaemonConfig;
pub use export::{
    BatteryExport, export_battery_status, export_offline_status, get_battery_cache_path,
    get_default_config_path,
};
pub use notification::{check_and_notify_battery, send_desktop_notification};
pub use scanner::{
    AutoProfileCache, detect_active_app, process_auto_profile, process_auto_profile_cached,
};
pub use service::run_daemon;
pub use systemd::install_systemd_user_service;
