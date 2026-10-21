//! Device discovery and HID communication for ICARUS V2 MINI.
//!
//! Uses `hidapi` to find the mouse on Interface 1 (vendor-specific HID)
//! and send/receive 17-byte command packets.

use crate::config::{Color, DpiLed, LedMode, LiftOffDistance, MouseConfig, PollingRate};
use crate::dpi;
use crate::protocol::{Command, PACKET_SIZE, ProtocolError, Response};
use anyhow::{Context, Result, bail};
use std::time::Duration;

/// USB Vendor ID for ICARUS V2 MINI
const VID: u16 = 0x3554;

/// USB Product IDs
const PID_USB: u16 = 0xF511;
const PID_WIRELESS: u16 = 0xF512;
const PID_WIRELESS_4G: u16 = 0xF510;

/// The vendor-specific HID interface index used for configuration
const CONFIG_INTERFACE: i32 = 1;

/// Read timeout in milliseconds
const READ_TIMEOUT_MS: i32 = 500;

/// Maximum retries for read operations
const MAX_RETRIES: usize = 3;

/// Represents an open connection to an ICARUS V2 MINI mouse.
pub struct IcarusDevice {
    device: hidapi::HidDevice,
    /// Which PID was matched
    #[allow(dead_code)]
    pub product_id: u16,
    /// Connection type description
    pub connection: String,
}

impl IcarusDevice {
    /// Discover and open the ICARUS V2 MINI.
    ///
    /// Searches for USB, then wireless, then 4G wireless connections.
    pub fn open() -> Result<Self> {
        let api = hidapi::HidApi::new().context("Failed to initialize HIDAPI")?;

        // Try each PID in order of preference
        let candidates = [
            (PID_USB, "USB"),
            (PID_WIRELESS, "2.4G Wireless"),
            (PID_WIRELESS_4G, "4G Wireless"),
        ];

        for &(pid, connection_name) in &candidates {
            // Enumerate to find the correct interface
            let devices: Vec<_> = api
                .device_list()
                .filter(|d| d.vendor_id() == VID && d.product_id() == pid)
                .collect();

            if devices.is_empty() {
                continue;
            }

            // Find the vendor-specific interface (interface 1)
            for dev_info in &devices {
                if dev_info.interface_number() == CONFIG_INTERFACE {
                    match dev_info.open_device(&api) {
                        Ok(device) => {
                            // Set non-blocking for reads with timeout
                            device
                                .set_blocking_mode(true)
                                .context("Failed to set blocking mode")?;

                            return Ok(IcarusDevice {
                                device,
                                product_id: pid,
                                connection: connection_name.to_string(),
                            });
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Found device on {} but failed to open: {}",
                                connection_name, e
                            );
                            eprintln!("Hint: You may need to set up a udev rule. See README.md.");
                            continue;
                        }
                    }
                }
            }
        }

        bail!(
            "ICARUS V2 MINI not found.\n\
             Make sure the mouse is connected and you have permission to access HID devices.\n\
             \n\
             Searched for:\n\
             - USB:          VID=0x{:04X} PID=0x{:04X}\n\
             - 2.4G Wireless: VID=0x{:04X} PID=0x{:04X}\n\
             - 4G Wireless:  VID=0x{:04X} PID=0x{:04X}\n\
             \n\
             To grant access without root, create a udev rule:\n\
             sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/\n\
             sudo udevadm control --reload-rules\n\
             sudo udevadm trigger",
            VID,
            PID_USB,
            VID,
            PID_WIRELESS,
            VID,
            PID_WIRELESS_4G
        );
    }

    /// Send a command and receive a response.
    fn send_command(&self, cmd: &Command) -> Result<Response> {
        let packet = cmd.to_packet();

        // Write the command
        self.device
            .write(&packet)
            .map_err(|e| ProtocolError::Hid(e.to_string()))
            .context("Failed to write HID report")?;

        // Read the response with retries
        for attempt in 0..MAX_RETRIES {
            let mut buf = [0u8; PACKET_SIZE];

            match self.device.read_timeout(&mut buf, READ_TIMEOUT_MS) {
                Ok(n) if n >= PACKET_SIZE => {
                    match Response::from_packet(&buf) {
                        Ok(resp) if resp.is_read_response() => return Ok(resp),
                        Ok(_) => {
                            // Got a response but not the expected type, retry
                            if attempt < MAX_RETRIES - 1 {
                                continue;
                            }
                        }
                        Err(e) => {
                            if attempt < MAX_RETRIES - 1 {
                                eprintln!("Retrying after error: {}", e);
                                std::thread::sleep(Duration::from_millis(50));
                                continue;
                            }
                            return Err(e.into());
                        }
                    }
                }
                Ok(n) if n > 0 => {
                    // Partial read — try to parse just the report bytes
                    if n >= 17 {
                        // We may have received without report ID prefix
                        match Response::from_bytes(&buf[..17]) {
                            Ok(resp) if resp.is_read_response() => return Ok(resp),
                            _ => {
                                if attempt < MAX_RETRIES - 1 {
                                    continue;
                                }
                            }
                        }
                    }
                }
                Ok(_) => {
                    // Timeout (0 bytes)
                    if attempt < MAX_RETRIES - 1 {
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                }
                Err(e) => {
                    if attempt < MAX_RETRIES - 1 {
                        eprintln!("Read error (attempt {}): {}", attempt + 1, e);
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    return Err(ProtocolError::Hid(e.to_string()).into());
                }
            }
        }

        Err(ProtocolError::Timeout.into())
    }

    /// Send a write command (no response expected for writes).
    fn send_write(&self, cmd: &Command) -> Result<()> {
        let packet = cmd.to_packet();
        self.device
            .write(&packet)
            .map_err(|e| ProtocolError::Hid(e.to_string()))
            .context("Failed to write HID report")?;

        // Small delay after write to let the MCU process
        std::thread::sleep(Duration::from_millis(20));
        Ok(())
    }

    // ── Read operations ──────────────────────────────────────────────

    /// Read the current polling rate.
    pub fn read_polling_rate(&self) -> Result<PollingRate> {
        let resp = self.send_command(&Command::read_report_rate())?;
        let code = resp.payload().first().copied().unwrap_or(0x01);
        PollingRate::from_code(code)
            .ok_or_else(|| anyhow::anyhow!("Unknown polling rate code: 0x{:02X}", code))
    }

    /// Read the active DPI stage index (0-based).
    pub fn read_active_dpi_stage(&self) -> Result<u8> {
        let resp = self.send_command(&Command::read_current_dpi())?;
        Ok(resp.payload().first().copied().unwrap_or(0))
    }

    /// Read battery level (0–100).
    pub fn read_battery_level(&self) -> Result<u8> {
        let resp = self.send_command(&Command::read_battery())?;
        Ok(resp.payload().first().copied().unwrap_or(0))
    }

    /// Read firmware version string.
    pub fn read_firmware_version(&self) -> Result<String> {
        let resp = self.send_command(&Command::read_version())?;
        let payload = resp.payload();
        // Version is typically ASCII bytes
        let version = payload
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect::<String>();
        Ok(if version.is_empty() {
            format!(
                "{}.{}.{}.{}",
                payload.first().copied().unwrap_or(0),
                payload.get(1).copied().unwrap_or(0),
                payload.get(2).copied().unwrap_or(0),
                payload.get(3).copied().unwrap_or(0),
            )
        } else {
            version
        })
    }

    /// Read DPI LED settings.
    pub fn read_dpi_led(&self) -> Result<DpiLed> {
        let resp = self.send_command(&Command::read_dpi_led())?;
        let p = resp.payload();
        Ok(DpiLed {
            mode: LedMode::from_code(p.first().copied().unwrap_or(0)).unwrap_or(LedMode::Off),
            brightness: p.get(1).copied().unwrap_or(10),
            breath_speed: p.get(2).copied().unwrap_or(3),
            color: Color::new(
                p.get(3).copied().unwrap_or(255),
                p.get(4).copied().unwrap_or(255),
                p.get(5).copied().unwrap_or(255),
            ),
        })
    }

    // ── Write operations ─────────────────────────────────────────────

    /// Set the polling rate.
    pub fn set_polling_rate(&self, rate: PollingRate) -> Result<()> {
        self.send_write(&Command::write_report_rate(rate.to_code()))?;
        Ok(())
    }

    /// Set the active DPI stage (0-based index).
    pub fn set_active_dpi_stage(&self, stage: u8) -> Result<()> {
        if stage > 7 {
            bail!("DPI stage must be 0–7, got {}", stage);
        }
        self.send_write(&Command::write_current_dpi(stage))?;
        Ok(())
    }

    /// Set a DPI value for a specific stage.
    pub fn set_dpi_for_stage(&self, stage: u8, dpi_value: u16) -> Result<()> {
        if stage > 7 {
            bail!("DPI stage must be 0–7, got {}", stage);
        }
        dpi::validate_dpi(dpi_value).map_err(|e| anyhow::anyhow!(e))?;

        let snapped = dpi::snap_dpi(dpi_value);
        let (_, reg) = dpi::encode_dpi(snapped);

        // DPI stage config is at register address:
        // addr_hi = 0x01, addr_lo = 0x10 + stage
        // Payload: [x_reg, y_reg] (X and Y use same value for this mouse)
        let cmd = Command::write(0x01, 0x10 + stage, &[reg, reg]);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set DPI LED configuration.
    pub fn set_dpi_led(&self, led: &DpiLed) -> Result<()> {
        self.send_write(&Command::write_dpi_led(
            led.mode.to_code(),
            led.brightness,
            led.breath_speed,
            led.color.r,
            led.color.g,
            led.color.b,
        ))?;
        Ok(())
    }

    /// Set the lift-off distance.
    pub fn set_lod(&self, lod: LiftOffDistance) -> Result<()> {
        let cmd = Command::write(0x01, 0x08, &[lod.to_code()]);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set angle snapping on/off.
    pub fn set_angle_snapping(&self, enabled: bool) -> Result<()> {
        let cmd = Command::write(0x01, 0x09, &[if enabled { 0x01 } else { 0x00 }]);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set motion sync on/off.
    pub fn set_motion_sync(&self, enabled: bool) -> Result<()> {
        let cmd = Command::write(0x01, 0x0B, &[if enabled { 0x01 } else { 0x00 }]);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set debounce time.
    pub fn set_debounce(&self, ms: u8) -> Result<()> {
        let cmd = Command::write(0x01, 0x07, &[ms]);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set the LED color for a specific DPI stage.
    pub fn set_dpi_stage_color(&self, stage: u8, color: Color) -> Result<()> {
        if stage > 7 {
            bail!("DPI stage must be 0–7, got {}", stage);
        }
        // LED bar address: 0x01, 0x20 + stage * 3
        let cmd = Command::write(0x01, 0x20 + stage * 3, &[color.r, color.g, color.b]);
        self.send_write(&cmd)?;
        Ok(())
    }

    // ── Composite operations ─────────────────────────────────────────

    /// Read the complete current configuration from the mouse.
    pub fn read_config(&self) -> Result<MouseConfig> {
        let config = MouseConfig {
            polling_rate: self.read_polling_rate()?,
            active_dpi_stage: self.read_active_dpi_stage()?,
            dpi_led: self.read_dpi_led()?,
            ..Default::default()
        };

        Ok(config)
    }

    /// Apply a full configuration to the mouse.
    pub fn apply_config(&self, config: &MouseConfig) -> Result<()> {
        self.set_polling_rate(config.polling_rate)?;
        self.set_active_dpi_stage(config.active_dpi_stage)?;

        for (i, stage) in config.dpi_stages.iter().enumerate() {
            if stage.enabled {
                self.set_dpi_for_stage(i as u8, stage.dpi)?;
                self.set_dpi_stage_color(i as u8, stage.color)?;
            }
        }

        self.set_dpi_led(&config.dpi_led)?;
        self.set_lod(config.lod)?;
        self.set_angle_snapping(config.angle_snapping)?;
        self.set_motion_sync(config.motion_sync)?;
        self.set_debounce(config.debounce_ms)?;

        Ok(())
    }
}

/// List all connected ICARUS V2 MINI devices.
pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    let api = hidapi::HidApi::new().context("Failed to initialize HIDAPI")?;
    let mut found = Vec::new();

    let pids = [
        (PID_USB, "USB"),
        (PID_WIRELESS, "2.4G Wireless"),
        (PID_WIRELESS_4G, "4G Wireless"),
    ];

    for &(pid, conn) in &pids {
        for dev in api.device_list() {
            if dev.vendor_id() == VID
                && dev.product_id() == pid
                && dev.interface_number() == CONFIG_INTERFACE
            {
                found.push(DeviceInfo {
                    vendor_id: VID,
                    product_id: pid,
                    connection: conn.to_string(),
                    path: dev.path().to_string_lossy().into_owned(),
                    manufacturer: dev
                        .manufacturer_string()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    product: dev
                        .product_string()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                });
            }
        }
    }

    Ok(found)
}

/// Information about a discovered device.
#[derive(Debug)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub connection: String,
    pub path: String,
    pub manufacturer: String,
    pub product: String,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({}) [{:04X}:{:04X}]",
            self.manufacturer, self.product, self.connection, self.vendor_id, self.product_id
        )
    }
}
