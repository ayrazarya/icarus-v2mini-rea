//! RGB lighting and dongle illumination commands.

use crate::cli::args::LedCommands;
use crate::cli::helpers::parse_hex_color;
use crate::{config, device};
use anyhow::Result;
use colored::Colorize;

pub fn cmd_led(dev: &device::IcarusDevice, subcmd: LedCommands) -> Result<()> {
    match subcmd {
        LedCommands::Mode { mode } => {
            let led_mode = match mode.to_lowercase().as_str() {
                "off" | "0" => config::LedMode::Off,
                "solid" | "on" | "1" => config::LedMode::Solid,
                "breathing" | "breath" | "2" => config::LedMode::Breathing,
                "neon" | "cycle" | "rainbow" | "3" => config::LedMode::Neon,
                _ => anyhow::bail!(
                    "Invalid LED mode: '{}'. Use: off, solid, breathing, neon",
                    mode
                ),
            };
            // Read current, modify mode, write back
            let mut led = dev.read_dpi_led()?;
            led.mode = led_mode;
            dev.set_dpi_led(&led)?;
            println!(
                "{} LED mode set to {}",
                "[OK]".green().bold(),
                format!("{}", led_mode).yellow(),
            );
        }
        LedCommands::Brightness { level } => {
            if level > 10 {
                anyhow::bail!("Brightness must be 0–10, got {}", level);
            }
            let mut led = dev.read_dpi_led()?;
            led.brightness = level;
            dev.set_dpi_led(&led)?;
            println!(
                "{} LED brightness set to {}/10",
                "[OK]".green().bold(),
                level.to_string().yellow(),
            );
        }
        LedCommands::Color { hex } => {
            let color = parse_hex_color(&hex)?;
            let mut led = dev.read_dpi_led()?;
            led.color = color;
            dev.set_dpi_led(&led)?;
            println!(
                "{} LED color set to {}",
                "[OK]".green().bold(),
                format!("{}", color).yellow(),
            );
        }
        LedCommands::Speed { speed } => {
            if speed == 0 || speed > 10 {
                anyhow::bail!("Breath speed must be 1–10, got {}", speed);
            }
            let mut led = dev.read_dpi_led()?;
            led.breath_speed = speed;
            dev.set_dpi_led(&led)?;
            println!(
                "{} Breathing speed set to {}/10",
                "[OK]".green().bold(),
                speed.to_string().yellow(),
            );
        }
        LedCommands::StageColor { stage, hex } => {
            if stage == 0 || stage > 8 {
                anyhow::bail!("Stage must be 1–8, got {}", stage);
            }
            let color = parse_hex_color(&hex)?;
            dev.set_dpi_stage_color(stage - 1, color)?;
            dev.commit_config(1)?;
            println!(
                "{} Stage {} LED color set to {}",
                "[OK]".green().bold(),
                stage.to_string().yellow(),
                format!("{}", color).cyan(),
            );
        }
    }
    Ok(())
}

pub fn cmd_dongle_rgb(
    dev: &device::IcarusDevice,
    mode: Option<String>,
    color: Option<String>,
    brightness: Option<u8>,
    speed: Option<u8>,
    bar_mode: Option<u8>,
) -> Result<()> {
    let mut cfg = config::DongleRgbConfig::default();

    let mut changed = false;
    if let Some(m) = mode {
        cfg.mode = match m.to_lowercase().as_str() {
            "off" | "0" => {
                cfg.enabled = false;
                config::LedMode::Off
            }
            "solid" | "on" | "1" => {
                cfg.enabled = true;
                config::LedMode::Solid
            }
            "breathing" | "breath" | "2" => {
                cfg.enabled = true;
                config::LedMode::Breathing
            }
            "neon" | "cycle" | "rainbow" | "3" => {
                cfg.enabled = true;
                config::LedMode::Neon
            }
            _ => anyhow::bail!("Invalid mode: '{}'. Use: off, solid, breathing, neon", m),
        };
        changed = true;
    }

    if let Some(c) = color {
        cfg.color = parse_hex_color(&c)?;
        changed = true;
    }

    if let Some(b) = brightness {
        if b > 10 {
            anyhow::bail!("Brightness must be 0–10, got {}", b);
        }
        cfg.brightness = b;
        changed = true;
    }

    if let Some(s) = speed {
        if s == 0 || s > 10 {
            anyhow::bail!("Speed must be 1–10, got {}", s);
        }
        cfg.speed = s;
        changed = true;
    }

    if let Some(bm) = bar_mode {
        if bm > 2 {
            anyhow::bail!(
                "Bar mode must be 0 (sync), 1 (battery), or 2 (fixed), got {}",
                bm
            );
        }
        cfg.bar_mode = bm;
        changed = true;
    }

    if !changed {
        println!("{}", "4K Wireless Dongle RGB Settings:".bold().cyan());
        println!(
            "  Enabled:    {}",
            if cfg.enabled {
                "Yes".green()
            } else {
                "No".red()
            }
        );
        println!("  Mode:       {}", format!("{}", cfg.mode).yellow());
        println!("  Brightness: {}/10", cfg.brightness.to_string().yellow());
        println!("  Speed:      {}/10", cfg.speed.to_string().yellow());
        println!("  Color:      {}", format!("{}", cfg.color).yellow());
        println!(
            "  Bar Mode:   {}",
            match cfg.bar_mode {
                0 => "Sync with RGB",
                1 => "Battery Status Indicator",
                2 => "Fixed Color",
                _ => "Custom",
            }
            .yellow()
        );
        println!(
            "\n{}",
            "To change settings: icarus-v2mini dongle-rgb --mode breathing --color 00F0FF --brightness 10 --speed 5"
                .dimmed()
        );
        return Ok(());
    }

    dev.apply_dongle_rgb(&cfg)?;
    println!(
        "{} 4K Dongle RGB updated (Mode: {}, Brightness: {}/10, Speed: {}, Color: {}, Bar: {})",
        "[OK]".green().bold(),
        format!("{}", cfg.mode).yellow(),
        cfg.brightness,
        cfg.speed,
        format!("{}", cfg.color).yellow(),
        cfg.bar_mode
    );
    Ok(())
}
