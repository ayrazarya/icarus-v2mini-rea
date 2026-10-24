//! Active window and process inspection engine for Wayland, X11, and Linux /proc.

use std::fs;
use std::io::Read;
use std::process::Command;
use std::time::SystemTime;

use colored::Colorize;

use crate::config::{AutoProfileRule, MouseConfig};
use crate::device::IcarusDevice;

use super::export::get_default_config_path;
use super::notification::send_desktop_notification;

/// Caches parsed auto-profile rules by file modification timestamp (`mtime`)
/// so the daemon avoids re-reading and parsing `config.toml` on every poll tick.
#[derive(Debug, Clone, Default)]
pub struct AutoProfileCache {
    last_mtime: Option<SystemTime>,
    enabled: bool,
    rules: Vec<AutoProfileRule>,
    reloaded: bool,
}

impl AutoProfileCache {
    /// Reloads `config.toml` only if its modification timestamp has changed.
    pub fn refresh(&mut self) -> (bool, &[AutoProfileRule]) {
        self.reloaded = false;
        let config_path = get_default_config_path();
        let current_mtime = fs::metadata(&config_path).and_then(|m| m.modified()).ok();

        if current_mtime.is_some() && current_mtime == self.last_mtime {
            return (self.enabled, &self.rules);
        }

        if let Some(mtime) = current_mtime
            && let Ok(cfg) = MouseConfig::from_file(&config_path)
        {
            self.last_mtime = Some(mtime);
            self.enabled = cfg.auto_profile_enabled;
            self.rules = cfg.auto_profile_rules;
            self.reloaded = true;
        } else if current_mtime.is_none() {
            if self.last_mtime.is_some() {
                self.reloaded = true;
            }
            self.last_mtime = None;
            self.enabled = false;
            self.rules.clear();
        }

        (self.enabled, &self.rules)
    }

    /// Returns whether the most recent [`Self::refresh`] call reloaded `config.toml`.
    #[must_use]
    pub const fn was_reloaded(&self) -> bool {
        self.reloaded
    }
}

/// Zero-allocation ASCII case-insensitive substring check.
#[inline]
fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

/// Inspects active window or running processes across Hyprland, Sway, GNOME/KDE (xdotool),
/// and Linux `/proc` scanner to identify the currently focused application.
pub fn detect_active_app(rules: &[AutoProfileRule]) -> Option<String> {
    // 1. Try Hyprland (only spawn if Hyprland compositor is active in environment)
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
        && let Ok(output) = Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
        && output.status.success()
    {
        let s = String::from_utf8_lossy(&output.stdout);
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
            if let Some(class) = val.get("class").and_then(|v| v.as_str())
                && !class.is_empty()
            {
                return Some(class.to_lowercase());
            }
            if let Some(title) = val.get("title").and_then(|v| v.as_str())
                && !title.is_empty()
            {
                return Some(title.to_lowercase());
            }
        }
    }

    // 2. Try Sway (only spawn if Sway socket is active in environment)
    if std::env::var_os("SWAYSOCK").is_some()
        && let Ok(output) = Command::new("swaymsg").args(["-t", "get_tree"]).output()
        && output.status.success()
    {
        let s = String::from_utf8_lossy(&output.stdout);
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s)
            && let Some(focused) = find_sway_focused(&val)
        {
            return Some(focused.to_lowercase());
        }
    }

    // 3. Try X11 / XWayland via xprop (only spawn if DISPLAY is set)
    if std::env::var_os("DISPLAY").is_some()
        && let Ok(output) = Command::new("xprop")
            .args(["-root", "_NET_ACTIVE_WINDOW"])
            .output()
        && output.status.success()
    {
        let s = String::from_utf8_lossy(&output.stdout);
        if let Some(id_part) = s.split('#').nth(1) {
            let win_id = id_part.split_whitespace().next().unwrap_or("");
            if !win_id.is_empty()
                && win_id != "0x0"
                && let Ok(class_out) = Command::new("xprop")
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

    // 4. Linux /proc scanner fallback: Check if target applications/games are currently running
    let valid_patterns: Vec<&[u8]> = rules
        .iter()
        .map(|r| r.app_pattern.trim().as_bytes())
        .filter(|pat| !pat.is_empty())
        .collect();

    if !valid_patterns.is_empty()
        && let Ok(entries) = fs::read_dir("/proc")
    {
        let mut comm_buf = [0u8; 64];
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(name_str) = file_name.to_str() else {
                continue;
            };
            // Only inspect numeric process IDs, skipping /proc/sys, /proc/net, /proc/acpi, etc.
            if !name_str.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let comm_file = entry.path().join("comm");
            if let Ok(mut file) = fs::File::open(comm_file)
                && let Ok(n) = file.read(&mut comm_buf)
                && n > 0
            {
                let comm_trimmed = comm_buf[..n].trim_ascii();
                if comm_trimmed.is_empty() {
                    continue;
                }
                for &pat in &valid_patterns {
                    if contains_ignore_ascii_case(comm_trimmed, pat) {
                        return Some(String::from_utf8_lossy(comm_trimmed).to_lowercase());
                    }
                }
            }
        }
    }

    None
}

fn find_sway_focused(val: &serde_json::Value) -> Option<String> {
    if val.get("focused").and_then(|v| v.as_bool()) == Some(true) {
        if let Some(app_id) = val.get("app_id").and_then(|v| v.as_str())
            && !app_id.is_empty()
        {
            return Some(app_id.to_string());
        }
        if let Some(name) = val.get("name").and_then(|v| v.as_str())
            && !name.is_empty()
        {
            return Some(name.to_string());
        }
    }
    for key in ["nodes", "floating_nodes"] {
        if let Some(nodes) = val.get(key).and_then(|v| v.as_array()) {
            for node in nodes {
                if let Some(found) = find_sway_focused(node) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// Evaluates active application against configured rules and switches profile when necessary.
pub fn process_auto_profile(
    dev: &IcarusDevice,
    current_profile: &mut u8,
    last_detected_app: &mut String,
    enable_notify: bool,
) {
    let mut cache = AutoProfileCache::default();
    process_auto_profile_cached(
        dev,
        &mut cache,
        current_profile,
        last_detected_app,
        enable_notify,
    );
}

/// Evaluates active application using an `mtime`-backed configuration cache.
pub fn process_auto_profile_cached(
    dev: &IcarusDevice,
    cache: &mut AutoProfileCache,
    current_profile: &mut u8,
    last_detected_app: &mut String,
    enable_notify: bool,
) {
    let _ = cache.refresh();
    let config_reloaded = cache.was_reloaded();
    let auto_profile_enabled = cache.enabled;
    let rules = &cache.rules;

    if !auto_profile_enabled || rules.is_empty() {
        return;
    }

    let Some(app) = detect_active_app(rules) else {
        return;
    };

    if !config_reloaded && app == *last_detected_app {
        return;
    }

    // Match the first applicable rule by pattern priority first, then check if profile differs
    let matched = rules.iter().find(|r| {
        let pat = r.app_pattern.trim();
        !pat.is_empty()
            && (1..=5).contains(&r.profile_index)
            && contains_ignore_ascii_case(app.as_bytes(), pat.as_bytes())
    });

    if let Some(rule) = matched
        && rule.profile_index != *current_profile
    {
        println!(
            "{} Active app '{}' matched rule: Switching to Profile {} ({})",
            "[AUTO-PROFILE]".magenta().bold(),
            app.cyan(),
            rule.profile_index,
            rule.description.yellow()
        );
        match dev.commit_config(rule.profile_index) {
            Ok(()) => {
                *current_profile = rule.profile_index;
                *last_detected_app = app.clone();

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
            Err(e) => {
                eprintln!(
                    "{} Failed to switch to Profile {}: {}",
                    "[ERROR]".red().bold(),
                    rule.profile_index,
                    e
                );
            }
        }
    } else {
        *last_detected_app = app;
    }
}
