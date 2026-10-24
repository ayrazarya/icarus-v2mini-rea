//! Live telemetry export to ~/.cache/icarus-v2mini/battery.json for Waybar and Polybar.

use crate::device::BatteryStatus;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BatteryExport {
    /// Formatted status string (e.g. "75%" or "Offline").
    pub text: String,
    /// Detailed tooltip text showing voltage, mode, and profile.
    pub tooltip: String,
    /// Battery level percentage (0 to 100).
    pub percentage: u8,
    /// Charging status flag.
    pub charging: bool,
    /// Measured voltage in millivolts.
    pub voltage_mv: u16,
    /// CSS class tag for custom bar styling ("normal", "warning", "critical", "charging", "offline").
    pub class: String,
    /// Currently active onboard hardware profile (1 to 5).
    pub active_profile: u8,
}

/// Path to store battery export json (`~/.cache/icarus-v2mini/battery.json`).
pub fn get_battery_cache_path() -> PathBuf {
    let cache_dir = dirs_fallback_cache();
    let dir = cache_dir.join("icarus-v2mini");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir.join("battery.json")
}

/// Path to default user config file (`~/.config/icarus-v2mini/config.toml`).
pub fn get_default_config_path() -> PathBuf {
    let config_dir = dirs_fallback_config();
    let dir = config_dir.join("icarus-v2mini");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir.join("config.toml")
}

fn dirs_fallback_config() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        PathBuf::from("/tmp")
    }
}

fn dirs_fallback_cache() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache")
    } else {
        PathBuf::from("/tmp")
    }
}

/// Formats and exports live battery status to cache JSON for Waybar/Polybar.
pub fn export_battery_status(status: &BatteryStatus, active_profile: u8) -> Result<()> {
    let clamped_level = status.level.min(100);
    let class = if status.is_charging {
        "charging"
    } else if clamped_level <= 15 {
        "critical"
    } else if clamped_level <= 30 {
        "warning"
    } else {
        "normal"
    };

    let export = BatteryExport {
        text: format!("{}%", clamped_level),
        tooltip: format!(
            "ICARUS V2 MINI: {}% ({} mV) [{}]",
            clamped_level,
            status.voltage_mv,
            if status.is_charging {
                "Charging"
            } else {
                "Battery"
            }
        ),
        percentage: clamped_level,
        charging: status.is_charging,
        voltage_mv: status.voltage_mv,
        class: class.to_string(),
        active_profile,
    };

    let json = serde_json::to_string_pretty(&export)?;
    write_json_atomically(&get_battery_cache_path(), &json)?;
    Ok(())
}

/// Exports offline status to cache JSON when mouse is disconnected or asleep.
pub fn export_offline_status(active_profile: u8) -> Result<()> {
    let export = BatteryExport {
        text: "Offline".to_string(),
        tooltip: "ICARUS V2 MINI: Disconnected / Asleep".to_string(),
        percentage: 0,
        charging: false,
        voltage_mv: 0,
        class: "offline".to_string(),
        active_profile,
    };

    let json = serde_json::to_string_pretty(&export)?;
    write_json_atomically(&get_battery_cache_path(), &json)?;
    Ok(())
}

/// Writes JSON to a temporary sibling file and atomically renames it into place
/// so status bars (Waybar/Polybar) never observe a partially-written file.
fn write_json_atomically(path: &std::path::Path, json: &str) -> Result<()> {
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}
