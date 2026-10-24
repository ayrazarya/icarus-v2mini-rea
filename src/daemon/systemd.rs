//! Systemd user service installation and autostart enablement.

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
ExecStart="{}" daemon --interval 10
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
    let reload_ok = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()
        .is_ok_and(|s| s.success());
    let enable_ok = reload_ok
        && Command::new("systemctl")
            .args(["--user", "enable", "--now", "icarus-v2mini.service"])
            .status()
            .is_ok_and(|s| s.success());

    if enable_ok {
        println!(
            "{} Service 'icarus-v2mini.service' enabled and active!",
            "[OK]".green().bold()
        );
        println!("Battery status will now automatically update in background on login.");
    } else {
        println!(
            "{}",
            "Note: Could not run systemctl --user automatically. Run manually:".yellow()
        );
        println!("  systemctl --user daemon-reload");
        println!("  systemctl --user enable --now icarus-v2mini.service");
    }

    Ok(())
}
