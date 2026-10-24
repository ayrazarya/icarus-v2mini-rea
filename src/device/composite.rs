//! High-level atomic profile operations (read_config and apply_config).

use super::IcarusDevice;
use crate::config::{Color, LedMode, MouseConfig};
use crate::dpi;
use crate::protocol::{Command, addr};
use anyhow::Result;

impl IcarusDevice {
    /// Applies RGB lighting configuration to the connected 2.4G/4K wireless receiver dongle.
    ///
    /// # Errors
    /// Returns an error if transmitting the dongle lighting packet fails.
    pub fn apply_dongle_rgb(&self, cfg: &crate::config::DongleRgbConfig) -> Result<()> {
        let mode_code = if cfg.enabled {
            cfg.mode.to_code()
        } else {
            0x00
        };
        let payload = [
            mode_code,
            cfg.brightness,
            cfg.speed,
            cfg.color.r,
            cfg.color.g,
            cfg.color.b,
            0x00,
            0x00,
            0x00,
            0x00,
        ];
        self.set_4k_dongle_rgb(&payload)?;
        self.set_dongle_rgb_bar(&[cfg.bar_mode, 0x00])?;
        Ok(())
    }

    /// Read the complete current configuration from the mouse.
    ///
    /// Uses graceful fallbacks for registers to avoid failing if a single packet drops,
    /// while failing fast if the wireless mouse is asleep/offline or all register queries time out.
    pub fn read_config(&self) -> Result<MouseConfig> {
        if !self.read_device_online()? {
            anyhow::bail!("Mouse is offline or asleep (2.4G wireless link inactive)");
        }

        let mut config = MouseConfig::default();
        let mut any_read_ok = false;

        if let Ok(rate) = self.read_polling_rate() {
            config.polling_rate = rate;
            any_read_ok = true;
        }
        if let Ok(long_range) = self.read_long_range_mode() {
            config.long_range_mode = long_range;
            any_read_ok = true;
        }
        if let Ok(stage) = self.read_active_dpi_stage()
            && (stage as usize) < config.dpi_stages.len()
        {
            config.active_dpi_stage = stage;
            any_read_ok = true;
        }

        // If none of the initial queries responded, fail fast instead of stalling on 32 more timeouts
        if !any_read_ok {
            anyhow::bail!(
                "Failed to read configuration from device: initial register queries timed out"
            );
        }

        // Query number of enabled stages from hardware (default to 5 if unreachable)
        let max_stages =
            self.read_max_dpi_stages()
                .unwrap_or_else(|_| config.dpi_stages.len().clamp(1, 8) as u8) as usize;

        // Read all 8 DPI stages and colors from hardware registers
        for i in 0..8 {
            // DPI Stage Value (0x00, 0x0C + i*4, length 4: [x_reg, y_reg, flag, chk])
            if let Ok(resp) = self.send_command(&Command::read(
                0x00,
                addr::DPI_STAGE_BASE + (i as u8) * 4,
                4,
            )) {
                let p = resp.payload();
                if let Some(&x_reg) = p.first() {
                    let y_reg = p.get(1).copied().unwrap_or(x_reg);
                    let flag = p.get(2).copied().unwrap_or(0);
                    let (dpi_val, dpi_y_val) = dpi::decode_dpi_xy_10bit(x_reg, y_reg, flag);

                    // Ensure config.dpi_stages has an entry for stage i
                    while config.dpi_stages.len() <= i {
                        config.dpi_stages.push(crate::config::DpiStage {
                            dpi: dpi_val,
                            dpi_y: None,
                            enabled: i < max_stages,
                            color: Color::WHITE,
                        });
                    }

                    if (dpi::MIN_DPI..=dpi::MAX_DPI).contains(&dpi_val) {
                        config.dpi_stages[i].dpi = dpi_val;
                    }

                    // Record independent Y-axis DPI if it differs from X-axis DPI
                    if (dpi::MIN_DPI..=dpi::MAX_DPI).contains(&dpi_y_val)
                        && (dpi_y_val != dpi_val || y_reg != x_reg)
                    {
                        config.dpi_stages[i].dpi_y = Some(dpi_y_val);
                        config.separate_xy_dpi = true;
                    }
                    config.dpi_stages[i].enabled = i < max_stages;
                }
            }

            // DPI Stage Color (0x00, 0x2C + i*4, length 4: [r, g, b, chk])
            if let Ok(resp) = self.send_command(&Command::read(
                0x00,
                addr::DPI_COLOR_BASE + (i as u8) * 4,
                4,
            )) {
                let p = resp.payload();
                if p.len() >= 3 && i < config.dpi_stages.len() {
                    config.dpi_stages[i].color = Color::new(p[0], p[1], p[2]);
                }
            }
        }

        if let Ok(led) = self.read_dpi_led() {
            config.dpi_led = led;
        }

        // Also check direct DPI LED registers (0x00, 0x4C, len 8)
        if let Ok(resp) = self.send_command(&Command::read(
            addr::DPI_LED_MODE.0,
            addr::DPI_LED_MODE.1,
            8,
        )) {
            let p = resp.payload();
            if p.len() >= 6 {
                if let Some(mode) = LedMode::from_code(p[0]) {
                    config.dpi_led.mode = mode;
                }
                let scaled_bright = ((p[2] as u16 * 10 + 127) / 255).clamp(1, 10) as u8;
                config.dpi_led.brightness = scaled_bright;
                config.dpi_led.breath_speed = p[4].clamp(1, 10);
            }
        }

        // On ICARUS V2 MINI, the mouse lamp is the DPI indicator.
        // Synchronize active stage color into dpi_led.color.
        let active_idx =
            (config.active_dpi_stage as usize).min(config.dpi_stages.len().saturating_sub(1));
        if let Some(active_stage) = config.dpi_stages.get(active_idx) {
            config.dpi_led.color = active_stage.color;
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

        // Read all 6 button mappings from hardware registers (0x00, 0x60 + idx*4)
        for i in 0..6 {
            if let Ok(resp) =
                self.send_command(&Command::read(0x00, addr::BUTTON_BASE + (i as u8) * 4, 4))
            {
                let p = resp.payload();
                if p.len() >= 3 {
                    let action = crate::config::ButtonAction::from_bytes([p[0], p[1], p[2]]);
                    if i < config.buttons.len() {
                        config.buttons[i] = action;
                    } else {
                        config.buttons.push(action);
                    }
                }
            }
        }

        config.validate();
        Ok(config)
    }

    /// Apply a full configuration to the mouse registers.
    pub fn apply_config(&self, config: &MouseConfig) -> Result<()> {
        if !self.read_device_online()? {
            let _ = self.apply_dongle_rgb(&config.dongle_rgb);
            anyhow::bail!("Mouse is offline or asleep (2.4G wireless link inactive)");
        }

        // 1. Notify MCU that PC Driver is running
        let _ = self.send_pc_driver_status(true);

        // 2. Set polling rate
        self.set_polling_rate(config.polling_rate)?;

        // 3. Set max DPI stages count
        let enabled_count = config
            .dpi_stages
            .iter()
            .filter(|s| s.enabled)
            .count()
            .clamp(1, 8) as u8;
        self.send_write(&Command::write_max_dpi_stages(enabled_count))?;

        // 4. Set active DPI stage register (committed atomically at step 9 with active_profile)
        let active_stage = if (config.active_dpi_stage as usize) < config.dpi_stages.len() {
            config.active_dpi_stage.min(7)
        } else {
            0
        };
        self.send_write(&Command::write_current_dpi(active_stage))?;

        // 5. Write all 8 DPI stages values & colors
        for (i, stage) in config.dpi_stages.iter().enumerate().take(8) {
            if config.separate_xy_dpi {
                self.set_dpi_for_stage_xy(i as u8, stage.effective_x(), stage.effective_y())?;
            } else {
                self.set_dpi_for_stage(i as u8, stage.dpi)?;
            }
            self.set_dpi_stage_color(i as u8, stage.color)?;
        }

        // 6. Write composite and direct DPI LED settings
        // The mouse LED indicates the active DPI stage, so use active stage color.
        let active_idx = (active_stage as usize).min(config.dpi_stages.len().saturating_sub(1));
        let active_color = config
            .dpi_stages
            .get(active_idx)
            .map(|s| s.color)
            .unwrap_or(config.dpi_led.color);

        for cmd in Command::write_dpi_led_commands(
            config.dpi_led.mode.to_code(),
            config.dpi_led.brightness.min(10),
            config.dpi_led.breath_speed.clamp(1, 10),
            active_color.r,
            active_color.g,
            active_color.b,
        ) {
            self.send_write(&cmd)?;
        }

        // 7. Write sensor parameters
        self.set_lod(config.lod)?;
        self.set_performance_mode(config.performance_mode)?;
        self.set_angle_snapping(config.angle_snapping)?;
        self.set_motion_sync(config.motion_sync)?;
        self.set_ripple_control(config.ripple_control)?;
        self.set_debounce((config.debounce_ms.min(20) / 2) * 2)?;
        self.set_sensor_sleep(config.sensor_sleep_seconds.min(600))?;
        let _ = self.set_long_range_mode(config.long_range_mode);

        // 8. Write button mappings
        for (i, btn) in config.buttons.iter().enumerate().take(6) {
            self.set_button_mapping(i as u8, btn)?;
        }

        // 9. Commit and apply active configuration profile (opcode 0x0F)
        self.commit_config(config.active_profile.clamp(1, 5))?;

        // 10. Flash assigned macros to on-board hardware EEPROM (each unique bound slot flashed once)
        let mut flashed_slots = 0u16;
        for btn in &config.buttons {
            if let crate::config::ButtonAction::Macro(slot) = btn
                && (*slot as usize) < crate::protocol::MAX_MACRO_SLOTS
            {
                let mask = 1u16 << *slot;
                if (flashed_slots & mask) == 0
                    && let Some(macro_item) = config.macros.get(*slot as usize)
                {
                    self.flash_macro_slot(*slot, &macro_item.name, &macro_item.events)?;
                    flashed_slots |= mask;
                }
            }
        }

        // 11. Apply 4K Dongle RGB configuration if connected
        let _ = self.apply_dongle_rgb(&config.dongle_rgb);

        Ok(())
    }
}
