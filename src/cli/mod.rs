//! Command-Line Interface (CLI) runner and dispatch for ICARUS V2 MINI.

pub mod args;
pub mod commands;
pub mod helpers;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::{daemon, device, gui};
use args::{Cli, Commands};
use commands::{
    config::{cmd_apply, cmd_save},
    dpi::cmd_dpi,
    info::{cmd_info, cmd_list},
    lighting::{cmd_dongle_rgb, cmd_led},
    macros::cmd_macro,
    sensor::{
        cmd_angle_snap, cmd_battery, cmd_debounce, cmd_lod, cmd_long_range, cmd_motion_sync,
        cmd_performance, cmd_rate, cmd_ripple,
    },
    system::{cmd_dfu, cmd_flash, cmd_pair, cmd_udev_rule},
};

/// Parse command-line arguments and dispatch the requested command.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Gui) => {
            if let Err(e) = gui::run_gui() {
                anyhow::bail!("Failed to launch GUI: {}", e);
            }
            Ok(())
        }
        Some(Commands::List) => cmd_list(),
        Some(Commands::UdevRule { install }) => cmd_udev_rule(install),
        Some(Commands::Daemon {
            interval,
            no_notify,
            no_auto_profile,
            install_service,
        }) => {
            if install_service {
                daemon::install_systemd_user_service()
            } else {
                let config = daemon::DaemonConfig::new()
                    .with_interval(interval)
                    .with_notifications(!no_notify)
                    .with_auto_profile(!no_auto_profile);
                daemon::run_daemon(&config)
            }
        }
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
                Commands::Pair { cid, mid } => cmd_pair(&dev, cid, mid),
                Commands::LongRange { state } => cmd_long_range(&dev, state),
                Commands::Dfu => cmd_dfu(&dev),
                Commands::DongleRgb {
                    mode,
                    color,
                    brightness,
                    speed,
                    bar_mode,
                } => cmd_dongle_rgb(&dev, mode, color, brightness, speed, bar_mode),
                Commands::Flash { file, dongle } => cmd_flash(&dev, &file, dongle),
                Commands::Macro { subcmd } => cmd_macro(&dev, subcmd),
                Commands::Gui
                | Commands::List
                | Commands::UdevRule { .. }
                | Commands::Daemon { .. } => unreachable!(),
            }
        }
    }
}
