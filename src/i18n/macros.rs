//! Tab 4: Hardware macro editor and keystroke recorder translations.

use super::AppLanguage;

impl AppLanguage {
    // ── Tab 4: Macro ─────────────────────────────────────────────────
    pub fn macro_manager_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Hardware Macro Sequencer",
            AppLanguage::Indonesian => "Manajer & Urutan Makro Hardware",
        }
    }

    pub fn macro_list_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Macro Sequence List",
            AppLanguage::Indonesian => "Daftar Urutan Makro",
        }
    }

    pub fn record_macro_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "🔴 Start Recording",
            AppLanguage::Indonesian => "🔴 Mulai Merekam",
        }
    }

    pub fn stop_record_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "⏹ Stop Recording",
            AppLanguage::Indonesian => "⏹ Hentikan Rekaman",
        }
    }

    pub fn flash_macro_slot_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "⚡ Flash to Mouse EEPROM",
            AppLanguage::Indonesian => "⚡ Flash ke EEPROM Mouse",
        }
    }

    pub fn flash_all_macros_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "⚡ Flash All Assigned Macros to Mouse",
            AppLanguage::Indonesian => "⚡ Flash Semua Makro Terpasang ke Mouse",
        }
    }

    pub fn repeat_mode_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Repeat Execution Mode",
            AppLanguage::Indonesian => "Mode Pengulangan Eksekusi",
        }
    }

    pub fn repeat_times(self) -> &'static str {
        match self {
            AppLanguage::English => "Repeat N times",
            AppLanguage::Indonesian => "Ulangi N kali",
        }
    }

    pub fn repeat_while_held(self) -> &'static str {
        match self {
            AppLanguage::English => "Repeat while button held",
            AppLanguage::Indonesian => "Ulangi selama tombol ditahan",
        }
    }

    pub fn repeat_toggle(self) -> &'static str {
        match self {
            AppLanguage::English => "Toggle on/off on click",
            AppLanguage::Indonesian => "Klik bergantian (Toggle nyala/mati)",
        }
    }
}
