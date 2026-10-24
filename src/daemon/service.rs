//! Main telemetry event loop, polling coordinator, and graceful shutdown handler.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use colored::Colorize;

use crate::config::MouseConfig;
use crate::device::IcarusDevice;

use super::config::DaemonConfig;
use super::export::{
    export_battery_status, export_offline_status, get_battery_cache_path, get_default_config_path,
};
use super::notification::check_and_notify_battery;
use super::scanner::{AutoProfileCache, process_auto_profile_cached};

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
    let r = Arc::clone(&running);

    // Catch Ctrl+C / SIGINT
    let _ = ctrlc::set_handler(move || {
        println!("\nShutdown signal received. Exiting daemon...");
        r.store(false, Ordering::SeqCst);
    });

    let mut last_notified_low = None::<Instant>;
    let mut last_charging_state = false;
    let mut current_profile = 1u8;
    // Load last configured active_profile from ~/.config/icarus-v2mini/config.toml
    let default_cfg_path = get_default_config_path();
    if default_cfg_path.exists()
        && let Ok(cfg) = MouseConfig::from_file(&default_cfg_path)
        && (1..=5).contains(&cfg.active_profile)
    {
        current_profile = cfg.active_profile;
    }
    let mut last_detected_app = String::new();
    let mut auto_profile_cache = AutoProfileCache::default();
    let mut current_device: Option<IcarusDevice> = None;

    while running.load(Ordering::SeqCst) {
        if current_device.is_none() {
            current_device = IcarusDevice::open().ok();
        }

        if let Some(ref dev) = current_device {
            match dev.read_device_online() {
                Ok(true) => {
                    // 1. Export Battery Status & Send Alerts
                    match dev.read_battery_status() {
                        Ok(status) => {
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
                        Err(_) => {
                            // Device communication dropped; invalidate handle
                            current_device = None;
                            let _ = export_offline_status(current_profile);
                        }
                    }

                    // 2. Auto-Profile Engine
                    if let Some(ref active_dev) = current_device
                        && config.auto_profile
                    {
                        process_auto_profile_cached(
                            active_dev,
                            &mut auto_profile_cache,
                            &mut current_profile,
                            &mut last_detected_app,
                            config.enable_notify,
                        );
                    }
                }
                Ok(false) => {
                    let _ = export_offline_status(current_profile);
                }
                Err(_) => {
                    // Wireless receiver unplugged or HID transport error; invalidate handle for hot-plug recovery
                    current_device = None;
                    let _ = export_offline_status(current_profile);
                }
            }
        } else {
            let _ = export_offline_status(current_profile);
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
