//! Low-level register access for PAW3395 sensor and NRF52840 MCU features.

use super::{BatteryStatus, IcarusDevice, PID_USB};
use crate::config::{Color, DpiLed, LedMode, LiftOffDistance, PollingRate};
use crate::dpi;

use crate::protocol::{Command, addr};
use anyhow::{Result, bail};

impl IcarusDevice {
    /// Read the current USB/wireless polling rate from hardware registers.
    ///
    /// # Errors
    /// Returns an error if the read command fails or returns an unmapped rate code.
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

    /// Read the number of enabled DPI stages configured on the mouse (1..=8).
    pub fn read_max_dpi_stages(&self) -> Result<u8> {
        let resp = self.send_command(&Command::read_max_dpi_stages())?;
        let count = resp.payload().first().copied().unwrap_or(5).clamp(1, 8);
        Ok(count)
    }

    /// Read comprehensive battery status including charging state and voltage.
    pub fn read_battery_status(&self) -> Result<BatteryStatus> {
        let is_usb = self.product_id == PID_USB;
        match self.send_command(&Command::query_battery()) {
            Ok(resp) => {
                let lvl = resp.battery_level();
                let charging = is_usb || resp.is_charging();
                let mv = resp.battery_voltage_mv();
                if lvl > 0 && lvl <= 100 {
                    return Ok(BatteryStatus {
                        level: lvl,
                        is_charging: charging,
                        voltage_mv: mv,
                    });
                }
            }
            Err(e) => {
                // If a fatal HID error or hardware timeout occurred (e.g. USB unplugged or wireless mouse asleep),
                // return immediately instead of stalling for a second 500ms timeout.
                if let Some(
                    crate::protocol::ProtocolError::Hid(_)
                    | crate::protocol::ProtocolError::Timeout,
                ) = e.downcast_ref::<crate::protocol::ProtocolError>()
                {
                    return Err(e);
                }
            }
        }

        match self.send_command(&Command::read(
            addr::READ_BATTERY.0,
            addr::READ_BATTERY.1,
            1,
        )) {
            Ok(resp) => {
                let lvl = resp.payload().first().copied().unwrap_or(0);
                if lvl > 0 && lvl <= 100 {
                    return Ok(BatteryStatus {
                        level: lvl,
                        is_charging: is_usb,
                        voltage_mv: 0,
                    });
                }
            }
            Err(e) => {
                if let Some(crate::protocol::ProtocolError::Hid(_)) =
                    e.downcast_ref::<crate::protocol::ProtocolError>()
                {
                    return Err(e);
                }
            }
        }

        if is_usb {
            return Ok(BatteryStatus {
                level: 100,
                is_charging: true,
                voltage_mv: 4200,
            });
        }
        bail!("Battery level not reported yet")
    }

    /// Read battery level percentage (0–100).
    pub fn read_battery_level(&self) -> Result<u8> {
        self.read_battery_status().map(|s| s.level)
    }

    /// Read whether the mouse is actively charging or powered via USB.
    pub fn is_charging(&self) -> bool {
        self.read_battery_status()
            .map(|s| s.is_charging)
            .unwrap_or(self.product_id == PID_USB)
    }

    /// Read whether the mouse is actively online and connected.
    pub fn read_device_online(&self) -> Result<bool> {
        if self.product_id == PID_USB {
            return Ok(true);
        }
        match self.send_command(&Command::query_online()) {
            Ok(resp) => Ok(resp.is_device_online()),
            Err(e) => {
                if let Some(crate::protocol::ProtocolError::Hid(_)) =
                    e.downcast_ref::<crate::protocol::ProtocolError>()
                {
                    return Err(e);
                }
                Ok(false)
            }
        }
    }

    /// Reads the firmware version string from the MCU or receiver.
    ///
    /// # Errors
    /// Returns an error if reading the version command fails.
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
        self.commit_config(1)?;
        Ok(())
    }

    /// Set a DPI value for a specific stage (0x00, 0x0C + stage * 4).
    pub fn set_dpi_for_stage(&self, stage: u8, dpi_value: u16) -> Result<()> {
        if stage > 7 {
            bail!("DPI stage must be 0–7, got {}", stage);
        }
        dpi::validate_dpi(dpi_value)?;

        let snapped = dpi::snap_dpi(dpi_value);
        let (_, low, flag) = dpi::encode_dpi_10bit(snapped);

        let cmd = Command::write_dpi_stage_ext(stage, low, low, flag);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Set independent X and Y DPI values for a specific stage (0x00, 0x0C + stage * 4).
    pub fn set_dpi_for_stage_xy(&self, stage: u8, dpi_x: u16, dpi_y: u16) -> Result<()> {
        if stage > 7 {
            bail!("DPI stage must be 0–7, got {}", stage);
        }
        dpi::validate_dpi(dpi_x)?;
        dpi::validate_dpi(dpi_y)?;

        let (low_x, low_y, combined_flag) = dpi::encode_dpi_xy_10bit(dpi_x, dpi_y);

        let cmd = Command::write_dpi_stage_ext(stage, low_x, low_y, combined_flag);
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
        self.commit_config(1)?;
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

    /// Set debounce time in milliseconds (0–20ms, snapped to even 2ms hardware steps).
    pub fn set_debounce(&self, ms: u8) -> Result<()> {
        let clamped = (ms.min(20) / 2) * 2;
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

    /// Set Long Range wireless mode (RF power boost).
    pub fn set_long_range_mode(&self, enabled: bool) -> Result<()> {
        let cmd = Command::set_long_range_mode(enabled);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Read Long Range wireless mode status.
    pub fn read_long_range_mode(&self) -> Result<bool> {
        let cmd = Command::read_long_range_mode();
        let resp = self.send_command(&cmd)?;
        Ok(resp.payload().first().copied().unwrap_or(0) != 0)
    }

    /// Enter 2.4G dongle pairing mode (Match).
    pub fn pair_dongle(&self, cid: Option<u8>, mid: Option<u8>) -> Result<()> {
        let cid = cid.unwrap_or(0x51); // 0x51 = 81 from Config.ini CID="ODE="
        let mid = mid.unwrap_or(0x01); // 0x01 from Config.ini MID="01"
        let cmd = Command::enter_dongle_pair_with_cid_mid(cid, mid);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Read 2.4G dongle pairing status.
    pub fn read_dongle_pair_status(&self) -> Result<u8> {
        let cmd = Command::read_dongle_pair_status();
        let resp = self.send_command(&cmd)?;
        Ok(resp.payload().first().copied().unwrap_or(0))
    }

    /// Set 4K Dongle RGB configuration.
    pub fn set_4k_dongle_rgb(&self, payload: &[u8]) -> Result<()> {
        let cmd = Command::set_4k_dongle_rgb(payload);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Read 4K Dongle RGB configuration.
    pub fn read_4k_dongle_rgb(&self) -> Result<Vec<u8>> {
        let cmd = Command::read_4k_dongle_rgb();
        let resp = self.send_command(&cmd)?;
        Ok(resp.payload().to_vec())
    }

    /// Set Dongle RGB Bar mode.
    pub fn set_dongle_rgb_bar(&self, payload: &[u8]) -> Result<()> {
        let cmd = Command::set_dongle_rgb_bar_mode(payload);
        self.send_write(&cmd)?;
        Ok(())
    }

    /// Read Dongle RGB Bar mode.
    pub fn read_dongle_rgb_bar(&self) -> Result<Vec<u8>> {
        let cmd = Command::read_dongle_rgb_bar_mode();
        let resp = self.send_command(&cmd)?;
        Ok(resp.payload().to_vec())
    }
}
