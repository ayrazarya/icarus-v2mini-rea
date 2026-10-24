//! Mouse configuration types for the ICARUS V2 MINI.
//!
//! These types represent the parsed configuration state that maps
//! to the FlashDataMap structure in the mouse firmware.

pub mod buttons;
pub mod dpi;
pub mod lighting;
pub mod macros;
pub mod mouse;
pub mod rules;
pub mod sensor;

pub use crate::i18n::AppLanguage;
pub use buttons::ButtonAction;
pub use dpi::{DpiStage, default_dpi_stages};
pub use lighting::{Color, DongleRgbConfig, DpiLed, LedMode};
pub use macros::{MacroEvent, MacroEventType, MacroItem, MacroRepeatMode, default_macros};
pub use mouse::MouseConfig;
pub use rules::{AutoProfileRule, default_auto_rules};
pub use sensor::{LiftOffDistance, PollingRate};
