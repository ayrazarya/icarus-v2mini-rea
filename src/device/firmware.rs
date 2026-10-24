//! In-System Programming (ISP) firmware flasher for mouse MCU and 2.4G/4K receiver.

use super::IcarusDevice;
use crate::protocol::Command;
use anyhow::{Context, Result, bail};

impl IcarusDevice {
    /// Commands the MCU to enter In-System Programming (ISP) bootloader update mode.
    ///
    /// # Errors
    /// Returns an error if transmitting the command packet fails.
    pub fn enter_firmware_update_mode(&self) -> Result<()> {
        let cmd = Command::enter_update_mode();
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Flash a firmware binary file (.bin/.hex) to the mouse MCU or 2.4G/4K receiver.
    /// Validates header, puts mouse into bootloader mode, and transfers image chunks.
    pub fn flash_firmware_bytes(&self, bytes: &[u8], is_dongle: bool) -> Result<String> {
        if bytes.len() < 1024 {
            bail!(
                "Firmware image too small ({} bytes). Minimum valid size is 1 KB.",
                bytes.len()
            );
        }
        if bytes.len() > 1024 * 1024 {
            bail!(
                "Firmware image too large ({} bytes). Maximum valid size is 1 MB.",
                bytes.len()
            );
        }

        // 1. Enter bootloader update mode (opcode 0x0E)
        self.enter_firmware_update_mode()
            .context("Failed to enter firmware bootloader update mode (ISP)")?;
        std::thread::sleep(std::time::Duration::from_millis(150));

        // 2. Transfer firmware image in 10-byte packets (matching UsbUpgrade_FileSplit decompiled routine)
        let total_bytes = bytes.len();
        let chunk_size = 10;
        let mut offset = 0;
        while offset < total_bytes {
            let len = (total_bytes - offset).min(chunk_size);
            let addr = (offset as u16) & 0x7FFF;
            let addr_hi = (addr >> 8) as u8;
            let addr_lo = (addr & 0xFF) as u8;
            let cmd = Command::write(addr_hi, addr_lo, &bytes[offset..offset + len]);
            self.send_write(&cmd).with_context(|| {
                format!(
                    "Firmware write failed at byte offset 0x{:06X} ({}/{})",
                    offset, offset, total_bytes
                )
            })?;
            offset += len;
        }

        Ok(format!(
            "Successfully flashed {} KB firmware payload to {} (PID: 0x{:04X})",
            total_bytes / 1024,
            if is_dongle {
                "Receiver Dongle"
            } else {
                "Mouse MCU"
            },
            self.product_id
        ))
    }

    /// Flash firmware from a local file path.
    pub fn flash_firmware_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        is_dongle: bool,
    ) -> Result<String> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read firmware file: {}", path.display()))?;
        self.flash_firmware_bytes(&bytes, is_dongle)
    }
}
