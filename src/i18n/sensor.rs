//! Tab 2: Sensor, DPI stages, LOD, and debounce translations.

use super::AppLanguage;

impl AppLanguage {
    // ── Tab 2: Sensor & DPI ──────────────────────────────────────────
    pub fn dpi_stages_title(self) -> &'static str {
        match self {
            AppLanguage::English => "DPI Sensitivity Stages",
            AppLanguage::Indonesian => "Tingkat Sensitivitas DPI",
        }
    }

    pub fn separate_xy_toggle(self) -> &'static str {
        match self {
            AppLanguage::English => "Independent X / Y Axis Sensitivity (Separate X/Y DPI)",
            AppLanguage::Indonesian => "Sensitivitas Sumbu X / Y Terpisah (DPI X/Y Independen)",
        }
    }

    pub fn separate_xy_desc(self) -> &'static str {
        match self {
            AppLanguage::English => "Decouple horizontal (X) and vertical (Y) sensor sensitivity",
            AppLanguage::Indonesian => {
                "Pisahkan sensitivitas sensor horizontal (X) dan vertikal (Y)"
            }
        }
    }

    pub fn single_dpi_lock_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Single DPI Lock (Esports Mode)",
            AppLanguage::Indonesian => "Kunci Satu DPI (Mode Esports)",
        }
    }

    pub fn single_dpi_lock_desc(self) -> &'static str {
        match self {
            AppLanguage::English => {
                "Lock to a single active DPI stage to avoid accidental stage switching"
            }
            AppLanguage::Indonesian => {
                "Kunci ke satu tingkat DPI aktif guna mencegah perpindahan tidak sengaja"
            }
        }
    }

    pub fn sensor_tuning_title(self) -> &'static str {
        match self {
            AppLanguage::English => "Sensor & Performance Tuning",
            AppLanguage::Indonesian => "Penyetelan Sensor & Performa",
        }
    }

    pub fn polling_rate_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Polling Rate (Report Frequency)",
            AppLanguage::Indonesian => "Frekuensi Polling (Polling Rate)",
        }
    }

    pub fn lod_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Lift-Off Distance (LOD)",
            AppLanguage::Indonesian => "Jarak Angkat Sensor (LOD)",
        }
    }

    pub fn angle_snapping_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Angle Snapping (Line Straightening)",
            AppLanguage::Indonesian => "Pelurus Garis (Angle Snapping)",
        }
    }

    pub fn motion_sync_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Motion Sync (USB Interval Alignment)",
            AppLanguage::Indonesian => "Sinkronisasi Gerak (Motion Sync)",
        }
    }

    pub fn ripple_control_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Ripple Control (High DPI Smoothing)",
            AppLanguage::Indonesian => "Peredam Jitter (Ripple Control)",
        }
    }

    pub fn performance_mode_label(self) -> &'static str {
        match self {
            AppLanguage::English => "Performance Mode (Low Latency Gaming)",
            AppLanguage::Indonesian => "Mode Performa (Gaming Responsif)",
        }
    }
}
