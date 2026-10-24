//! Tab 5: Application profiles, rules, pairing, and firmware update translations.

use super::AppLanguage;

impl AppLanguage {
    // ── Tab 5: Settings ──────────────────────────────────────────────
    pub fn settings_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Device Settings & Profiles",
            AppLanguage::Indonesian => "Pengaturan Perangkat & Profil",
        }
    }

    pub fn language_setting_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Interface Language",
            AppLanguage::Indonesian => "Bahasa Tampilan",
        }
    }

    pub fn language_setting_desc(self) -> &'static str {
        match self {
            AppLanguage::English => "Choose the display language for all interface elements",
            AppLanguage::Indonesian => "Pilih bahasa tampilan untuk seluruh antarmuka aplikasi",
        }
    }

    pub fn profile_storage_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Profile Import & Export",
            AppLanguage::Indonesian => "Impor & Ekspor Profil",
        }
    }

    pub fn export_profile_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "💾 Export Profile (TOML)",
            AppLanguage::Indonesian => "💾 Ekspor Profil (TOML)",
        }
    }

    pub fn import_profile_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "📂 Import Profile (TOML)",
            AppLanguage::Indonesian => "📂 Impor Profil (TOML)",
        }
    }

    pub fn reset_defaults_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "🔄 Reset to Factory Defaults",
            AppLanguage::Indonesian => "🔄 Kembalikan ke Standar Pabrik",
        }
    }

    pub fn dongle_pairing_title(self) -> &'static str {
        match self {
            AppLanguage::English => "2.4G Wireless Dongle Pairing",
            AppLanguage::Indonesian => "Pemasangan Dongle Nirkabel 2.4G",
        }
    }

    pub fn start_pairing_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "📡 Start Pairing",
            AppLanguage::Indonesian => "📡 Mulai Pemasangan",
        }
    }

    pub fn pairing_hint(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "Bring mouse close to the dongle and press Left + Middle + Right buttons for 3 seconds."
            }
            AppLanguage::Indonesian => {
                "Dekatkan mouse ke dongle dan tahan tombol Kiri + Tengah + Kanan selama 3 detik."
            }
        }
    }

    pub fn firmware_flasher_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Firmware Update (ISP Flasher)",
            AppLanguage::Indonesian => "Pembaruan Firmware (Flasher ISP)",
        }
    }

    pub fn target_mcu_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Target Microcontroller (MCU):",
            AppLanguage::Indonesian => "Mikrokontroler Target (MCU):",
        }
    }

    pub fn target_mouse_mcu(self) -> &'static str {
        match self {
            AppLanguage::English => "Mouse MCU (Nordic nRF52840)",
            AppLanguage::Indonesian => "MCU Mouse (Nordic nRF52840)",
        }
    }

    pub fn target_dongle_mcu(self) -> &'static str {
        match self {
            AppLanguage::English => "4K Receiver Dongle MCU",
            AppLanguage::Indonesian => "MCU Dongle Receiver 4K",
        }
    }

    pub fn browse_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "Browse...",
            AppLanguage::Indonesian => "Pilih File...",
        }
    }

    pub fn flash_firmware_now_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "⚡ Flash Firmware Now",
            AppLanguage::Indonesian => "⚡ Flash Firmware Sekarang",
        }
    }

    pub fn firmware_warning(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "CAUTION: Do NOT disconnect USB cable or power off computer during firmware flashing."
            }
            AppLanguage::Indonesian => {
                "PERINGATAN: JANGAN cabut kabel USB atau matikan komputer selama proses flashing firmware."
            }
        }
    }
}
