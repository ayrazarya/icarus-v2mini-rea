//! DPI sensitivity stage configuration and default presets.

use super::lighting::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiStage {
    /// DPI value (50–26000), represents X-axis DPI or unified DPI
    pub dpi: u16,
    /// Optional independent Y-axis DPI value (50–26000)
    #[serde(default)]
    pub dpi_y: Option<u16>,
    /// Whether this stage is enabled
    pub enabled: bool,
    /// LED color for this stage
    pub color: Color,
}

impl DpiStage {
    /// Get effective X-axis DPI
    #[must_use]
    pub const fn effective_x(&self) -> u16 {
        self.dpi
    }

    /// Get effective Y-axis DPI (falls back to X-axis DPI if None)
    #[must_use]
    pub const fn effective_y(&self) -> u16 {
        match self.dpi_y {
            Some(y) => y,
            None => self.dpi,
        }
    }
}

/// Returns the standard 5 default DPI stages (400, 800, 1600, 3200, 6400).
#[must_use]
pub fn default_dpi_stages() -> Vec<DpiStage> {
    vec![
        DpiStage {
            dpi: 400,
            dpi_y: None,
            enabled: true,
            color: Color::RED,
        },
        DpiStage {
            dpi: 800,
            dpi_y: None,
            enabled: true,
            color: Color::CYAN,
        },
        DpiStage {
            dpi: 1600,
            dpi_y: None,
            enabled: true,
            color: Color::GREEN,
        },
        DpiStage {
            dpi: 3200,
            dpi_y: None,
            enabled: true,
            color: Color::WHITE,
        },
        DpiStage {
            dpi: 6400,
            dpi_y: None,
            enabled: true,
            color: Color::YELLOW,
        },
    ]
}
