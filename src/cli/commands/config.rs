//! Configuration file profile apply and save commands.

use crate::{config, device};
use anyhow::{Context, Result};
use colored::Colorize;

pub fn cmd_apply(dev: &device::IcarusDevice, file: &str) -> Result<()> {
    let cfg = config::MouseConfig::from_file(file)
        .with_context(|| format!("Failed to load config file: {}", file))?;

    println!("Applying configuration from {}...", file.cyan());
    dev.apply_config(&cfg)?;
    println!(
        "{} Configuration applied successfully!",
        "[OK]".green().bold()
    );
    Ok(())
}

pub fn cmd_save(dev: &device::IcarusDevice, file: &str) -> Result<()> {
    let cfg = dev.read_config()?;
    cfg.to_file(file)
        .with_context(|| format!("Failed to save configuration to: {}", file))?;
    println!(
        "{} Configuration saved to {}",
        "[OK]".green().bold(),
        file.cyan()
    );
    Ok(())
}
