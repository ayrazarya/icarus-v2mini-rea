//! Internationalization (i18n) module for ICARUS V2 MINI.
//!
//! Provides bilingual translations (English and Bahasa Indonesia) for all
//! UI components, dialogs, status banners, telemetry, tooltips, and settings,
//! preventing mixed or overlapping language strings.

mod buttons;
mod lighting;
mod macros;
mod navigation;
mod sensor;
mod settings;
mod status;

use serde::{Deserialize, Serialize};

/// Application UI display language setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppLanguage {
    #[default]
    English,
    Indonesian,
}

impl AppLanguage {
    /// Return the language display name in its native script.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            AppLanguage::English => "English (US)",
            AppLanguage::Indonesian => "Bahasa Indonesia",
        }
    }

    /// Return short 2-letter uppercase ISO code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            AppLanguage::English => "EN",
            AppLanguage::Indonesian => "ID",
        }
    }

    /// All available languages.
    #[must_use]
    pub const fn all() -> &'static [AppLanguage] {
        &[AppLanguage::English, AppLanguage::Indonesian]
    }
}
