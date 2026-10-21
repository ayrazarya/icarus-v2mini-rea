//! Device discovery and HID communication for ICARUS V2 MINI.
//!
//! Uses `hidapi` to find the mouse on Interface 1 (vendor-specific HID)
//! and send/receive 17-byte command packets.

use crate::config::{Color, DpiLed, LedMode, LiftOffDistance, MouseConfig, PollingRate};
use crate::dpi;
use crate::protocol::{
    addr, Command, ProtocolError, Response, CMD_READ, CMD_RESPONSE, CMD_WRITE, REPORT_ID,
};
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

                            let dev = IcarusDevice {
                                device,
                                product_id: pid,
                                connection: connection_name.to_string(),
                            };
                            let _ = dev.send_pc_driver_status(true);
                            return Ok(dev);
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

    /// Send a command and receive a response matching the expected opcode.
    pub fn send_command(&self, cmd: &Command) -> Result<Response> {
        let packet = cmd.to_packet();
        let expected_cmd = cmd.data[1];

        // Drain any stale pending packets to keep the buffer synchronized
        let mut drain_buf = [0u8; 64];
        while let Ok(n) = self.device.read_timeout(&mut drain_buf, 5) {
            if n == 0 {
                break;
            }
        }

        // Write the command
        self.device
            .write(&packet)
            .map_err(|e| ProtocolError::Hid(e.to_string()))
            .context("Failed to write HID report")?;

        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(READ_TIMEOUT_MS as u64);

        while start.elapsed() < timeout {
            let mut buf = [0u8; 64];
            let remaining_ms = (timeout.saturating_sub(start.elapsed()).as_millis() as i32).max(10);
            match self.device.read_timeout(&mut buf, remaining_ms) {
                Ok(n) if n >= 17 => {
                    if let Ok(resp) = Response::from_packet(&buf[..n])
                        && resp.data[0] == REPORT_ID
                    {
                        let matches = if expected_cmd == CMD_READ {
                            (resp.data[1] == CMD_READ || resp.data[1] == CMD_RESPONSE)
                                && resp.data[3] == cmd.data[3]
                                && resp.data[4] == cmd.data[4]
                        } else {
                            resp.data[1] == expected_cmd
                        };
                        if matches {
                            return Ok(resp);
                        }
                    }
                    // Non-matching packet (e.g. async 0x0A or stale ACK) - continue waiting
                }
                Ok(_) => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => {
                    return Err(ProtocolError::Hid(e.to_string()).into());
                }
            }
        }

        Err(ProtocolError::Timeout.into())
    }

    /// Send a write command and cleanly consume acknowledgment (8ms pacing from HIDUsb.dll).
    pub fn send_write(&self, cmd: &Command) -> Result<()> {
        let packet = cmd.to_packet();
        let expected_cmd = cmd.data[1];

        self.device
            .write(&packet)
            .map_err(|e| ProtocolError::Hid(e.to_string()))
            .context("Failed to write HID report")?;

        // Read ACK packet with short timeout so input buffer remains clean
        let mut ack_buf = [0u8; 64];
        let start = std::time::Instant::now();
        let ack_timeout = Duration::from_millis(60);

        while start.elapsed() < ack_timeout {
            if let Ok(n) = self.device.read_timeout(&mut ack_buf, 20)
                && n >= 17
                && ack_buf[0] == REPORT_ID
                && (ack_buf[1] == expected_cmd || ack_buf[1] == CMD_WRITE)
            {
                break;
            }
        }

        // 8ms pacing delay between writes matching sub_1000c690 in HIDUsb.dll
        std::thread::sleep(Duration::from_millis(8));
        Ok(())
    }

    /// Commit and apply configuration profile to hardware (opcode 0x0F).
    pub fn commit_config(&self, profile: u8) -> Result<()> {
        self.send_write(&Command::apply_config(profile))
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

    /// Read battery level percentage (0–100).
    pub fn read_battery_level(&self) -> Result<u8> {
        if let Ok(resp) = self.send_command(&Command::query_battery()) {
            let lvl = resp.battery_level();
            if lvl > 0 && lvl <= 100 {
                return Ok(lvl);
            }
        }
        if let Ok(resp) = self.send_command(&Command::read(addr::READ_BATTERY.0, addr::READ_BATTERY.1, 1)) {
            let lvl = resp.payload().first().copied().unwrap_or(0);
            if lvl > 0 && lvl <= 100 {
                return Ok(lvl);
            }
        }
        bail!("Battery level not reported yet")
    }

    /// Read whether the mouse is actively online and connected.
    pub fn read_device_online(&self) -> Result<bool> {
        if self.product_id == PID_USB {
            return Ok(true);
        }
        match self.send_command(&Command::query_online()) {
            Ok(resp) => Ok(resp.is_device_online()),
            Err(_) => {
                // If query command drops, assume device is online to avoid false lockouts
                Ok(true)
            }
        }
    }

    /// Notify MCU that PC Driver is running.
    pub fn send_pc_driver_status(&self, active: bool) -> Result<()> {
        self.send_write(&Command::pc_driver_status(active))
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

    /// Read lift-off distance (LOD).
    pub fn read_lod(&self) -> Result<LiftOffDistance> {
        let resp = self.send_command(&Command::read_lod())?;
        let code = resp.payload().first().copied().unwrap_or(0x00);
        Ok(LiftOffDistance::from_code(code).unwrap_or(LiftOffDistance::Low))
    }

    /// Read debounce time in milliseconds.
    pub fn read_debounce(&self) -> Result<u8> {
        let resp = self.send_command(&Command::read_debounce())?;
        Ok(resp.payload().first().copied().unwrap_or(4).min(20))
    }

    /// Read angle snapping.
    pub fn read_angle_snapping(&self) -> Result<bool> {
        let resp = self.send_command(&Command::read_angle_snap())?;
        Ok(resp.payload().first().copied().unwrap_or(0x00) == 0x01)
    }

    /// Read motion sync.
    pub fn read_motion_sync(&self) -> Result<bool> {
        let resp = self.send_command(&Command::read_motion_sync())?;
        Ok(resp.payload().first().copied().unwrap_or(0x00) == 0x01)
    }

    /// Read ripple control (high DPI smoothing).
    pub fn read_ripple_control(&self) -> Result<bool> {
        let resp = self.send_command(&Command::read_ripple())?;
        Ok(resp.payload().first().copied().unwrap_or(0x00) == 0x01)
    }

    /// Read sensor performance mode (true = Performance, false = Office).
    pub fn read_performance_mode(&self) -> Result<bool> {
        let resp = self.send_command(&Command::read_sensor_mode())?;
        // 0x00 = Performance, 0x01 = Office
        Ok(resp.payload().first().copied().unwrap_or(0x00) == 0x00)
    }

    /// Read sensor sleep timer in seconds.
    pub fn read_sensor_sleep(&self) -> Result<u32> {
        let resp = self.send_command(&Command::read_sensor_sleep())?;
        let code = resp.payload().first().copied().unwrap_or(0x00);
        Ok((code as u32) * 10)
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
        let _ = self.commit_config(1);
        Ok(())
    }

    /// Set a DPI value for a specific stage (0x01, 0x2C + stage * 4).
    pub fn set_dpi_for_stage(&self, stage: u8, dpi_value: u16) -> Result<()> {
        if stage > 7 {
            bail!("DPI stage must be 0–7, got {}", stage);
        }
        dpi::validate_dpi(dpi_value).map_err(|e| anyhow::anyhow!(e))?;

        let snapped = dpi::snap_dpi(dpi_value);
        let (_, reg) = dpi::encode_dpi(snapped);

        let cmd = Command::write_dpi_stage(stage, reg, reg);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set DPI LED configuration.
    pub fn set_dpi_led(&self, led: &DpiLed) -> Result<()> {
        for cmd in Command::write_dpi_led_commands(
            led.mode.to_code(),
            led.brightness,
            led.breath_speed,
            led.color.r,
            led.color.g,
            led.color.b,
        ) {
            self.send_write(&cmd)?;
        }
        let _ = self.commit_config(1);
        Ok(())
    }

    /// Set the lift-off distance.
    pub fn set_lod(&self, lod: LiftOffDistance) -> Result<()> {
        let cmd = Command::write_lod(lod.to_code());
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set sensor performance mode (true = Performance, false = Office).
    pub fn set_performance_mode(&self, performance: bool) -> Result<()> {
        let cmd = Command::write_sensor_mode(performance);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set angle snapping on/off.
    pub fn set_angle_snapping(&self, enabled: bool) -> Result<()> {
        let cmd = Command::write_angle_snap(enabled);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set motion sync on/off.
    pub fn set_motion_sync(&self, enabled: bool) -> Result<()> {
        let cmd = Command::write_motion_sync(enabled);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set ripple control on/off.
    pub fn set_ripple_control(&self, enabled: bool) -> Result<()> {
        let cmd = Command::write_ripple(enabled);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set debounce time in milliseconds (0–20ms).
    pub fn set_debounce(&self, ms: u8) -> Result<()> {
        let clamped = ms.min(20);
        let cmd = Command::write_debounce(clamped);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set sensor idle sleep timer in seconds.
    pub fn set_sensor_sleep(&self, seconds: u32) -> Result<()> {
        let cmd = Command::write_sensor_sleep(seconds);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set the button mapping for a specific button index (0–5).
    pub fn set_button_mapping(&self, idx: u8, action: &crate::config::ButtonAction) -> Result<()> {
        if idx > 5 {
            bail!("Button index must be 0–5, got {}", idx);
        }
        let cmd = Command::write_button(idx, action.to_bytes());
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set the LED color for a specific DPI stage.
    pub fn set_dpi_stage_color(&self, stage: u8, color: Color) -> Result<()> {
        if stage > 7 {
            bail!("DPI stage must be 0–7, got {}", stage);
        }
        let cmd = Command::write_dpi_stage_color(stage, color.r, color.g, color.b);
        self.send_write(&cmd)?;
        Ok(())
    }

    // ── Composite operations ─────────────────────────────────────────

    /// Read the complete current configuration from the mouse.
    ///
    /// Uses graceful fallbacks for registers to avoid failing if a single packet drops.
    pub fn read_config(&self) -> Result<MouseConfig> {
        let mut config = MouseConfig::default();

        if let Ok(rate) = self.read_polling_rate() {
            config.polling_rate = rate;
        }
        if let Ok(stage) = self.read_active_dpi_stage()
            && (stage as usize) < config.dpi_stages.len()
        {
            config.active_dpi_stage = stage;
        }

        // Read all 8 DPI stages and colors from hardware registers
        for i in 0..config.dpi_stages.len().min(8) {
            // DPI Stage Value (0x00, 0x0C + i*4, length 4)
            if let Ok(resp) = self.send_command(&Command::read(0x00, addr::DPI_STAGE_BASE + (i as u8) * 4, 4)) {
                let p = resp.payload();
                if let Some(&x_reg) = p.first() {
                    let dpi_val = dpi::decode_dpi(x_reg);
                    if (50..=26000).contains(&dpi_val) {
                        config.dpi_stages[i].dpi = dpi_val;
                    }
                }
            }
            // DPI Stage Color (0x00, 0x2C + i*4, length 4)
            if let Ok(resp) = self.send_command(&Command::read(0x00, addr::DPI_COLOR_BASE + (i as u8) * 4, 4)) {
                let p = resp.payload();
                if p.len() >= 3 {
                    config.dpi_stages[i].color = Color::new(p[0], p[1], p[2]);
                }
            }
        }

        if let Ok(led) = self.read_dpi_led() {
            config.dpi_led = led;
        }
        if let Ok(lod) = self.read_lod() {
            config.lod = lod;
        }
        if let Ok(perf) = self.read_performance_mode() {
            config.performance_mode = perf;
        }
        if let Ok(angle) = self.read_angle_snapping() {
            config.angle_snapping = angle;
        }
        if let Ok(sync) = self.read_motion_sync() {
            config.motion_sync = sync;
        }
        if let Ok(ripple) = self.read_ripple_control() {
            config.ripple_control = ripple;
        }
        if let Ok(debounce) = self.read_debounce() {
            config.debounce_ms = debounce;
        }
        if let Ok(sleep) = self.read_sensor_sleep() {
            config.sensor_sleep_seconds = sleep;
        }

        config.validate();
        Ok(config)
    }

    /// Apply a full configuration to the mouse registers.
    pub fn apply_config(&self, config: &MouseConfig) -> Result<()> {
        let mut cfg = config.clone();
        cfg.validate();

        // 1. Notify MCU that PC Driver is running
        let _ = self.send_pc_driver_status(true);

        // 2. Set polling rate
        self.set_polling_rate(cfg.polling_rate)?;

        // 3. Set max DPI stages count
        let enabled_count = cfg.dpi_stages.iter().filter(|s| s.enabled).count().clamp(1, 8) as u8;
        let _ = self.send_write(&Command::write_max_dpi_stages(enabled_count));

        // 4. Set active DPI stage
        self.set_active_dpi_stage(cfg.active_dpi_stage)?;

        // 5. Write all 8 DPI stages values & colors
        for (i, stage) in cfg.dpi_stages.iter().enumerate().take(8) {
            self.set_dpi_for_stage(i as u8, stage.dpi)?;
            self.set_dpi_stage_color(i as u8, stage.color)?;
        }

        // 6. Write composite DPI LED settings
        self.set_dpi_led(&cfg.dpi_led)?;

        // 7. Write sensor parameters
        self.set_lod(cfg.lod)?;
        self.set_performance_mode(cfg.performance_mode)?;
        self.set_angle_snapping(cfg.angle_snapping)?;
        self.set_motion_sync(cfg.motion_sync)?;
        self.set_ripple_control(cfg.ripple_control)?;
        self.set_debounce(cfg.debounce_ms)?;
        self.set_sensor_sleep(cfg.sensor_sleep_seconds)?;

        // 8. Write button mappings
        for (i, btn) in cfg.buttons.iter().enumerate().take(6) {
            self.set_button_mapping(i as u8, btn)?;
        }

        // 9. Commit and apply active configuration profile (opcode 0x0F)
        self.commit_config(1)?;

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
