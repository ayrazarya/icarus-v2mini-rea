//! Hardware macro flashing commands.

use crate::cli::args::MacroCommands;
use crate::{config, device};
use anyhow::{Context, Result};
use colored::Colorize;

pub fn cmd_macro(dev: &device::IcarusDevice, subcmd: MacroCommands) -> Result<()> {
    match subcmd {
        MacroCommands::Flash {
            slot,
            name,
            sequence,
        } => {
            if slot == 0 || slot > 16 {
                anyhow::bail!("Macro slot must be 1–16, got {}", slot);
            }
            let hardware_slot = slot - 1;

            let mut events = Vec::new();
            for token in sequence.split(',') {
                let token = token.trim();
                if token.is_empty() {
                    continue;
                }
                if let Some(delay_str) = token.strip_suffix("ms")
                    && let Ok(ms) = delay_str.parse::<u32>()
                {
                    events.push(config::MacroEvent {
                        event_type: config::MacroEventType::Delay,
                        key: String::new(),
                        delay_ms: ms,
                    });
                    continue;
                }
                if let Ok(ms) = token.parse::<u32>() {
                    events.push(config::MacroEvent {
                        event_type: config::MacroEventType::Delay,
                        key: String::new(),
                        delay_ms: ms,
                    });
                    continue;
                }
                if let Some(key) = token.strip_prefix('+') {
                    let key_trimmed = key.trim();
                    if key_trimmed.is_empty() {
                        anyhow::bail!("Macro key down token '+' is missing a key name");
                    }
                    if crate::protocol::key_to_hid_usage(key_trimmed) == 0 {
                        anyhow::bail!("Unrecognized macro key name: '{}'", key_trimmed);
                    }
                    events.push(config::MacroEvent {
                        event_type: config::MacroEventType::KeyDown,
                        key: key_trimmed.to_string(),
                        delay_ms: 0,
                    });
                    continue;
                }
                if let Some(key) = token.strip_prefix('-') {
                    let key_trimmed = key.trim();
                    if key_trimmed.is_empty() {
                        anyhow::bail!("Macro key up token '-' is missing a key name");
                    }
                    if crate::protocol::key_to_hid_usage(key_trimmed) == 0 {
                        anyhow::bail!("Unrecognized macro key name: '{}'", key_trimmed);
                    }
                    events.push(config::MacroEvent {
                        event_type: config::MacroEventType::KeyUp,
                        key: key_trimmed.to_string(),
                        delay_ms: 0,
                    });
                    continue;
                }
                if crate::protocol::key_to_hid_usage(token) == 0 {
                    anyhow::bail!("Unrecognized macro key name: '{}'", token);
                }
                // Default: press and release with 12ms delay
                events.push(config::MacroEvent {
                    event_type: config::MacroEventType::KeyDown,
                    key: token.to_string(),
                    delay_ms: 0,
                });
                events.push(config::MacroEvent {
                    event_type: config::MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 12,
                });
                events.push(config::MacroEvent {
                    event_type: config::MacroEventType::KeyUp,
                    key: token.to_string(),
                    delay_ms: 0,
                });
            }

            if events.is_empty() {
                anyhow::bail!("Macro sequence cannot be empty");
            }
            if events.len() > 64 {
                anyhow::bail!(
                    "Macro sequence expands to {} events, exceeding hardware limit of 64 events",
                    events.len()
                );
            }

            println!(
                "Flashing macro '{}' ({} events) to hardware slot {}...",
                name.yellow(),
                events.len(),
                slot
            );
            dev.flash_macro_slot(hardware_slot, &name, &events)?;
            println!(
                "{} Macro '{}' flashed to slot {} successfully!",
                "[OK]".green().bold(),
                name.yellow(),
                slot
            );
        }
        MacroCommands::FlashAll { file } => {
            let cfg = config::MouseConfig::from_file(&file)
                .with_context(|| format!("Failed to load config file: {}", file))?;

            println!(
                "Flashing all assigned macros from {} to hardware memory...",
                file.cyan()
            );
            dev.flash_all_assigned_macros(&cfg)?;
            println!(
                "{} All assigned macros flashed to hardware memory successfully!",
                "[OK]".green().bold()
            );
        }
    }
    Ok(())
}
