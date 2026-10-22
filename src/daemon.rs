//! Background daemon for ICARUS V2 MINI mouse.
//!
//! Exceeds OEM Windows capabilities by providing:
//! 1. Native Desktop Battery Integration:
//!    - Periodic polling & JSON export to `~/.cache/icarus-v2mini/battery.json` for Waybar, Polybar, i3blocks, Conky.
//!    - Desktop notification warnings (`notify-send`) when battery drops <= 15% or reaches 100% full.
//! 2. Real-Time Auto-Profile Switching:
//!    - Continuously inspects active foreground applications & games (Wayland Hyprland/Sway, X11/XWayland, /proc).
//!    - Automatically switches mouse profiles (DPI, LOD, polling rate, etc.) on-the-fly when launching target games.
//! 3. Systemd User Service Integration (`systemctl --user enable icarus-v2mini`).

use crate::config::AutoProfileRule;
use crate::device::{BatteryStatus, IcarusDevice};

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Status JSON structure designed for Waybar / Polybar / Conky custom modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Configuration options for running the background daemon.
///
/// Follows the builder pattern and encapsulates polling intervals and feature toggles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConfig {
    /// Telemetry polling interval in seconds (default: 10).
    pub interval_secs: u64,
    /// Whether desktop notifications are enabled via `notify-send`.
    pub enable_notify: bool,
    /// Whether auto-profile switching is enabled.
    pub auto_profile: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            interval_secs: 10,
            enable_notify: true,
            auto_profile: true,
        }
    }
}

impl DaemonConfig {
    /// Creates a new `DaemonConfig` with standard default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the polling interval in seconds.
    pub fn with_interval(mut self, secs: u64) -> Self {
        self.interval_secs = secs;
        self
    }

    /// Enables or disables desktop notifications.
    pub fn with_notifications(mut self, enable: bool) -> Self {
        self.enable_notify = enable;
        self
    }

    /// Enables or disables per-app auto-profile switching.
    pub fn with_auto_profile(mut self, enable: bool) -> Self {
        self.auto_profile = enable;
        self
    }
}

/// Send a native desktop notification via `notify-send`.
pub fn send_desktop_notification(summary: &str, body: &str, urgency: &str) {
    let _ = Command::new("notify-send")
        .arg("-a")
        .arg("ICARUS V2 MINI")
        .arg("-u")
        .arg(urgency)
        .arg("-i")
        .arg("input-mouse")
        .arg(summary)
        .arg(body)
        .spawn();
}

/// Detect the active foreground application name or window class.
///
/// Multi-layered architecture:
/// 1. Hyprland Wayland: `hyprctl activewindow -j`
/// 2. Sway Wayland: `swaymsg -t get_tree`
/// 3. X11 / XWayland: `xprop -root _NET_ACTIVE_WINDOW`
/// 4. Process scanner (/proc) for running game processes matching configured rules.
#[allow(clippy::collapsible_if)]
pub fn detect_active_app(rules: &[AutoProfileRule]) -> Option<String> {
    // 1. Try Hyprland
    if let Ok(output) = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(class) = val.get("class").and_then(|v| v.as_str()) {
                    if !class.is_empty() {
                        return Some(class.to_lowercase());
                    }
                }
                if let Some(title) = val.get("title").and_then(|v| v.as_str()) {
                    if !title.is_empty() {
                        return Some(title.to_lowercase());
                    }
                }
            }
        }
    }

    // 2. Try Sway
    if let Ok(output) = Command::new("swaymsg").args(["-t", "get_tree"]).output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(focused) = find_sway_focused(&val) {
                    return Some(focused.to_lowercase());
                }
            }
        }
    }

    // 3. Try X11 / XWayland via xprop
    if let Ok(output) = Command::new("xprop")
        .args(["-root", "_NET_ACTIVE_WINDOW"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(id_part) = s.split('#').nth(1) {
                let win_id = id_part.split_whitespace().next().unwrap_or("");
                if !win_id.is_empty() && win_id != "0x0" {
                    if let Ok(class_out) = Command::new("xprop")
                        .args(["-id", win_id, "WM_CLASS"])
                        .output()
                    {
                        let class_s = String::from_utf8_lossy(&class_out.stdout);
                        if let Some(quoted) = class_s.split('"').nth(1).filter(|s| !s.is_empty()) {
                            return Some(quoted.to_lowercase());
                        }
                    }
                }
            }
        }
    }

    // 4. Linux /proc scanner fallback: Check if target applications/games are currently running
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let comm_file = path.join("comm");
                if let Ok(comm) = fs::read_to_string(comm_file) {
                    let comm_lower = comm.trim().to_lowercase();
                    for rule in rules {
                        let pat = rule.app_pattern.to_lowercase();
                        if comm_lower == pat || comm_lower.contains(&pat) {
                            return Some(comm_lower);
                        }
                    }
                }
            }
        }
    }

    None
}

fn find_sway_focused(val: &serde_json::Value) -> Option<String> {
    if val.get("focused").and_then(|v| v.as_bool()) == Some(true) {
        if let Some(app_id) = val.get("app_id").and_then(|v| v.as_str()) {
            return Some(app_id.to_string());
        }
        if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
            return Some(name.to_string());
        }
    }
    if let Some(nodes) = val.get("nodes").and_then(|v| v.as_array()) {
        for node in nodes {
            if let Some(found) = find_sway_focused(node) {
                return Some(found);
            }
        }
    }
    None
}

/// Path to store battery export json (`~/.cache/icarus-v2mini/battery.json`).
pub fn get_battery_cache_path() -> PathBuf {
    let cache_dir = dirs_fallback_cache();
    let dir = cache_dir.join("icarus-v2mini");
    let _ = fs::create_dir_all(&dir);
    dir.join("battery.json")
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
    let class = if status.is_charging {
        "charging"
    } else if status.level <= 15 {
        "critical"
    } else if status.level <= 30 {
        "warning"
    } else {
        "normal"
    };

    let export = BatteryExport {
        text: format!("{}%", status.level),
        tooltip: format!(
            "ICARUS V2 MINI: {}% ({} mV) [{}]",
            status.level,
            status.voltage_mv,
            if status.is_charging {
                "Charging"
            } else {
                "Battery"
            }
        ),
        percentage: status.level,
        charging: status.is_charging,
        voltage_mv: status.voltage_mv,
        class: class.to_string(),
        active_profile,
    };

    let json = serde_json::to_string_pretty(&export)?;
    fs::write(get_battery_cache_path(), json)?;
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
    fs::write(get_battery_cache_path(), json)?;
    Ok(())
}

/// Evaluates battery telemetry and emits desktop notification when threshold triggers.
pub fn check_and_notify_battery(
    level: u8,
    is_charging: bool,
    last_charging_state: &mut bool,
    last_notified_low: &mut Option<Instant>,
) {
    // Low battery alert with 5-minute cooldown
    if level <= 15 && !is_charging {
        let should_notify = match *last_notified_low {
            None => true,
            Some(t) => t.elapsed() >= Duration::from_secs(300),
        };
        if should_notify {
            send_desktop_notification(
                "⚠️ ICARUS V2 MINI: Baterai Lemah!",
                &format!(
                    "Sisa baterai mouse {}%. Harap sambungkan kabel USB pengisi daya.",
                    level
                ),
                "critical",
            );
            *last_notified_low = Some(Instant::now());
        }
    }

    // Fully charged alert
    if !*last_charging_state && is_charging && level == 100 {
        send_desktop_notification(
            "🔋 ICARUS V2 MINI: Baterai Penuh!",
            "Baterai mouse telah terisi 100%. Kabel dapat dilepas.",
            "normal",
        );
    }

    *last_charging_state = is_charging;
}

/// Evaluates active application against configured rules and switches profile when necessary.
pub fn process_auto_profile(
    dev: &IcarusDevice,
    current_profile: &mut u8,
    last_detected_app: &mut String,
    enable_notify: bool,
) {
    let Ok(cfg) = dev.read_config() else {
        return;
    };

    if !cfg.auto_profile_enabled || cfg.auto_profile_rules.is_empty() {
        return;
    }

    let Some(app) = detect_active_app(&cfg.auto_profile_rules) else {
        return;
    };

    if app == *last_detected_app {
        return;
    }

    *last_detected_app = app.clone();

    let matched = cfg.auto_profile_rules.iter().find(|r| {
        app.contains(&r.app_pattern.to_lowercase())
            && r.profile_index != *current_profile
            && (1..=5).contains(&r.profile_index)
    });

    if let Some(rule) = matched {
        println!(
            "{} Active app '{}' matched rule: Switching to Profile {} ({})",
            "[AUTO-PROFILE]".magenta().bold(),
            app.cyan(),
            rule.profile_index,
            rule.description.yellow()
        );
        let _ = dev.commit_config(rule.profile_index);
        *current_profile = rule.profile_index;

        if enable_notify {
            send_desktop_notification(
                "🎮 Auto-Profile Beralih!",
                &format!(
                    "Aplikasi '{}' terdeteksi. Mouse beralih ke Profile {} ({})",
                    app, rule.profile_index, rule.description
                ),
                "low",
            );
        }
    }
}

/// Run the background battery monitoring and auto-profile daemon.
pub fn run_daemon(config: &DaemonConfig) -> Result<()> {
    println!("{}", "═".repeat(60).cyan());
    println!(
        "{} Starting ICARUS V2 MINI Background Daemon",
        "[DAEMON]".green().bold()
    );
    println!(
        "  Poll Interval:     {}s",
        config.interval_secs.to_string().yellow()
    );
    println!(
        "  Desktop Notify:    {}",
        if config.enable_notify {
            "Enabled".green()
        } else {
            "Disabled".yellow()
        }
    );
    println!(
        "  Auto-Profile:      {}",
        if config.auto_profile {
            "Enabled".green()
        } else {
            "Disabled".yellow()
        }
    );
    println!(
        "  Cache JSON Output: {}",
        get_battery_cache_path().display().to_string().cyan()
    );
    println!("{}", "═".repeat(60).cyan());

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Catch Ctrl+C / SIGINT
    let _ = ctrlc::set_handler(move || {
        println!("\nShutdown signal received. Exiting daemon...");
        r.store(false, Ordering::SeqCst);
    });

    let mut last_notified_low = None::<Instant>;
    let mut last_charging_state = false;
    let mut current_profile = 1u8;
    let mut last_detected_app = String::new();

    while running.load(Ordering::SeqCst) {
        match IcarusDevice::open() {
            Ok(dev) => {
                let online = dev.read_device_online().unwrap_or(true);
                if online {
                    // 1. Export Battery Status & Send Alerts
                    if let Ok(status) = dev.read_battery_status() {
                        let _ = export_battery_status(&status, current_profile);
                        if config.enable_notify {
                            check_and_notify_battery(
                                status.level,
                                status.is_charging,
                                &mut last_charging_state,
                                &mut last_notified_low,
                            );
                        } else {
                            last_charging_state = status.is_charging;
                        }
                    }

                    // 2. Auto-Profile Engine
                    if config.auto_profile {
                        process_auto_profile(
                            &dev,
                            &mut current_profile,
                            &mut last_detected_app,
                            config.enable_notify,
                        );
                    }
                }
            }
            Err(_) => {
                let _ = export_offline_status(current_profile);
            }
        }

        // Sleep for poll interval in small increments to respond quickly to shutdown signal
        for _ in 0..(config.interval_secs.max(1) * 2) {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    println!("{} Daemon stopped gracefully.", "[DAEMON]".green());
    Ok(())
}

/// Generate and install the user systemd service file (`~/.config/systemd/user/icarus-v2mini.service`).
pub fn install_systemd_user_service() -> Result<()> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let service_dir = PathBuf::from(home).join(".config/systemd/user");
    fs::create_dir_all(&service_dir).context("Failed to create systemd user directory")?;

    let service_file = service_dir.join("icarus-v2mini.service");
    let current_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "icarus-v2mini".to_string());

    let unit_content = format!(
        r#"[Unit]
Description=ICARUS V2 MINI Mouse Battery Monitor & Auto-Profile Daemon
Documentation=https://github.com/ayrazarya/icarus-v2mini-rea
After=graphical-session.target

[Service]
Type=simple
ExecStart={} daemon --interval 10
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
"#,
        current_exe
    );

    fs::write(&service_file, unit_content).context("Failed to write systemd service file")?;
    println!(
        "{} Created service unit at: {}",
        "[OK]".green().bold(),
        service_file.display().to_string().cyan()
    );

    // Enable and start via systemctl --user
    println!("Enabling and starting systemd service...");
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let status = Command::new("systemctl")
        .args(["--user", "enable", "--now", "icarus-v2mini.service"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!(
                "{} Service 'icarus-v2mini.service' enabled and active!",
                "[OK]".green().bold()
            );
            println!("Battery status will now automatically update in background on login.");
        }
        _ => {
            println!(
                "{}",
                "Note: Could not run systemctl --user automatically. Run manually:".yellow()
            );
            println!("  systemctl --user daemon-reload");
            println!("  systemctl --user enable --now icarus-v2mini.service");
        }
    }

    Ok(())
}
