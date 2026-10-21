//! icarus-v2mini — Native Linux configuration tool for ICARUS V2 MINI gaming mouse
//!
//! Reverse-engineered from the Windows-only ICARUS V2 MINI configuration
//! software. Communicates directly with the mouse over HID to configure
//! DPI, polling rate, LED effects, button mappings, and sensor features.

use icarus_v2mini::{config, device, dpi, gui};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "icarus-v2mini",
    about = "Native Linux configuration tool for ICARUS V2 MINI gaming mouse",
    version,
    long_about = "Configure your ICARUS V2 MINI gaming mouse on Linux.\n\
                  Supports DPI, polling rate, LED effects, LOD, angle snapping, \n\
                  motion sync, debounce, and button remapping.\n\n\
                  Run without arguments to launch the Desktop GUI.\n\
                  Hardware: PAW3395 sensor, NRF52840 MCU."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the Desktop Graphical User Interface (GUI)
    Gui,

    /// Show device information and current settings
    Info,

    /// List connected ICARUS V2 MINI devices
    List,

    /// Get or set the polling rate
    Rate {
        /// Set polling rate (125, 250, 500, 1000)
        #[arg(value_name = "HZ")]
        rate: Option<u16>,
    },

    /// Get or set DPI settings
    Dpi {
        #[command(subcommand)]
        subcmd: Option<DpiCommands>,
    },

    /// Configure LED effects
    Led {
        #[command(subcommand)]
        subcmd: LedCommands,
    },

    /// Get or set lift-off distance
    Lod {
        /// Set LOD: "1mm" or "2mm" (or "low"/"high")
        #[arg(value_name = "DISTANCE")]
        distance: Option<String>,
    },

    /// Toggle angle snapping (line straightening)
    #[command(name = "angle-snap")]
    AngleSnap {
        /// "on" or "off"
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Toggle motion sync
    #[command(name = "motion-sync")]
    MotionSync {
        /// "on" or "off"
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Toggle sensor performance mode (performance vs office)
    Performance {
        /// "on"/"perf" for performance mode, "off"/"office" for power-saving office mode
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Toggle ripple control (high DPI smoothing)
    Ripple {
        /// "on" or "off"
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Set debounce time in milliseconds
    Debounce {
        /// Debounce time in ms (0, 2, 4, 6, 8, 10, 12, 16, 20)
        #[arg(value_name = "MS")]
        ms: Option<u8>,
    },

    /// Read battery level (wireless only)
    Battery,

    /// Apply a saved configuration profile (TOML file)
    Apply {
        /// Path to the TOML configuration file
        #[arg(value_name = "FILE")]
        file: String,
    },

    /// Save current device configuration to a TOML file
    Save {
        /// Path to save the configuration
        #[arg(value_name = "FILE")]
        file: String,
    },

    /// Generate a udev rules file for non-root access
    #[command(name = "udev-rule")]
    UdevRule,
}

#[derive(Subcommand)]
enum DpiCommands {
    /// Show current DPI settings
    Get,

    /// Set DPI for a specific stage (1-indexed)
    Set {
        /// DPI stage number (1–8)
        #[arg(value_name = "STAGE")]
        stage: u8,

        /// DPI value (50–26000)
        #[arg(value_name = "DPI")]
        value: u16,
    },

    /// Set the active DPI stage (1-indexed)
    Active {
        /// Stage number (1–8)
        #[arg(value_name = "STAGE")]
        stage: u8,
    },
}

#[derive(Subcommand)]
enum LedCommands {
    /// Set LED mode (off, solid, breathing)
    Mode {
        #[arg(value_name = "MODE")]
        mode: String,
    },

    /// Set LED brightness (0–10)
    Brightness {
        #[arg(value_name = "LEVEL")]
        level: u8,
    },

    /// Set LED color as hex (e.g., "FF0000" for red)
    Color {
        #[arg(value_name = "HEX")]
        hex: String,
    },

    /// Set breathing speed (1–10)
    Speed {
        #[arg(value_name = "SPEED")]
        speed: u8,
    },

    /// Set a DPI stage indicator color
    #[command(name = "stage-color")]
    StageColor {
        /// DPI stage number (1–8)
        #[arg(value_name = "STAGE")]
        stage: u8,

        /// Hex color (e.g., "00FF00")
        #[arg(value_name = "HEX")]
        hex: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Gui) => {
            if let Err(e) = gui::run_gui() {
                eprintln!("Failed to launch GUI: {}", e);
                std::process::exit(1);
            }
            Ok(())
        }
        Some(Commands::List) => cmd_list(),
        Some(Commands::UdevRule) => cmd_udev_rule(),
        // All other commands need the device connected
        Some(cmd) => {
            let dev = device::IcarusDevice::open()?;
            println!(
                "{} Connected via {}",
                "[OK]".green().bold(),
                dev.connection.cyan()
            );
            match cmd {
                Commands::Info => cmd_info(&dev),
                Commands::Rate { rate } => cmd_rate(&dev, rate),
                Commands::Dpi { subcmd } => cmd_dpi(&dev, subcmd),
                Commands::Led { subcmd } => cmd_led(&dev, subcmd),
                Commands::Lod { distance } => cmd_lod(&dev, distance),
                Commands::AngleSnap { state } => cmd_angle_snap(&dev, state),
                Commands::MotionSync { state } => cmd_motion_sync(&dev, state),
                Commands::Performance { state } => cmd_performance(&dev, state),
                Commands::Ripple { state } => cmd_ripple(&dev, state),
                Commands::Debounce { ms } => cmd_debounce(&dev, ms),
                Commands::Battery => cmd_battery(&dev),
                Commands::Apply { file } => cmd_apply(&dev, &file),
                Commands::Save { file } => cmd_save(&dev, &file),
                Commands::Gui | Commands::List | Commands::UdevRule => unreachable!(),
            }
        }
    }
}

fn cmd_list() -> Result<()> {
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

fn cmd_info(dev: &device::IcarusDevice) -> Result<()> {
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
            "Hint:          Switch mouse to 2.4G or click to wake it up.\n".dimmed()
        );
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

    match dev.read_battery_level() {
        Ok(level) => {
            let bar = battery_bar(level);
            println!("  Battery:       {} {}%", bar, level);
        }
        Err(_) => println!("  Battery:       {} (USB powered)", "N/A".dimmed()),
    }

    println!("{}", "═".repeat(40));
    Ok(())
}

fn battery_bar(level: u8) -> String {
    let filled = (level as usize) / 10;
    let empty = 10 - filled;
    let color = if level > 50 {
        "green"
    } else if level > 20 {
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

fn cmd_rate(dev: &device::IcarusDevice, rate: Option<u16>) -> Result<()> {
    match rate {
        None => {
            let current = dev.read_polling_rate()?;
            println!("Current polling rate: {}", format!("{}", current).yellow());
            println!(
                "\nAvailable rates: {}",
                config::PollingRate::all()
                    .iter()
                    .map(|r| format!("{}", r))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
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
            println!(
                "{} Polling rate set to {}",
                "[OK]".green().bold(),
                format!("{}", rate).yellow()
            );
        }
    }
    Ok(())
}

fn cmd_dpi(dev: &device::IcarusDevice, subcmd: Option<DpiCommands>) -> Result<()> {
    match subcmd {
        None | Some(DpiCommands::Get) => {
            let stage = dev.read_active_dpi_stage()?;
            println!("Active DPI stage: {}", (stage + 1).to_string().yellow());
            println!("DPI range: 50–26,000 (step: 50 below 4000, 200 above)");
        }
        Some(DpiCommands::Set { stage, value }) => {
            if stage == 0 || stage > 8 {
                anyhow::bail!("DPI stage must be 1–8, got {}", stage);
            }
            let snapped = dpi::snap_dpi(value);
            dev.set_dpi_for_stage(stage - 1, snapped)?;
            if snapped != value {
                println!(
                    "{} DPI for stage {} set to {} (snapped from {})",
                    "[OK]".green().bold(),
                    stage.to_string().yellow(),
                    snapped.to_string().cyan(),
                    value
                );
            } else {
                println!(
                    "{} DPI for stage {} set to {}",
                    "[OK]".green().bold(),
                    stage.to_string().yellow(),
                    value.to_string().cyan(),
                );
            }
        }
        Some(DpiCommands::Active { stage }) => {
            if stage == 0 || stage > 8 {
                anyhow::bail!("DPI stage must be 1–8, got {}", stage);
            }
            dev.set_active_dpi_stage(stage - 1)?;
            println!(
                "{} Active DPI stage set to {}",
                "[OK]".green().bold(),
                stage.to_string().yellow(),
            );
        }
    }
    Ok(())
}

fn parse_hex_color(hex: &str) -> Result<config::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        anyhow::bail!("Color must be 6 hex digits (e.g., FF0000), got '{}'", hex);
    }
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    Ok(config::Color::new(r, g, b))
}

fn cmd_led(dev: &device::IcarusDevice, subcmd: LedCommands) -> Result<()> {
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

fn cmd_lod(dev: &device::IcarusDevice, distance: Option<String>) -> Result<()> {
    match distance {
        None => {
            match dev.read_lod() {
                Ok(lod) => println!(
                    "Current Lift-Off Distance (LOD): {}",
                    lod.to_string().yellow()
                ),
                Err(e) => println!("Lift-Off Distance (LOD): error reading ({})", e),
            }
            println!("Options: 1mm (low), 2mm (high)");
        }
        Some(d) => {
            let lod = match d.to_lowercase().as_str() {
                "1mm" | "low" | "1" => config::LiftOffDistance::Low,
                "2mm" | "high" | "2" => config::LiftOffDistance::High,
                _ => anyhow::bail!("Invalid LOD: '{}'. Use: 1mm or 2mm", d),
            };
            dev.set_lod(lod)?;
            println!(
                "{} Lift-off distance set to {}",
                "[OK]".green().bold(),
                lod.to_string().yellow(),
            );
        }
    }
    Ok(())
}

fn cmd_angle_snap(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => match dev.read_angle_snapping() {
            Ok(enabled) => println!(
                "Angle Snapping: {}",
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            ),
            Err(e) => println!("Angle Snapping: error reading ({})", e),
        },
        Some(s) => {
            let enabled = parse_on_off(&s)?;
            dev.set_angle_snapping(enabled)?;
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

fn cmd_motion_sync(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => match dev.read_motion_sync() {
            Ok(enabled) => println!(
                "Motion Sync: {}",
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            ),
            Err(e) => println!("Motion Sync: error reading ({})", e),
        },
        Some(s) => {
            let enabled = parse_on_off(&s)?;
            dev.set_motion_sync(enabled)?;
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

fn cmd_performance(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => match dev.read_performance_mode() {
            Ok(perf) => println!(
                "Sensor Mode: {}",
                if perf {
                    "Performance (Gaming)".green().bold()
                } else {
                    "Office (Power-saving)".yellow()
                }
            ),
            Err(e) => println!("Sensor Mode: error reading ({})", e),
        },
        Some(s) => {
            let perf = parse_on_off(&s)?;
            dev.set_performance_mode(perf)?;
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

fn cmd_ripple(dev: &device::IcarusDevice, state: Option<String>) -> Result<()> {
    match state {
        None => match dev.read_ripple_control() {
            Ok(enabled) => println!(
                "Ripple Control: {}",
                if enabled {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            ),
            Err(e) => println!("Ripple Control: error reading ({})", e),
        },
        Some(s) => {
            let enabled = parse_on_off(&s)?;
            dev.set_ripple_control(enabled)?;
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

fn cmd_debounce(dev: &device::IcarusDevice, ms: Option<u8>) -> Result<()> {
    match ms {
        None => {
            match dev.read_debounce() {
                Ok(ms) => println!("Current debounce time: {} ms", ms.to_string().yellow()),
                Err(e) => println!("Debounce time: error reading ({})", e),
            }
            println!("Options (ms): 0, 2, 4, 6, 8, 10, 12, 16, 20");
        }
        Some(ms) => {
            if ms > 20 {
                anyhow::bail!("Debounce time must be 0–20 ms, got {}", ms);
            }
            dev.set_debounce(ms)?;
            println!(
                "{} Debounce time set to {} ms",
                "[OK]".green().bold(),
                ms.to_string().yellow(),
            );
        }
    }
    Ok(())
}

fn cmd_battery(dev: &device::IcarusDevice) -> Result<()> {
    let level = dev.read_battery_level()?;
    let bar = battery_bar(level);
    println!("Battery: {} {}%", bar, level);
    Ok(())
}

fn cmd_apply(dev: &device::IcarusDevice, file: &str) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read config file: {}", file))?;
    let mut cfg: config::MouseConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", file))?;
    cfg.validate();

    println!("Applying configuration from {}...", file.cyan());
    dev.apply_config(&cfg)?;
    println!(
        "{} Configuration applied successfully!",
        "[OK]".green().bold()
    );
    Ok(())
}

fn cmd_save(dev: &device::IcarusDevice, file: &str) -> Result<()> {
    let cfg = dev.read_config()?;
    let content = toml::to_string_pretty(&cfg).context("Failed to serialize configuration")?;
    std::fs::write(file, &content).with_context(|| format!("Failed to write: {}", file))?;
    println!(
        "{} Configuration saved to {}",
        "[OK]".green().bold(),
        file.cyan()
    );
    Ok(())
}

fn cmd_udev_rule() -> Result<()> {
    let rule = r#"# ICARUS V2 MINI Gaming Mouse — allow non-root HID access
# Place this file at /etc/udev/rules.d/99-icarus-mouse.rules

# USB wired
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f511", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f511", MODE="0666"

# 2.4G Wireless dongle
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f512", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f512", MODE="0666"

# 4G Wireless dongle
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f510", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f510", MODE="0666"
"#;

    println!("{}", "udev rule for ICARUS V2 MINI".bold().cyan());
    println!("{}", "─".repeat(50));
    println!("{}", rule);
    println!("{}", "─".repeat(50));
    println!("\nTo install:");
    println!(
        "  {} | sudo tee /etc/udev/rules.d/99-icarus-mouse.rules",
        "icarus-v2mini udev-rule".bold()
    );
    println!("  sudo udevadm control --reload-rules");
    println!("  sudo udevadm trigger");
    println!("\nOr save to file:");

    let rules_path = "99-icarus-mouse.rules";
    std::fs::write(rules_path, rule)?;
    println!(
        "  {} Written to {}",
        "[OK]".green().bold(),
        rules_path.cyan()
    );
    println!("  sudo cp {} /etc/udev/rules.d/", rules_path);

    Ok(())
}

fn parse_on_off(s: &str) -> Result<bool> {
    match s.to_lowercase().as_str() {
        "on" | "true" | "1" | "enable" | "enabled" | "yes" | "perf" | "performance" => Ok(true),
        "off" | "false" | "0" | "disable" | "disabled" | "no" | "office" | "eco" => Ok(false),
        _ => anyhow::bail!("Invalid value: '{}'. Use 'on' or 'off'", s),
    }
}
