//! Sensor hardware settings: Polling rate and Lift-off distance.

use serde::{Deserialize, Serialize};

/// Polling rate setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PollingRate {
    Hz125,
    Hz250,
    Hz500,
    #[default]
    Hz1000,
    Hz2000,
    Hz4000,
}

impl PollingRate {
    /// Convert to the wire protocol byte value.
    #[must_use]
    pub const fn to_code(self) -> u8 {
        match self {
            PollingRate::Hz125 => 0x08,
            PollingRate::Hz250 => 0x04,
            PollingRate::Hz500 => 0x02,
            PollingRate::Hz1000 => 0x01,
            PollingRate::Hz2000 => 0x10,
            PollingRate::Hz4000 => 0x20,
        }
    }

    /// Parse from the wire protocol byte value.
    #[must_use]
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0x08 => Some(PollingRate::Hz125),
            0x04 => Some(PollingRate::Hz250),
            0x02 => Some(PollingRate::Hz500),
            0x01 => Some(PollingRate::Hz1000),
            0x10 => Some(PollingRate::Hz2000),
            0x20 => Some(PollingRate::Hz4000),
            _ => None,
        }
    }

    /// Get the rate in Hz.
    #[must_use]
    pub const fn hz(self) -> u16 {
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
    #[must_use]
    pub const fn all() -> &'static [PollingRate] {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LiftOffDistance {
    /// 1mm lift-off distance
    #[default]
    Low,
    /// 2mm lift-off distance
    High,
}

impl LiftOffDistance {
    /// Returns the raw hardware register code for the lift-off distance setting.
    #[must_use]
    pub const fn to_code(self) -> u8 {
        match self {
            LiftOffDistance::Low => 0x01,
            LiftOffDistance::High => 0x02,
        }
    }

    /// Converts a raw register code byte into the corresponding `LiftOffDistance`.
    #[must_use]
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0x00 | 0x01 => Some(LiftOffDistance::Low),
            0x02 => Some(LiftOffDistance::High),
            _ => None,
        }
    }

    /// Returns the human-readable physical lift-off height in millimeters.
    #[must_use]
    pub const fn mm(self) -> &'static str {
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
