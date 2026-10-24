//! Tab 1: Button remapping and action selector translations.

use super::AppLanguage;

impl AppLanguage {
    // ── Tab 1: Buttons ───────────────────────────────────────────────
    pub fn button_definitions(self) -> [(&'static str, &'static str, usize); 6] {
        match self {
            AppLanguage::English => [
                ("Button 1: Left Click", "Left Click Switch", 0),
                ("Button 2: Right Click", "Right Click Switch", 1),
                ("Button 3: Middle Click (Wheel)", "Scroll Wheel Click", 2),
                ("Button 4: Forward", "Front Side Button", 3),
                ("Button 5: Back", "Rear Side Button", 4),
                ("Button 6: DPI Cycle Button", "Underside Base Button", 5),
            ],
            AppLanguage::Indonesian => [
                ("Tombol 1: Klik Kiri", "Sakelar Klik Kiri", 0),
                ("Tombol 2: Klik Kanan", "Sakelar Klik Kanan", 1),
                ("Tombol 3: Klik Tengah (Roda)", "Klik Roda Scroll", 2),
                ("Tombol 4: Maju (Forward)", "Tombol Samping Depan", 3),
                ("Tombol 5: Mundur (Back)", "Tombol Samping Belakang", 4),
                ("Tombol 6: Tombol DPI", "Tombol Bawah Mouse", 5),
            ],
        }
    }

    pub fn button_remapping_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Button Remapping & Assignment",
            AppLanguage::Indonesian => "Pemetaan & Konfigurasi Tombol",
        }
    }

    pub fn button_remapping_subtitle(self) -> &'static str {
        match self {
            AppLanguage::English => "Select an action for each of the 6 physical mouse buttons",
            AppLanguage::Indonesian => "Pilih fungsi untuk setiap dari 6 tombol fisik mouse",
        }
    }

    pub fn debounce_tester_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Mouse Click & Debounce Tester",
            AppLanguage::Indonesian => "Penguji Klik & Debounce Mouse",
        }
    }

    pub fn debounce_tester_subtitle(self) -> &'static str {
        match self {
            AppLanguage::English => "Test switch response, measure CPS, and detect double-clicking",
            AppLanguage::Indonesian => {
                "Uji respons sakelar, hitung CPS, dan deteksi klik ganda (double-click)"
            }
        }
    }

    pub fn btn_left(self) -> &'static str {
        match self {
            AppLanguage::English => "1. Left Click",
            AppLanguage::Indonesian => "1. Klik Kiri",
        }
    }

    pub fn btn_right(self) -> &'static str {
        match self {
            AppLanguage::English => "2. Right Click",
            AppLanguage::Indonesian => "2. Klik Kanan",
        }
    }

    pub fn btn_middle(self) -> &'static str {
        match self {
            AppLanguage::English => "3. Middle Click",
            AppLanguage::Indonesian => "3. Klik Tengah",
        }
    }

    pub fn btn_forward(self) -> &'static str {
        match self {
            AppLanguage::English => "4. Forward (Side)",
            AppLanguage::Indonesian => "4. Maju (Samping)",
        }
    }

    pub fn btn_back(self) -> &'static str {
        match self {
            AppLanguage::English => "5. Back (Side)",
            AppLanguage::Indonesian => "5. Mundur (Samping)",
        }
    }

    pub fn btn_dpi(self) -> &'static str {
        match self {
            AppLanguage::English => "6. DPI Switch (Base)",
            AppLanguage::Indonesian => "6. Tombol DPI (Bawah)",
        }
    }

    pub fn capture_key_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "⌨ Capture Key",
            AppLanguage::Indonesian => "⌨ Rekam Tombol",
        }
    }

    pub fn listening_key_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "🔴 Press any key...",
            AppLanguage::Indonesian => "🔴 Tekan tombol keyboard...",
        }
    }

    pub fn left_click_safety_lock(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "Left Click Safety Lock: At least one button must always be assigned to Left Click."
            }
            AppLanguage::Indonesian => {
                "Kunci Pengaman Klik Kiri: Minimal satu tombol harus selalu berfungsi sebagai Klik Kiri."
            }
        }
    }

    pub fn debounce_slider_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Hardware Debounce Time",
            AppLanguage::Indonesian => "Waktu Debounce Hardware",
        }
    }

    pub fn click_test_pad_placeholder(self) -> &'static str {
        match self {
            AppLanguage::English => "CLICK OR TEST HERE",
            AppLanguage::Indonesian => "KLIK ATAU UJI DI SINI",
        }
    }

    pub fn click_test_pad_hint(self) -> &'static str {
        match self {
            AppLanguage::English => "Click with any mouse button inside this pad to test switches",
            AppLanguage::Indonesian => {
                "Klik tombol mouse apa saja di area ini untuk menguji sakelar"
            }
        }
    }

    pub fn clear_counter_btn(self) -> &'static str {
        match self {
            AppLanguage::English => "Clear Counter",
            AppLanguage::Indonesian => "Reset Hitungan",
        }
    }
}
