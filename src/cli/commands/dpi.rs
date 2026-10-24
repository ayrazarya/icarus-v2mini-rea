//! DPI sensitivity and stage management commands.

use crate::cli::args::DpiCommands;
use crate::{device, dpi};
use anyhow::Result;
use colored::Colorize;

pub fn cmd_dpi(dev: &device::IcarusDevice, subcmd: Option<DpiCommands>) -> Result<()> {
    match subcmd {
        None | Some(DpiCommands::Get) => {
            let stage = dev.read_active_dpi_stage()?;
            println!("Active DPI stage: {}", (stage + 1).to_string().yellow());
            println!("DPI range: 50–26,000 (step: 50 below 4000, 200 above)");
        }
        Some(DpiCommands::Set {
            stage,
            value,
            dpi_y,
        }) => {
            if stage == 0 || stage > 8 {
                anyhow::bail!("DPI stage must be 1–8, got {}", stage);
            }
            let snapped_x = dpi::snap_dpi(value);
            if let Some(y) = dpi_y {
                let snapped_y = dpi::snap_dpi(y);
                dev.set_dpi_for_stage_xy(stage - 1, snapped_x, snapped_y)?;
                dev.commit_config(1)?;
                println!(
                    "{} DPI for stage {} set to X: {}, Y: {}",
                    "[OK]".green().bold(),
                    stage.to_string().yellow(),
                    snapped_x.to_string().cyan(),
                    snapped_y.to_string().cyan()
                );
            } else {
                dev.set_dpi_for_stage(stage - 1, snapped_x)?;
                dev.commit_config(1)?;
                if snapped_x != value {
                    println!(
                        "{} DPI for stage {} set to {} (snapped from {})",
                        "[OK]".green().bold(),
                        stage.to_string().yellow(),
                        snapped_x.to_string().cyan(),
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
