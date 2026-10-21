//! Mouse configuration types for the ICARUS V2 MINI.
//!
//! These types represent the parsed configuration state that maps
//! to the FlashDataMap structure in the mouse firmware.

use serde::{Deserialize, Serialize};

/// Polling rate setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PollingRate {
    Hz125,
    Hz250,
    Hz500,
    Hz1000,
    Hz2000,
    Hz4000,
}

impl PollingRate {
    /// Convert to the wire protocol byte value.
    pub fn to_code(self) -> u8 {
        match self {
            PollingRate::Hz125 => 0x08,
            PollingRate::Hz250 => 0x04,
            PollingRate::Hz500 => 0x02,
            PollingRate::Hz1000 => 0x01,
            PollingRate::Hz2000 => 0x00,
            PollingRate::Hz4000 => 0x10,
        }
    }

    /// Parse from the wire protocol byte value.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x08 => Some(PollingRate::Hz125),
            0x04 => Some(PollingRate::Hz250),
            0x02 => Some(PollingRate::Hz500),
            0x01 => Some(PollingRate::Hz1000),
            0x00 => Some(PollingRate::Hz2000),
            0x10 => Some(PollingRate::Hz4000),
            _ => None,
        }
    }

    /// Get the rate in Hz.
    pub fn hz(self) -> u16 {
        match self {
            PollingRate::Hz125 => 125,
            PollingRate::Hz250 => 250,
            PollingRate::Hz500 => 500,
            PollingRate::Hz1000 => 1000,
            PollingRate::Hz2000 => 2000,
            PollingRate::Hz4000 => 4000,
        }
    }

    /// All available polling rates.
    pub fn all() -> &'static [PollingRate] {
        &[
            PollingRate::Hz125,
            PollingRate::Hz250,
            PollingRate::Hz500,
            PollingRate::Hz1000,
            PollingRate::Hz2000,
            PollingRate::Hz4000,
        ]
    }
}

impl std::fmt::Display for PollingRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Hz", self.hz())
    }
}

/// LED effect mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedMode {
    Off,
    Solid,
    Breathing,
    Neon,
}

impl LedMode {
    pub fn to_code(self) -> u8 {
        match self {
            LedMode::Off => 0x00,
            LedMode::Solid => 0x01,
            LedMode::Breathing => 0x02,
            LedMode::Neon => 0x03,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x00 => Some(LedMode::Off),
            0x01 => Some(LedMode::Solid),
            0x02 => Some(LedMode::Breathing),
            0x03 => Some(LedMode::Neon),
            _ => None,
        }
    }
}

impl std::fmt::Display for LedMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedMode::Off => write!(f, "Off"),
            LedMode::Solid => write!(f, "Solid"),
            LedMode::Breathing => write!(f, "Breathing"),
            LedMode::Neon => write!(f, "Neon / Rainbow Cycle"),
        }
    }
}

/// RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const RED: Self = Self::new(255, 0, 0);
    pub const CYAN: Self = Self::new(0, 255, 255);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0);
    #[allow(dead_code)]
    pub const MAGENTA: Self = Self::new(170, 255, 255);
    #[allow(dead_code)]
    pub const PURPLE: Self = Self::new(0, 255, 255);
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Lift-Off Distance setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiftOffDistance {
    /// 1mm lift-off distance
    Low,
    /// 2mm lift-off distance
    High,
}

impl LiftOffDistance {
    pub fn to_code(self) -> u8 {
        match self {
            LiftOffDistance::Low => 0x00,
            LiftOffDistance::High => 0x01,
        }
    }

    #[allow(dead_code)]
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x00 => Some(LiftOffDistance::Low),
            0x01 => Some(LiftOffDistance::High),
            _ => None,
        }
    }

    pub fn mm(self) -> &'static str {
        match self {
            LiftOffDistance::Low => "1mm",
            LiftOffDistance::High => "2mm",
        }
    }
}

impl std::fmt::Display for LiftOffDistance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mm())
    }
}

/// Single DPI stage configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiStage {
    /// DPI value (50–26000)
    pub dpi: u16,
    /// Whether this stage is enabled
    pub enabled: bool,
    /// LED color for this stage
    pub color: Color,
}

/// DPI LED configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiLed {
    pub mode: LedMode,
    /// 0–10
    pub brightness: u8,
    /// 1–10
    pub breath_speed: u8,
    pub color: Color,
}

/// Button action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonAction {
    /// Standard mouse button (button number)
    MouseButton(u8),
    /// Keyboard key (HID usage code, modifier)
    Keyboard { usage: u8, modifier: u8 },
    /// Media key
    Media(u8),
    /// DPI switch action
    DpiSwitch,
    /// Macro playback
    Macro(u8),
    /// Fire key / rapid fire (clicks count, delay in ms)
    FireKey { clicks: u8, delay_ms: u8 },
    /// Disabled
    Disabled,
}

impl ButtonAction {
    /// Encode to the 3-byte wire format.
    #[allow(dead_code)]
    pub fn to_bytes(self) -> [u8; 3] {
        match self {
            ButtonAction::MouseButton(btn) => [0x01, btn, 0x00],
            ButtonAction::Keyboard { usage, modifier } => [0x02, usage, modifier],
            ButtonAction::Media(code) => [0x03, code, 0x00],
            ButtonAction::DpiSwitch => [0x04, 0x00, 0x00],
            ButtonAction::Macro(id) => [0x05, id, 0x00],
            ButtonAction::FireKey { clicks, delay_ms } => [0x06, clicks, delay_ms],
            ButtonAction::Disabled => [0x00, 0x00, 0x00],
        }
    }

    /// Parse from the 3-byte wire format.
    #[allow(dead_code)]
    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        match bytes[0] {
            0x01 => ButtonAction::MouseButton(bytes[1]),
            0x02 => ButtonAction::Keyboard {
                usage: bytes[1],
                modifier: bytes[2],
            },
            0x03 => ButtonAction::Media(bytes[1]),
            0x04 => ButtonAction::DpiSwitch,
            0x05 => ButtonAction::Macro(bytes[1]),
            0x06 => ButtonAction::FireKey {
                clicks: bytes[1],
                delay_ms: bytes[2],
            },
            _ => ButtonAction::Disabled,
        }
    }
}

impl std::fmt::Display for ButtonAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ButtonAction::MouseButton(1) => write!(f, "Left Click"),
            ButtonAction::MouseButton(2) => write!(f, "Right Click"),
            ButtonAction::MouseButton(4) => write!(f, "Middle Click"),
            ButtonAction::MouseButton(8) => write!(f, "Back"),
            ButtonAction::MouseButton(16) => write!(f, "Forward"),
            ButtonAction::MouseButton(b) => write!(f, "Mouse Button {}", b),
            ButtonAction::Keyboard { usage, modifier } => {
                write!(f, "Key(0x{:02X}, mod=0x{:02X})", usage, modifier)
            }
            ButtonAction::Media(code) => write!(f, "Media(0x{:02X})", code),
            ButtonAction::DpiSwitch => write!(f, "DPI Cycle"),
            ButtonAction::Macro(id) => write!(f, "Macro #{}", id),
            ButtonAction::FireKey { clicks, delay_ms } => {
                write!(f, "Fire Key ({}x, {}ms)", clicks, delay_ms)
            }
            ButtonAction::Disabled => write!(f, "Disabled"),
        }
    }
}

/// Macro event type for the Macro Manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroEventType {
    KeyDown,
    KeyUp,
    Delay,
}

/// Single event step within a macro sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroEvent {
    pub event_type: MacroEventType,
    pub key: String,
    pub delay_ms: u32,
}

/// Macro repeat behavior mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroRepeatMode {
    /// Repeat N times (ReNumber)
    Times(u32),
    /// Repeat while button held down (ReUntilLoose)
    WhileHeld,
    /// Toggle on click until clicked again (ReUntilClick)
    Toggle,
}

/// Complete macro sequence item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroItem {
    pub name: String,
    pub repeat_mode: MacroRepeatMode,
    pub auto_insert_delay: bool,
    pub default_delay_ms: u32,
    pub events: Vec<MacroEvent>,
}

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

fn default_macros() -> Vec<MacroItem> {
    vec![
        MacroItem {
            name: "Rapid Fire (L-Click)".to_string(),
            repeat_mode: MacroRepeatMode::Times(3),
            auto_insert_delay: true,
            default_delay_ms: 12,
            events: vec![
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 12,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
            ],
        },
        MacroItem {
            name: "Sprint + Jump".to_string(),
            repeat_mode: MacroRepeatMode::Times(1),
            auto_insert_delay: true,
            default_delay_ms: 15,
            events: vec![
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Shift".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 20,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Space".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 20,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Space".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Shift".into(),
                    delay_ms: 0,
                },
            ],
        },
        MacroItem {
            name: "Sniper Quick Scope".to_string(),
            repeat_mode: MacroRepeatMode::Times(1),
            auto_insert_delay: true,
            default_delay_ms: 10,
            events: vec![
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Mouse Right".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 80,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 20,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Mouse Right".into(),
                    delay_ms: 0,
                },
            ],
        },
    ]
}

/// Complete mouse configuration state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseConfig {
    /// Active profile (1 to 5)
    #[serde(default = "default_active_profile")]
    pub active_profile: u8,
    /// Polling rate
    pub polling_rate: PollingRate,
    /// Active DPI stage index (0-based)
    pub active_dpi_stage: u8,
    /// DPI stages (up to 8)
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
    /// Stored macro library
    #[serde(default = "default_macros")]
    pub macros: Vec<MacroItem>,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            active_profile: 1,
            polling_rate: PollingRate::Hz1000,
            active_dpi_stage: 1,
            dpi_stages: vec![
                DpiStage {
                    dpi: 400,
                    enabled: true,
                    color: Color::RED,
                },
                DpiStage {
                    dpi: 800,
                    enabled: true,
                    color: Color::CYAN,
                },
                DpiStage {
                    dpi: 1600,
                    enabled: true,
                    color: Color::GREEN,
                },
                DpiStage {
                    dpi: 3200,
                    enabled: true,
                    color: Color::WHITE,
                },
                DpiStage {
                    dpi: 6400,
                    enabled: true,
                    color: Color::YELLOW,
                },
            ],
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
                ButtonAction::MouseButton(0x01), // Left
                ButtonAction::MouseButton(0x02), // Right
                ButtonAction::MouseButton(0x04), // Middle
                ButtonAction::MouseButton(0x08), // Back
                ButtonAction::MouseButton(0x10), // Forward
                ButtonAction::DpiSwitch,         // DPI button
            ],
            macros: default_macros(),
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
            ButtonAction::MouseButton(0x08),
            ButtonAction::MouseButton(0x10),
            ButtonAction::DpiSwitch,
        ];
        while self.buttons.len() < 6 {
            let idx = self.buttons.len();
            self.buttons.push(default_buttons[idx]);
        }
        self.buttons.truncate(6);

        // Ensure dpi_stages is not empty and has at most 8 stages
        if self.dpi_stages.is_empty() {
            *self = Self::default();
            return;
        }
        if self.dpi_stages.len() > 8 {
            self.dpi_stages.truncate(8);
        }

        // Validate each DPI stage value
        for stage in &mut self.dpi_stages {
            stage.dpi = crate::dpi::snap_dpi(stage.dpi);
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

        // Sleep timers (max 600s / 10 minutes)
        if self.sensor_sleep_seconds > 600 {
            self.sensor_sleep_seconds = 600;
        }
        if self.led_sleep_seconds > 600 {
            self.led_sleep_seconds = 600;
        }

        // Macros list: ensure at least 1 macro exists
        if self.macros.is_empty() {
            self.macros = default_macros();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polling_rate_roundtrip() {
        for &rate in PollingRate::all() {
            let code = rate.to_code();
            let parsed = PollingRate::from_code(code).unwrap();
            assert_eq!(rate, parsed);
        }
    }

    #[test]
    fn test_lod_roundtrip() {
        let low = LiftOffDistance::Low;
        let high = LiftOffDistance::High;
        assert_eq!(LiftOffDistance::from_code(low.to_code()), Some(low));
        assert_eq!(LiftOffDistance::from_code(high.to_code()), Some(high));
        assert_eq!(LiftOffDistance::from_code(0xFF), None);
    }

    #[test]
    fn test_button_action_roundtrip() {
        let left = ButtonAction::MouseButton(1);
        let wire = left.to_bytes();
        assert_eq!(ButtonAction::from_bytes(wire), left);

        let kb = ButtonAction::Keyboard {
            usage: 0x04,
            modifier: 0x01,
        };
        let wire_kb = kb.to_bytes();
        assert_eq!(ButtonAction::from_bytes(wire_kb), kb);

        let dpi = ButtonAction::DpiSwitch;
        let wire_dpi = dpi.to_bytes();
        assert_eq!(ButtonAction::from_bytes(wire_dpi), dpi);

        let fire = ButtonAction::FireKey {
            clicks: 3,
            delay_ms: 12,
        };
        let wire_fire = fire.to_bytes();
        assert_eq!(ButtonAction::from_bytes(wire_fire), fire);
    }

    #[test]
    fn test_led_mode_neon() {
        assert_eq!(LedMode::Neon.to_code(), 0x03);
        assert_eq!(LedMode::from_code(0x03), Some(LedMode::Neon));
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let default_cfg = MouseConfig::default();
        let toml_str = toml::to_string_pretty(&default_cfg).expect("Serialize TOML");
        let parsed_cfg: MouseConfig = toml::from_str(&toml_str).expect("Deserialize TOML");
        assert_eq!(default_cfg.polling_rate, parsed_cfg.polling_rate);
        assert_eq!(default_cfg.active_dpi_stage, parsed_cfg.active_dpi_stage);
        assert_eq!(default_cfg.debounce_ms, parsed_cfg.debounce_ms);
        assert_eq!(default_cfg.lod, parsed_cfg.lod);
        assert_eq!(default_cfg.dpi_stages.len(), parsed_cfg.dpi_stages.len());
    }

    #[test]
    fn test_parse_example_profile() {
        let example = include_str!("../examples/profile-default.toml");
        let parsed: MouseConfig = toml::from_str(example).expect("Failed to parse example profile");
        assert_eq!(parsed.polling_rate, PollingRate::Hz1000);
        assert_eq!(parsed.dpi_stages.len(), 5);
        assert_eq!(parsed.buttons.len(), 6);
    }

    #[test]
    fn test_config_validation() {
        let mut cfg = MouseConfig::default();
        cfg.active_profile = 99; // invalid
        cfg.debounce_ms = 55; // invalid
        cfg.buttons.clear(); // empty buttons
        cfg.dpi_stages[cfg.active_dpi_stage as usize].enabled = false; // active stage disabled
        cfg.dpi_led.brightness = 99; // invalid
        cfg.dpi_led.breath_speed = 0; // invalid

        cfg.validate();

        assert_eq!(cfg.active_profile, 1);
        assert_eq!(cfg.debounce_ms, 20);
        assert_eq!(cfg.buttons.len(), 6);
        assert!(cfg.dpi_stages[cfg.active_dpi_stage as usize].enabled);
        assert_eq!(cfg.dpi_led.brightness, 10);
        assert_eq!(cfg.dpi_led.breath_speed, 1);
    }
}
