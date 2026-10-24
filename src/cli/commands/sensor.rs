//! Sensor and performance configuration commands.

use crate::cli::commands::info::battery_bar;
use crate::cli::helpers::parse_on_off;
use crate::{config, device};
use anyhow::Result;
use colored::Colorize;

pub fn cmd_rate(dev: &device::IcarusDevice, rate: Option<u16>) -> Result<()> {
    match rate {
        None => {
            let current = dev.read_polling_rate()?;
            println!("Current polling rate: {}", format!("{}", current).yellow());
            println!("\nAvailable rates: 125 Hz, 250 Hz, 500 Hz, 1000 Hz, 2000 Hz, 4000 Hz");
        }
        Some(hz) => {
            let rate = match hz {
                125 => config::PollingRate::Hz125,
                250 => config::PollingRate::Hz250,
                500 => config::PollingRate::Hz500,
                1000 => config::PollingRate::Hz1000,
                2000 => config::PollingRate::Hz2000,
                4000 => config::PollingRate::Hz4000,
                _ => anyhow::bail!(
                    "Invalid polling rate: {}. Valid values: 125, 250, 500, 1000, 2000, 4000",
                    hz
                ),
            };
            dev.set_polling_rate(rate)?;
            dev.commit_config(1)?;
            println!(
                "{} Polling rate set to {}",
                "[OK]".green().bold(),
                format!("{}", rate).yellow()
            );
        }
    }
    Ok(())
}

pub fn cmd_lod(dev: &device::IcarusDevice, distance: Option<String>) -> Result<()> {
    match distance {
        None => {
            let lod = dev.read_lod()?;
            println!(
                "Current Lift-Off Distance (LOD): {}",
                lod.to_string().yellow()
            );
            println!("Options: 1mm (low), 2mm (high)");
        }
        Some(d) => {
            let lod = match d.to_lowercase().as_str() {
                "1mm" | "low" | "1" => config::LiftOffDistance::Low,
                "2mm" | "high" | "2" => config::LiftOffDistance::High,
                _ => anyhow::bail!("Invalid LOD: '{}'. Use: 1mm or 2mm", d),
            };
            dev.set_lod(lod)?;
            dev.commit_config(1)?;
            println!(
                "{} Lift-off distance set to {}",
                "[OK]".green().bold(),
                lod.to_string().yellow(),
            );
        }
    }
    Ok(())
}

pub fn cmd_angle_snap(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => {
            let enabled = dev.read_angle_snapping()?;
            println!(
                "Angle Snapping: {}",
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            );
        }
        Some(s) => {
            let enabled = parse_on_off(&s)?;
            dev.set_angle_snapping(enabled)?;
            dev.commit_config(1)?;
            println!(
                "{} Angle snapping {}",
                "[OK]".green().bold(),
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            );
        }
    }
    Ok(())
}

pub fn cmd_motion_sync(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => {
            let enabled = dev.read_motion_sync()?;
            println!(
                "Motion Sync: {}",
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            );
        }
        Some(s) => {
            let enabled = parse_on_off(&s)?;
            dev.set_motion_sync(enabled)?;
            dev.commit_config(1)?;
            println!(
                "{} Motion sync {}",
                "[OK]".green().bold(),
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            );
        }
    }
    Ok(())
}

pub fn cmd_performance(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => {
            let perf = dev.read_performance_mode()?;
            println!(
                "Sensor Mode: {}",
                if perf {
                    "Performance (Gaming)".green().bold()
                } else {
                    "Office (Power-saving)".yellow()
                }
            );
        }
        Some(s) => {
            let perf = parse_on_off(&s)?;
            dev.set_performance_mode(perf)?;
            dev.commit_config(1)?;
            println!(
                "{} Sensor mode set to {}",
                "[OK]".green().bold(),
                if perf {
                    "Performance".green().bold()
                } else {
                    "Office".yellow()
                }
            );
        }
    }
    Ok(())
}

pub fn cmd_ripple(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => {
            let enabled = dev.read_ripple_control()?;
            println!(
                "Ripple Control: {}",
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            );
        }
        Some(s) => {
            let enabled = parse_on_off(&s)?;
            dev.set_ripple_control(enabled)?;
            dev.commit_config(1)?;
            println!(
                "{} Ripple control {}",
                "[OK]".green().bold(),
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            );
        }
    }
    Ok(())
}

pub fn cmd_debounce(dev: &device::IcarusDevice, ms: Option<u8>) -> Result<()> {
    match ms {
        None => {
            let ms = dev.read_debounce()?;
            println!("Current debounce time: {} ms", ms.to_string().yellow());
            println!("Options (ms): 0, 2, 4, 6, 8, 10, 12, 16, 20");
        }
        Some(ms) => {
            if ms > 20 {
                anyhow::bail!("Debounce time must be 0–20 ms, got {}", ms);
            }
            let snapped_ms = (ms / 2) * 2;
            dev.set_debounce(snapped_ms)?;
            dev.commit_config(1)?;
            println!(
                "{} Debounce time set to {} ms",
                "[OK]".green().bold(),
                snapped_ms.to_string().yellow(),
            );
        }
    }
    Ok(())
}

pub fn cmd_battery(dev: &device::IcarusDevice) -> Result<()> {
    let status = dev.read_battery_status()?;
    let bar = battery_bar(status.level);
    let state_str = if status.is_charging {
        " [⚡ Charging / USB Mode]".cyan().bold()
    } else if status.level <= 10 {
        " [🪫 Sekarat / Kritis!]".red().bold()
    } else if status.level <= 20 {
        " [⚠️ Baterai Lemah]".yellow().bold()
    } else {
        "".normal()
    };
    println!("Battery: {} {}%{}", bar, status.level, state_str);
    if status.voltage_mv > 0 {
        println!(
            "Voltage: {:.2} V ({} mV)",
            status.voltage_mv as f32 / 1000.0,
            status.voltage_mv
        );
    }
    Ok(())
}

pub fn cmd_long_range(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => {
            let current = dev.read_long_range_mode()?;
            println!(
                "Long Range Mode (High RF Power): {}",
                if current {
                    "on".green().bold()
                } else {
                    "off".yellow()
                }
            );
        }
        Some(s) => {
            let enable = parse_on_off(&s)?;
            dev.set_long_range_mode(enable)?;
            println!(
                "{} Long Range Mode set to {}",
                "[OK]".green().bold(),
                if enable { "on".green() } else { "off".yellow() }
            );
        }
    }
    Ok(())
}
