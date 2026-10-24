//! Complete mouse configuration data model, safety validation, and TOML persistence.

use super::buttons::ButtonAction;
use super::dpi::{DpiStage, default_dpi_stages};
use super::lighting::{Color, DongleRgbConfig, DpiLed, LedMode};
use super::macros::{MacroItem, default_macros};
use super::rules::{AutoProfileRule, default_auto_rules};
use super::sensor::{LiftOffDistance, PollingRate};
use crate::i18n::AppLanguage;
use serde::{Deserialize, Serialize};

fn default_performance_mode() -> bool {
    true
}

fn default_sensor_sleep() -> u32 {
    60
}

fn default_led_sleep() -> u32 {
    30
}

fn default_active_profile() -> u8 {
    1
}

fn default_false() -> bool {
    false
}

fn default_auto_profile_enabled() -> bool {
    false
}

fn default_long_range_mode() -> bool {
    true
}

/// Complete mouse configuration state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MouseConfig {
    /// Active profile (1 to 5)
    #[serde(default = "default_active_profile")]
    pub active_profile: u8,
    /// Polling rate
    pub polling_rate: PollingRate,
    /// Active DPI stage index (0-based)
    pub active_dpi_stage: u8,
    /// DPI stages (up to 8)
    #[serde(default = "default_dpi_stages")]
    pub dpi_stages: Vec<DpiStage>,
    /// DPI LED configuration
    pub dpi_led: DpiLed,
    /// Lift-off distance
    pub lod: LiftOffDistance,
    /// Performance Mode (high response) vs Office Mode (power save)
    #[serde(default = "default_performance_mode")]
    pub performance_mode: bool,
    /// Sensor idle sleep timer in seconds (0 = Never)
    #[serde(default = "default_sensor_sleep")]
    pub sensor_sleep_seconds: u32,
    /// LED idle sleep timer in seconds (0 = Never)
    #[serde(default = "default_led_sleep")]
    pub led_sleep_seconds: u32,
    /// Angle snapping (straightening)
    pub angle_snapping: bool,
    /// Ripple control
    pub ripple_control: bool,
    /// Motion sync
    pub motion_sync: bool,
    /// Debounce time in ms
    pub debounce_ms: u8,
    /// Button mappings (6 buttons)
    pub buttons: Vec<ButtonAction>,
    /// Long Range Wireless Mode (RF transmission power boost)
    #[serde(default = "default_long_range_mode")]
    pub long_range_mode: bool,
    /// Whether automatic profile switching based on foreground application is active
    #[serde(default = "default_auto_profile_enabled")]
    pub auto_profile_enabled: bool,
    /// Rules for per-application auto-profile switching
    #[serde(default = "default_auto_rules")]
    pub auto_profile_rules: Vec<AutoProfileRule>,
    /// Stored macro library
    #[serde(default = "default_macros")]
    pub macros: Vec<MacroItem>,
    /// Whether X and Y DPI axes are configured independently
    #[serde(default = "default_false")]
    pub separate_xy_dpi: bool,
    /// 4K Wireless Receiver Dongle RGB configuration
    #[serde(default)]
    pub dongle_rgb: DongleRgbConfig,
    /// UI display language (English or Indonesian)
    #[serde(default)]
    pub language: AppLanguage,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            active_profile: 1,
            polling_rate: PollingRate::Hz1000,
            active_dpi_stage: 1,
            dpi_stages: default_dpi_stages(),
            dpi_led: DpiLed {
                mode: LedMode::Solid,
                brightness: 10,
                breath_speed: 3,
                color: Color::WHITE,
            },
            lod: LiftOffDistance::Low,
            performance_mode: true,
            sensor_sleep_seconds: 60,
            led_sleep_seconds: 30,
            angle_snapping: false,
            ripple_control: false,
            motion_sync: true,
            debounce_ms: 4,
            buttons: vec![
                ButtonAction::MouseButton(0x01), // Button 1: Left Click
                ButtonAction::MouseButton(0x02), // Button 2: Right Click
                ButtonAction::MouseButton(0x04), // Button 3: Middle Click
                ButtonAction::MouseButton(0x10), // Button 4: Forward
                ButtonAction::MouseButton(0x08), // Button 5: Back
                ButtonAction::DpiSwitch,         // Button 6: DPI button
            ],
            long_range_mode: true,
            auto_profile_enabled: false,
            auto_profile_rules: default_auto_rules(),
            macros: default_macros(),
            separate_xy_dpi: false,
            dongle_rgb: DongleRgbConfig::default(),
            language: AppLanguage::default(),
        }
    }
}

impl MouseConfig {
    /// Validates and sanitizes the configuration, clamping all values into valid hardware ranges.
    pub fn validate(&mut self) {
        // Active profile: 1..=5
        if self.active_profile < 1 || self.active_profile > 5 {
            self.active_profile = 1;
        }

        // Debounce: 0..=20 ms, snapped to even milliseconds
        if self.debounce_ms > 20 {
            self.debounce_ms = 20;
        }
        self.debounce_ms = (self.debounce_ms / 2) * 2;

        // Ensure buttons has exactly 6 buttons
        let default_buttons = [
            ButtonAction::MouseButton(0x01),
            ButtonAction::MouseButton(0x02),
            ButtonAction::MouseButton(0x04),
            ButtonAction::MouseButton(0x10),
            ButtonAction::MouseButton(0x08),
            ButtonAction::DpiSwitch,
        ];
        while self.buttons.len() < 6 {
            let idx = self.buttons.len();
            self.buttons.push(default_buttons[idx]);
        }
        self.buttons.truncate(6);

        // Safety Lock: Ensure at least one button is Left Click (decompiled from `leftbuttononly.obj`)
        if !self.buttons.contains(&ButtonAction::MouseButton(1)) {
            self.buttons[0] = ButtonAction::MouseButton(1);
        }

        // Ensure dpi_stages is not empty and has at most 8 stages
        if self.dpi_stages.is_empty() {
            self.dpi_stages = default_dpi_stages();
        }
        if self.dpi_stages.len() > 8 {
            self.dpi_stages.truncate(8);
        }

        // Validate each DPI stage value and auto-enable separate_xy_dpi if any stage defines distinct Y-axis DPI
        for stage in &mut self.dpi_stages {
            stage.dpi = crate::dpi::snap_dpi(stage.dpi);
            if let Some(y) = stage.dpi_y {
                let snapped_y = crate::dpi::snap_dpi(y);
                stage.dpi_y = Some(snapped_y);
                if snapped_y != stage.dpi {
                    self.separate_xy_dpi = true;
                }
            }
        }

        // Ensure at least one DPI stage is enabled
        let has_enabled = self.dpi_stages.iter().any(|s| s.enabled);
        if !has_enabled {
            self.dpi_stages[0].enabled = true;
        }

        // Active DPI stage must point to an enabled stage within bounds
        if self.active_dpi_stage as usize >= self.dpi_stages.len()
            || !self.dpi_stages[self.active_dpi_stage as usize].enabled
        {
            if let Some(first_enabled) = self.dpi_stages.iter().position(|s| s.enabled) {
                self.active_dpi_stage = first_enabled as u8;
            } else {
                self.dpi_stages[0].enabled = true;
                self.active_dpi_stage = 0;
            }
        }

        // DPI LED brightness: 0..=10
        if self.dpi_led.brightness > 10 {
            self.dpi_led.brightness = 10;
        }

        // DPI LED breath speed: 1..=10
        self.dpi_led.breath_speed = self.dpi_led.breath_speed.clamp(1, 10);

        // 4K Dongle RGB brightness: 0..=10, speed: 1..=10, bar_mode: 0..=2
        if self.dongle_rgb.brightness > 10 {
            self.dongle_rgb.brightness = 10;
        }
        self.dongle_rgb.speed = self.dongle_rgb.speed.clamp(1, 10);
        if self.dongle_rgb.bar_mode > 2 {
            self.dongle_rgb.bar_mode = 0;
        }
        if self.dongle_rgb.mode == LedMode::Off {
            self.dongle_rgb.enabled = false;
        }

        // Sleep timers (max 600s / 10 minutes)
        if self.sensor_sleep_seconds > 600 {
            self.sensor_sleep_seconds = 600;
        }
        if self.led_sleep_seconds > 600 {
            self.led_sleep_seconds = 600;
        }

        // Macros list: ensure at least 1 macro exists and does not exceed hardware capacity
        if self.macros.is_empty() {
            self.macros = default_macros();
        }
        if self.macros.len() > crate::protocol::MAX_MACRO_SLOTS {
            self.macros.truncate(crate::protocol::MAX_MACRO_SLOTS);
        }
        for mac in &mut self.macros {
            if mac.name.trim().is_empty() {
                mac.name = "Macro".to_string();
            }
            if let super::macros::MacroRepeatMode::Times(n) = &mut mac.repeat_mode {
                *n = (*n).clamp(1, 255);
            }
            mac.default_delay_ms = mac.default_delay_ms.clamp(1, u16::MAX as u32);
            if mac.events.len() > 64 {
                mac.events.truncate(64);
            }
            for ev in &mut mac.events {
                if ev.event_type == super::macros::MacroEventType::Delay {
                    ev.key.clear();
                    ev.delay_ms = ev.delay_ms.clamp(1, u16::MAX as u32);
                } else {
                    ev.delay_ms = ev.delay_ms.min(u16::MAX as u32);
                }
            }
        }

        // Auto-profile rules: clamp target profile index to valid hardware slots (1..=5)
        for rule in &mut self.auto_profile_rules {
            rule.profile_index = rule.profile_index.clamp(1, 5);
        }

        // Ensure button macro slots and FireKey bounds remain valid
        let max_macro_slot = (self.macros.len() - 1) as u8;
        for btn in &mut self.buttons {
            match btn {
                ButtonAction::Macro(slot) if *slot > max_macro_slot => {
                    *slot = max_macro_slot;
                }
                ButtonAction::FireKey { clicks, delay_ms } => {
                    *clicks = (*clicks).clamp(2, 15);
                    *delay_ms = (*delay_ms).max(5);
                }
                _ => {}
            }
        }
    }

    /// Validates button assignments enforcing Left Click Safety Lock (matching `leftbuttononly.obj`).
    /// Returns an error if no button is configured for Left Click (MouseButton 1).
    pub fn validate_buttons(&self) -> Result<(), String> {
        if !self.buttons.contains(&ButtonAction::MouseButton(1)) {
            let msg = match self.language {
                AppLanguage::English => {
                    "Safety Lock: At least one button must be assigned to Left Click (Button 1) to prevent user lockout."
                }
                AppLanguage::Indonesian => {
                    "Kunci Pengaman: Minimal satu tombol harus diatur sebagai Klik Kiri (Tombol 1) guna mencegah penguncian mouse."
                }
            };
            return Err(msg.to_string());
        }
        Ok(())
    }

    /// Serializes configuration to a formatted TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserializes and validates configuration from a TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        let mut cfg: Self = toml::from_str(s)?;
        cfg.validate();
        Ok(cfg)
    }

    /// Reads and deserializes configuration from a file path.
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut cfg: Self = toml::from_str(&content)?;
        cfg.validate();
        Ok(cfg)
    }

    /// Serializes and atomically writes configuration to a file path.
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();
        let content = self.to_toml()?;
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let tmp_path = path.with_extension("toml.tmp");
        if std::fs::write(&tmp_path, &content).is_ok() {
            if std::fs::rename(&tmp_path, path).is_ok() {
                return Ok(());
            }
            let _ = std::fs::remove_file(&tmp_path);
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}
