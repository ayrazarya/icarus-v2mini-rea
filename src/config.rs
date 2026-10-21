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
}

impl PollingRate {
    /// Convert to the wire protocol byte value.
    pub fn to_code(self) -> u8 {
        match self {
            PollingRate::Hz125 => 0x08,
            PollingRate::Hz250 => 0x04,
            PollingRate::Hz500 => 0x02,
            PollingRate::Hz1000 => 0x01,
        }
    }

    /// Parse from the wire protocol byte value.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x08 => Some(PollingRate::Hz125),
            0x04 => Some(PollingRate::Hz250),
            0x02 => Some(PollingRate::Hz500),
            0x01 => Some(PollingRate::Hz1000),
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
        }
    }

    /// All available polling rates.
    pub fn all() -> &'static [PollingRate] {
        &[
            PollingRate::Hz125,
            PollingRate::Hz250,
            PollingRate::Hz500,
            PollingRate::Hz1000,
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
}

impl LedMode {
    pub fn to_code(self) -> u8 {
        match self {
            LedMode::Off => 0x00,
            LedMode::Solid => 0x01,
            LedMode::Breathing => 0x02,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x00 => Some(LedMode::Off),
            0x01 => Some(LedMode::Solid),
            0x02 => Some(LedMode::Breathing),
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
            ButtonAction::Disabled => write!(f, "Disabled"),
        }
    }
}

/// Complete mouse configuration state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseConfig {
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
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
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
}
