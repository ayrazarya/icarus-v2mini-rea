//! System integration, pairing, and firmware update commands.

use crate::device;
use anyhow::Result;
use colored::Colorize;

pub fn cmd_pair(dev: &device::IcarusDevice, cid: Option<u8>, mid: Option<u8>) -> Result<()> {
    println!(
        "{}",
        "Initiating 2.4G wireless dongle pairing (Match)..."
            .cyan()
            .bold()
    );
    dev.pair_dongle(cid, mid)?;
    println!(
        "{} Pairing command successfully transmitted to dongle!",
        "[OK]".green().bold()
    );
    println!("Please place your mouse close to the dongle (< 10cm) and turn it on.");
    if let Ok(status) = dev.read_dongle_pair_status() {
        println!("Pairing status response: 0x{:02X}", status);
    }
    Ok(())
}

pub fn cmd_dfu(dev: &device::IcarusDevice) -> Result<()> {
    println!(
        "{}",
        "Sending command to enter USB ISP/DFU firmware update mode..."
            .yellow()
            .bold()
    );
    dev.enter_firmware_update_mode()?;
    println!(
        "{} Device entered firmware upgrade mode.",
        "[OK]".green().bold()
    );
    Ok(())
}

pub fn cmd_udev_rule(install: bool) -> Result<()> {
    let rule = r#"# ICARUS V2 MINI Gaming Mouse — allow non-root HID access
# Place this file at /etc/udev/rules.d/99-icarus-mouse.rules

# USB wired
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f511", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f511", MODE="0666", TAG+="uaccess"

# 2.4G Wireless dongle
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f512", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f512", MODE="0666", TAG+="uaccess"

# 4G Wireless dongle
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f510", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f510", MODE="0666", TAG+="uaccess"
"#;

    if install {
        println!(
            "{}",
            "Installing udev rules to /etc/udev/rules.d/99-icarus-mouse.rules..."
                .cyan()
                .bold()
        );
        let target_path = "/etc/udev/rules.d/99-icarus-mouse.rules";
        let install_ok = std::process::Command::new("sudo")
            .args(["tee", target_path])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(rule.as_bytes())?;
                }
                child.wait()
            })
            .is_ok_and(|s| s.success())
            && std::process::Command::new("sudo")
                .args(["udevadm", "control", "--reload-rules"])
                .status()
                .is_ok_and(|s| s.success())
            && std::process::Command::new("sudo")
                .args(["udevadm", "trigger"])
                .status()
                .is_ok_and(|s| s.success());

        if install_ok {
            println!(
                "{} Udev rules installed and reloaded successfully!",
                "[OK]".green().bold()
            );
            println!("You can now use ICARUS V2 MINI without root permissions.");
            return Ok(());
        }
        println!(
            "{}",
            "Failed to run sudo command automatically. You can install it manually:".yellow()
        );
    }

    println!("{}", "udev rule for ICARUS V2 MINI".bold().cyan());
    println!("{}", "─".repeat(50));
    println!("{}", rule);
    println!("{}", "─".repeat(50));
    println!("\nTo install automatically:");
    println!("  sudo icarus-v2mini udev-rule --install");
    println!("\nOr install manually:");
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

pub fn cmd_flash(dev: &device::IcarusDevice, file: &str, dongle: bool) -> Result<()> {
    let path = std::path::Path::new(file);
    if !path.exists() {
        anyhow::bail!("Firmware file '{}' not found", file);
    }
    let target = if dongle {
        "4K Wireless Receiver Dongle"
    } else {
        "Mouse MCU"
    };
    println!(
        "{} Preparing ISP firmware flash for {} from {}",
        "[INFO]".cyan().bold(),
        target.yellow(),
        file.cyan()
    );
    let msg = dev.flash_firmware_file(path, dongle)?;
    println!("{} {}", "[OK]".green().bold(), msg.green().bold());
    Ok(())
}
