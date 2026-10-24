//! Device information and discovery commands.

use crate::{config, device};
use anyhow::Result;
use colored::Colorize;

pub fn cmd_list() -> Result<()> {
    let devices = device::list_devices()?;
    if devices.is_empty() {
        println!("{}", "No ICARUS V2 MINI devices found.".yellow());
        println!("Make sure the mouse is connected.");
        println!(
            "If connected but not visible, run: {} {}",
            "icarus-v2mini".bold(),
            "udev-rule".bold()
        );
    } else {
        println!("{}", "Found devices:".green().bold());
        for (i, dev) in devices.iter().enumerate() {
            println!("  {}. {}", i + 1, dev);
            println!("     Path: {}", dev.path.dimmed());
        }
    }
    Ok(())
}

pub fn cmd_info(dev: &device::IcarusDevice) -> Result<()> {
    println!("\n{}", "ICARUS V2 MINI".bold().cyan());
    println!("{}", "═".repeat(40));

    if let Ok(false) = dev.read_device_online() {
        println!(
            "  {}",
            "Status:        Dongle Ready (Mouse Offline / Asleep)"
                .yellow()
                .bold()
        );
        println!(
            "  {}",
            "Hint:          Switch mouse to 2.4G or click to wake it up.".dimmed()
        );
        println!("{}", "═".repeat(40));
        return Ok(());
    }

    match dev.read_polling_rate() {
        Ok(rate) => println!("  Polling Rate:  {}", format!("{}", rate).yellow()),
        Err(e) => println!("  Polling Rate:  {} ({})", "?".red(), e),
    }

    match dev.read_active_dpi_stage() {
        Ok(stage) => println!(
            "  Active DPI:    Stage {}",
            (stage + 1).to_string().yellow()
        ),
        Err(e) => println!("  Active DPI:    {} ({})", "?".red(), e),
    }

    if let Ok(lod) = dev.read_lod() {
        println!("  Lift-off Dist: {}", lod.to_string().yellow());
    }

    if let Ok(perf) = dev.read_performance_mode() {
        println!(
            "  Sensor Mode:   {}",
            if perf {
                "Performance".green().bold()
            } else {
                "Office".yellow()
            }
        );
    }

    if let Ok(ms) = dev.read_debounce() {
        println!("  Debounce Time: {} ms", ms.to_string().yellow());
    }

    if let Ok(snap) = dev.read_angle_snapping() {
        println!(
            "  Angle Snapping:{}",
            if snap {
                " Enabled".green()
            } else {
                " Disabled".dimmed()
            }
        );
    }

    if let Ok(sync) = dev.read_motion_sync() {
        println!(
            "  Motion Sync:   {}",
            if sync {
                " Enabled".green()
            } else {
                " Disabled".dimmed()
            }
        );
    }

    if let Ok(ripple) = dev.read_ripple_control() {
        println!(
            "  Ripple Control:{}",
            if ripple {
                " Enabled".green()
            } else {
                " Disabled".dimmed()
            }
        );
    }

    if let Ok(sec) = dev.read_sensor_sleep() {
        if sec > 0 {
            println!("  Sleep Timer:   {}s", sec.to_string().yellow());
        } else {
            println!("  Sleep Timer:   Never");
        }
    }

    match dev.read_dpi_led() {
        Ok(led) => {
            println!("  LED Mode:      {}", format!("{}", led.mode).yellow());
            println!("  LED Bright:    {}/10", led.brightness);
            println!("  LED Color:     {}", led.color);
            if led.mode == config::LedMode::Breathing {
                println!("  Breath Speed:  {}/10", led.breath_speed);
            }
        }
        Err(e) => println!("  LED:           {} ({})", "?".red(), e),
    }

    match dev.read_firmware_version() {
        Ok(ver) => println!("  Firmware:      {}", ver.green()),
        Err(e) => println!("  Firmware:      {} ({})", "?".red(), e),
    }

    match dev.read_battery_status() {
        Ok(status) => {
            let bar = battery_bar(status.level);
            let state_str = if status.is_charging {
                " (⚡ Charging / USB)".cyan()
            } else if status.level <= 10 {
                " (🪫 Sekarat / Kritis!)".red().bold()
            } else if status.level <= 20 {
                " (⚠️ Lemah)".yellow()
            } else {
                "".normal()
            };
            println!("  Battery:       {} {}%{}", bar, status.level, state_str);
        }
        Err(_) => println!("  Battery:       {} (USB powered)", "N/A".dimmed()),
    }

    println!("{}", "═".repeat(40));
    Ok(())
}

pub fn battery_bar(level: u8) -> String {
    let clamped = level.min(100);
    let filled = (clamped as usize) / 10;
    let empty = 10usize.saturating_sub(filled);
    let color = if clamped > 50 {
        "green"
    } else if clamped > 20 {
        "yellow"
    } else {
        "red"
    };
    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));
    match color {
        "green" => bar.green().to_string(),
        "yellow" => bar.yellow().to_string(),
        "red" => bar.red().to_string(),
        _ => bar,
    }
}
