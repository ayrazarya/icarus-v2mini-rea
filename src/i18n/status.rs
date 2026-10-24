//! Status bar, alert banners, notifications, and telemetry translations.

use super::AppLanguage;

impl AppLanguage {
    // ── Status & Notification Strings ─────────────────────────────────
    pub fn button_options(self) -> &'static [&'static str] {
        match self {
            AppLanguage::English => &[
                "Left Click",
                "Right Click",
                "Middle Click",
                "Forward",
                "Back",
                "DPI Switch / Cycle",
                "Fire Key (Rapid Fire)",
                "Media Function...",
                "Keyboard Key / Shortcut...",
                "Macro Playback...",
                "Disabled",
            ],
            AppLanguage::Indonesian => &[
                "Klik Kiri",
                "Klik Kanan",
                "Klik Tengah",
                "Maju (Forward)",
                "Mundur (Back)",
                "Ganti / Siklus DPI",
                "Klik Cepat (Rapid Fire)",
                "Fungsi Media...",
                "Tombol Keyboard / Pintasan...",
                "Putar Makro...",
                "Nonaktif",
            ],
        }
    }

    pub fn status_mouse_offline_hint(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "Mouse is offline or asleep. Move mouse or toggle bottom switch first."
            }
            AppLanguage::Indonesian => {
                "Mouse sedang offline/sleep. Gerakkan mouse atau nyalakan sakelar terlebih dahulu."
            }
        }
    }

    pub fn status_button_bound(self, idx: usize, action: &str) -> String {
        match self {
            AppLanguage::English => format!("Button {} set to: {}", idx, action),
            AppLanguage::Indonesian => format!("Tombol {} diatur ke: {}", idx, action),
        }
    }

    pub fn status_left_click_lock_warn(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "⚠️ Safety Lock: Left Click cannot be removed! Assign Left Click to another button first."
            }
            AppLanguage::Indonesian => {
                "⚠️ Safety Lock: Tombol Left Click tidak boleh dihilangkan! Pasang Left Click pada tombol lain terlebih dahulu."
            }
        }
    }

    pub fn status_listen_hint(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "Press any key on your keyboard to bind this button (or Esc to cancel)"
            }
            AppLanguage::Indonesian => {
                "Tekan sembarang tombol pada keyboard Anda untuk mengikat tombol ini (atau Esc untuk batal)"
            }
        }
    }

    pub fn status_recording_live(self) -> &'static str {
        match self {
            AppLanguage::English => "[REC] Recording live actions... Press keys or mouse buttons!",
            AppLanguage::Indonesian => {
                "[REC] Merekam aksi langsung... Tekan tombol keyboard atau tombol mouse!"
            }
        }
    }

    pub fn status_macro_buffer_ok(self, name: &str, bytes: usize) -> String {
        match self {
            AppLanguage::English => {
                format!(
                    "Macro '{}' converted to hardware buffer ({} bytes)",
                    name, bytes
                )
            }
            AppLanguage::Indonesian => {
                format!(
                    "Macro '{}' berhasil dikonversi ke hardware buffer ({} bytes)",
                    name, bytes
                )
            }
        }
    }

    pub fn status_macro_flashed_ok(self, name: &str, slot: u8) -> String {
        match self {
            AppLanguage::English => {
                format!(
                    "Macro '{}' successfully flashed to On-board Slot #{}!",
                    name, slot
                )
            }
            AppLanguage::Indonesian => {
                format!(
                    "Macro '{}' berhasil di-flash ke On-board Slot #{}!",
                    name, slot
                )
            }
        }
    }

    pub fn status_macro_flash_err(self, err: &str) -> String {
        match self {
            AppLanguage::English => format!("Failed to flash macro to hardware: {}", err),
            AppLanguage::Indonesian => format!("Gagal mem-flash macro ke hardware: {}", err),
        }
    }

    pub fn status_macro_all_flashed_ok(self) -> &'static str {
        match self {
            AppLanguage::English => "Successfully flashed all assigned macros to mouse EEPROM!",
            AppLanguage::Indonesian => "Berhasil mem-flash semua macro tombol ke mouse!",
        }
    }

    pub fn status_macro_all_flash_err(self, err: &str) -> String {
        match self {
            AppLanguage::English => format!("Failed to flash all macros: {}", err),
            AppLanguage::Indonesian => format!("Gagal mem-flash macro: {}", err),
        }
    }

    pub fn status_mouse_not_connected(self) -> &'static str {
        match self {
            AppLanguage::English => "Mouse device is not connected!",
            AppLanguage::Indonesian => "Perangkat mouse tidak terhubung!",
        }
    }

    pub fn status_pairing_err(self, err: &str) -> String {
        match self {
            AppLanguage::English => format!("Failed to send pairing command: {}", err),
            AppLanguage::Indonesian => format!("Gagal mengirim perintah pairing: {}", err),
        }
    }

    pub fn status_select_firmware_first(self) -> &'static str {
        match self {
            AppLanguage::English => "Please select a firmware file (.bin) first!",
            AppLanguage::Indonesian => "Pilih file firmware (.bin) terlebih dahulu!",
        }
    }

    pub fn status_firmware_flashed_ok(self, msg: &str) -> String {
        match self {
            AppLanguage::English => format!("Firmware successfully flashed! {}", msg),
            AppLanguage::Indonesian => format!("Firmware berhasil di-flash! {}", msg),
        }
    }

    pub fn status_firmware_flash_err(self, err: &str) -> String {
        match self {
            AppLanguage::English => format!("Firmware flashing failed: {}", err),
            AppLanguage::Indonesian => format!("Flashing firmware gagal: {}", err),
        }
    }

    pub fn status_firmware_need_usb(self) -> &'static str {
        match self {
            AppLanguage::English => "Device not connected! Please connect USB cable.",
            AppLanguage::Indonesian => "Perangkat tidak terhubung! Hubungkan kabel USB.",
        }
    }

    pub fn battery_critical_tag(self, bat: u8) -> String {
        match self {
            AppLanguage::English => format!("LOW {}% (Critical!)", bat),
            AppLanguage::Indonesian => format!("LOW {}% (Sekarat!)", bat),
        }
    }

    pub fn battery_low_tag(self, bat: u8) -> String {
        match self {
            AppLanguage::English => format!("LOW {}% (Low)", bat),
            AppLanguage::Indonesian => format!("LOW {}% (Lemah)", bat),
        }
    }

    pub fn voltage_label(self, v: f32, mv: u16) -> String {
        match self {
            AppLanguage::English => format!("Battery Voltage: {:.2} V ({} mV)", v, mv),
            AppLanguage::Indonesian => format!("Voltase Baterai: {:.2} V ({} mV)", v, mv),
        }
    }

    pub fn offline_banner_text(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "Ensure switch under mouse is in ON (2.4G) position, move mouse to wake up, or plug in cable if battery is empty."
            }
            AppLanguage::Indonesian => {
                "Pastikan saklar di bawah mouse posisi ON (2.4G), gerakkan mouse untuk wake up, atau colok kabel jika baterai habis."
            }
        }
    }
}
