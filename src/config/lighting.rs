//! RGB lighting modes, color models, and receiver LED bar configuration.

use serde::{Deserialize, Serialize};

/// LED effect mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LedMode {
    Off,
    #[default]
    Solid,
    Breathing,
    Neon,
}

impl LedMode {
    /// Returns the raw hardware register code corresponding to this LED mode.
    #[must_use]
    pub const fn to_code(self) -> u8 {
        match self {
            LedMode::Off => 0x00,
            LedMode::Solid => 0x01,
            LedMode::Breathing => 0x02,
            LedMode::Neon => 0x03,
        }
    }

    /// Converts a raw hardware register byte into the corresponding `LedMode`.
    #[must_use]
    pub const fn from_code(code: u8) -> Option<Self> {
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

impl Default for Color {
    fn default() -> Self {
        Self::CYAN
    }
}

impl Color {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const RED: Self = Self::new(255, 0, 0);
    pub const CYAN: Self = Self::new(0, 255, 255);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0);
    pub const MAGENTA: Self = Self::new(255, 0, 255);
    pub const PURPLE: Self = Self::new(128, 0, 255);
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// DPI LED indicator configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DpiLed {
    pub mode: LedMode,
    /// 0–10
    pub brightness: u8,
    /// 1–10
    pub breath_speed: u8,
    pub color: Color,
}

/// 4K Wireless Dongle RGB receiver configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DongleRgbConfig {
    /// Whether the 4K dongle LED is enabled
    pub enabled: bool,
    /// LED lighting mode (Solid, Breathing, Neon, Off)
    pub mode: LedMode,
    /// Brightness level (0..=10)
    pub brightness: u8,
    /// Breathing / Neon animation speed (1..=10)
    pub speed: u8,
    /// Custom RGB color
    pub color: Color,
    /// LED Bar indicator mode (0: Sync with mouse, 1: Battery state, 2: Fixed color)
    pub bar_mode: u8,
}

impl Default for DongleRgbConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: LedMode::Solid,
            brightness: 10,
            speed: 5,
            color: Color::CYAN,
            bar_mode: 0,
        }
    }
}
