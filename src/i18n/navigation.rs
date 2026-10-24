//! Navigation, header, connection status, and footer translations.

use super::AppLanguage;

impl AppLanguage {
    // ── Header & Navigation ──────────────────────────────────────────
    pub fn title(self) -> &'static str {
        "ICARUS V2 MINI"
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            AppLanguage::English => "PAW3395 LINUX SOFTWARE",
            AppLanguage::Indonesian => "PERANGKAT LUNAK LINUX PAW3395",
        }
    }

    pub fn profile(self) -> &'static str {
        match self {
            AppLanguage::English => "Profile:",
            AppLanguage::Indonesian => "Profil:",
        }
    }

    pub fn rename(self) -> &'static str {
        match self {
            AppLanguage::English => "Rename",
            AppLanguage::Indonesian => "Ubah Nama",
        }
    }

    pub fn save(self) -> &'static str {
        match self {
            AppLanguage::English => "Save",
            AppLanguage::Indonesian => "Simpan",
        }
    }

    pub fn cancel(self) -> &'static str {
        match self {
            AppLanguage::English => "Cancel",
            AppLanguage::Indonesian => "Batal",
        }
    }

    pub fn scan_devices(self) -> &'static str {
        match self {
            AppLanguage::English => "Scan Devices",
            AppLanguage::Indonesian => "Pindai Perangkat",
        }
    }

    pub fn tab_buttons(self) -> &'static str {
        match self {
            AppLanguage::English => "Buttons & Remapping",
            AppLanguage::Indonesian => "Tombol & Pemetaan",
        }
    }

    pub fn tab_sensor(self) -> &'static str {
        match self {
            AppLanguage::English => "DPI & Sensor",
            AppLanguage::Indonesian => "DPI & Sensor",
        }
    }

    pub fn tab_lighting(self) -> &'static str {
        match self {
            AppLanguage::English => "Lighting & RGB",
            AppLanguage::Indonesian => "Pencahayaan & RGB",
        }
    }

    pub fn tab_macro(self) -> &'static str {
        match self {
            AppLanguage::English => "Macro Manager",
            AppLanguage::Indonesian => "Manajer Makro",
        }
    }

    pub fn tab_settings(self) -> &'static str {
        match self {
            AppLanguage::English => "Settings & Pairing",
            AppLanguage::Indonesian => "Pengaturan & Pairing",
        }
    }

    // ── Connection & Battery ─────────────────────────────────────────
    pub fn disconnected(self) -> &'static str {
        match self {
            AppLanguage::English => "Disconnected (Offline Mode)",
            AppLanguage::Indonesian => "Terputus (Mode Offline)",
        }
    }

    pub fn connected_usb(self) -> &'static str {
        match self {
            AppLanguage::English => "Connected via USB (Wired)",
            AppLanguage::Indonesian => "Terhubung via USB (Kabel)",
        }
    }

    pub fn connected_24g(self) -> &'static str {
        match self {
            AppLanguage::English => "Connected via 2.4G Wireless",
            AppLanguage::Indonesian => "Terhubung via 2.4G Nirkabel",
        }
    }

    pub fn connected_4k(self) -> &'static str {
        match self {
            AppLanguage::English => "Connected via 4K Wireless",
            AppLanguage::Indonesian => "Terhubung via 4K Nirkabel",
        }
    }

    pub fn battery_usb_charging(self) -> &'static str {
        match self {
            AppLanguage::English => "Battery Full • USB Mode (Charging)",
            AppLanguage::Indonesian => "Baterai Terisi • Mode USB (Dicas)",
        }
    }

    pub fn battery_usb_charging_desc(self) -> &'static str {
        match self {
            AppLanguage::English => "Mouse is connected via USB cable and charging battery.",
            AppLanguage::Indonesian => "Mouse sedang terhubung kabel USB dan mengisi daya baterai.",
        }
    }

    pub fn battery_critical_title(self) -> &'static str {
        match self {
            AppLanguage::English => "WARNING: Battery Critical!",
            AppLanguage::Indonesian => "PERINGATAN: Baterai Sekarat!",
        }
    }

    pub fn battery_critical_desc(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "Battery level is critically low (≤ 10%). Connect USB cable immediately."
            }
            AppLanguage::Indonesian => {
                "Daya baterai hampir habis (≤ 10%). Segera hubungkan kabel USB agar mouse tidak mati mendadak."
            }
        }
    }

    pub fn battery_low_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Battery Low Warning",
            AppLanguage::Indonesian => "Peringatan Baterai Lemah",
        }
    }

    pub fn battery_low_desc(self) -> &'static str {
        match self {
            AppLanguage::English => "Battery is below 20%. Please consider recharging soon.",
            AppLanguage::Indonesian => {
                "Daya baterai di bawah 20%. Disarankan untuk segera mengisi daya."
            }
        }
    }

    pub fn battery_normal_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Battery Status: Healthy",
            AppLanguage::Indonesian => "Status Baterai: Normal",
        }
    }

    pub fn battery_normal_desc(self) -> &'static str {
        match self {
            AppLanguage::English => "Battery power is optimal for gaming sessions.",
            AppLanguage::Indonesian => "Daya baterai optimal untuk sesi penggunaan.",
        }
    }

    pub fn dongle_ready_mouse_offline(self) -> &'static str {
        match self {
            AppLanguage::English => "Dongle Ready (Mouse Offline / Asleep)",
            AppLanguage::Indonesian => "Dongle Siap (Mouse Offline / Tertidur)",
        }
    }

    pub fn dongle_ready_hint(self) -> &'static str {
        match self {
            AppLanguage::English => "Click mouse button or switch power to 2.4G to wake up.",
            AppLanguage::Indonesian => {
                "Klik tombol mouse atau geser sakelar ke 2.4G untuk membangunkan."
            }
        }
    }

    pub fn dongle_offline_hover(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "2.4G Dongle detected, but mouse is not responding.\n\
                 Ensure the power switch on bottom is ON (2.4G), move mouse to wake it, or connect USB cable if battery is depleted."
            }
            AppLanguage::Indonesian => {
                "Dongle 2.4G terdeteksi, tetapi mouse tidak merespons.\n\
                 Pastikan saklar di bawah mouse posisi ON (2.4G), gerakkan mouse untuk wake up, atau colok kabel jika baterai habis."
            }
        }
    }

    // ── Footer ───────────────────────────────────────────────────────
    pub fn apply_changes_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "⚡ Apply Changes",
            AppLanguage::Indonesian => "⚡ Terapkan Perubahan",
        }
    }

    pub fn read_from_mouse_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "🔄 Read from Mouse",
            AppLanguage::Indonesian => "🔄 Baca dari Mouse",
        }
    }

    pub fn sync_status_saved(self) -> &'static str {
        match self {
            AppLanguage::English => "✓ Synchronized with mouse hardware",
            AppLanguage::Indonesian => "✓ Tersinkronisasi dengan hardware mouse",
        }
    }

    pub fn sync_status_unsaved(self) -> &'static str {
        match self {
            AppLanguage::English => "● Unsaved changes (Press Ctrl+S to apply)",
            AppLanguage::Indonesian => "● Ada perubahan belum diterapkan (Tekan Ctrl+S)",
        }
    }
}
