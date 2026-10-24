//! Tab 3: RGB lighting, LED effects, and dongle bar translations.

use super::AppLanguage;

impl AppLanguage {
    // ── Tab 3: Lighting ──────────────────────────────────────────────
    pub fn mouse_lighting_title(self) -> &'static str {
        match self {
            AppLanguage::English => "DPI LED Lighting Effects",
            AppLanguage::Indonesian => "Efek Pencahayaan LED DPI",
        }
    }

    pub fn dongle_lighting_title(self) -> &'static str {
        match self {
            AppLanguage::English => "4K Wireless Receiver Dongle RGB",
            AppLanguage::Indonesian => "RGB Receiver Dongle Nirkabel 4K",
        }
    }

    pub fn led_mode_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Lighting Mode",
            AppLanguage::Indonesian => "Mode Pencahayaan",
        }
    }

    pub fn brightness_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Brightness Level",
            AppLanguage::Indonesian => "Tingkat Kecerahan",
        }
    }

    pub fn speed_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Breathing / Animation Speed",
            AppLanguage::Indonesian => "Kecepatan Denyut / Animasi",
        }
    }

    pub fn sleep_timer_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Idle Sleep Timeout",
            AppLanguage::Indonesian => "Waktu Siaga Tidur (Sleep Timer)",
        }
    }

    pub fn auto_apply_toggle(self) -> &'static str {
        match self {
            AppLanguage::English => "Live Auto-Apply (Instant Hardware Sync)",
            AppLanguage::Indonesian => "Otomatis Terapkan Langsung (Sinkronisasi Instan)",
        }
    }

    pub fn bar_mode_label(self) -> &'static str {
        match self {
            AppLanguage::English => "LED Indicator Bar Mode",
            AppLanguage::Indonesian => "Mode Bilah Indikator LED",
        }
    }

    pub fn bar_mode_sync(self) -> &'static str {
        match self {
            AppLanguage::English => "Synchronize with RGB Lighting",
            AppLanguage::Indonesian => "Sinkron dengan Lampu RGB",
        }
    }

    pub fn bar_mode_battery(self) -> &'static str {
        match self {
            AppLanguage::English => "Battery Status Indicator",
            AppLanguage::Indonesian => "Indikator Status Baterai",
        }
    }

    pub fn bar_mode_fixed(self) -> &'static str {
        match self {
            AppLanguage::English => "Fixed Solid Color",
            AppLanguage::Indonesian => "Warna Tetap (Solid)",
        }
    }
}
