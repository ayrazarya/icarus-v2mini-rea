use std::time::Instant;

use eframe::egui;

use crate::config::{AppLanguage, MouseConfig};
use crate::device::IcarusDevice;

/// Active navigation tab in the desktop application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Buttons,
    Sensor,
    Lighting,
    Macro,
    Settings,
}

/// Pairing state for 2.4G wireless dongle tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PairingState {
    #[default]
    Idle,
    Pairing,
    Success,
    Failed,
}

/// The main application state.
pub struct IcarusApp {
    /// Active mouse configuration
    pub(crate) config: MouseConfig,
    /// Last saved or hardware-synced mouse configuration for unsaved changes detection
    pub(crate) synced_config: MouseConfig,
    /// Currently selected tab
    pub(crate) active_tab: Tab,
    /// Connection to mouse hardware (if available)
    pub(crate) device: Option<IcarusDevice>,
    /// Status message (text, is_success, timestamp)
    pub(crate) status: Option<(String, bool, Instant)>,
    /// Detected firmware version
    pub(crate) firmware_version: String,
    /// Battery level percentage (if wireless or USB)
    pub(crate) battery_level: Option<u8>,
    /// Whether the battery is actively charging or powered via USB
    pub(crate) is_charging: bool,
    /// Battery voltage in millivolts
    pub(crate) battery_voltage_mv: Option<u16>,
    /// Last battery poll timestamp
    pub(crate) last_battery_poll: Instant,
    /// Connection status text
    pub(crate) connection_status: String,
    /// Whether connection is wireless
    pub(crate) is_wireless: bool,
    /// Whether the wireless mouse is actively online and responding
    pub(crate) is_device_online: bool,
    /// Currently hovered button index (0..5) for interactive two-way visual mapping
    pub(crate) hovered_button: Option<usize>,
    /// App start time for animations (LED breathing, neon rainbow cycle)
    pub(crate) start_time: Instant,
    /// Selected macro index in Macro Manager
    pub(crate) selected_macro_idx: usize,
    /// Whether macro recording mode is currently active
    pub(crate) is_recording_macro: bool,
    /// Timestamp of last recorded macro event for calculating natural delays
    pub(crate) last_macro_record_time: Option<Instant>,
    /// 2.4G wireless pairing state
    pub(crate) pairing_state: PairingState,
    /// Start time of current pairing attempt
    pub(crate) pairing_start: Option<Instant>,
    /// Last timestamp when dongle pairing status was polled over USB
    pub(crate) last_pairing_poll: Instant,
    /// Buffer for adding a new macro name
    pub(crate) new_macro_name: String,
    /// Buffer for inserting a key event
    pub(crate) new_key_input: String,
    /// Buffer for inserting delay ms
    pub(crate) new_delay_ms: u32,
    /// Editable text buffers for each DPI stage free-text input (X-axis)
    pub(crate) dpi_text_inputs: Vec<String>,
    /// Editable text buffers for each DPI stage free-text input (Y-axis)
    pub(crate) dpi_y_text_inputs: Vec<String>,
    /// Onboard profile configurations cache (profiles 1 to 5)
    pub(crate) profiles: [MouseConfig; 5],
    /// Index of button currently awaiting interactive keyboard key press binding
    pub(crate) binding_button_idx: Option<usize>,
    /// Firmware update file path
    pub(crate) firmware_update_path: String,
    /// Firmware update target (false = mouse MCU, true = 4K dongle MCU)
    pub(crate) firmware_is_dongle: bool,
    /// Custom names for profiles 1 to 5
    pub(crate) profile_names: [String; 5],
    /// Whether inline profile renaming is active
    pub(crate) is_renaming_profile: bool,
    /// Temporary buffer for profile rename
    pub(crate) profile_rename_buf: String,
    /// Click and debounce test state
    pub(crate) test_clicks_count: u32,
    pub(crate) test_last_click_time: Option<Instant>,
    pub(crate) test_last_click_delta_ms: Option<u64>,
    pub(crate) test_double_click_detected: bool,
    pub(crate) test_recent_clicks: Vec<Instant>,
    pub(crate) test_last_button_name: String,
    /// Profile file path for import/export
    pub(crate) profile_custom_path: String,
    /// Auto-apply lighting changes in real-time
    pub(crate) auto_apply_lighting: bool,
    /// Timestamp of last real-time lighting hardware write (for slider drag throttling)
    pub(crate) last_lighting_apply: Instant,
    /// Whether mouse DPI LED changes are queued for hardware sync
    pub(crate) pending_lighting_sync: bool,
    /// Whether 4K dongle RGB changes are queued for hardware sync
    pub(crate) pending_dongle_sync: bool,
}

impl std::fmt::Debug for IcarusApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcarusApp")
            .field("active_tab", &self.active_tab)
            .field("firmware_version", &self.firmware_version)
            .field("battery_level", &self.battery_level)
            .field("has_device", &self.device.is_some())
            .finish()
    }
}

impl IcarusApp {
    /// Creates and initializes a new `IcarusApp` desktop GUI instance.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders for embedded PNGs
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Configure clean white light gaming theme
        crate::gui::theme::apply_theme(&cc.egui_ctx);

        // Register system fallback font if available for comprehensive unicode coverage
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(font_bytes) = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf") {
            fonts
                .font_data
                .insert("dejavu".to_owned(), egui::FontData::from_owned(font_bytes));
            if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                prop.push("dejavu".to_owned());
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        let cfg_path = crate::daemon::get_default_config_path();
        let default_cfg = if cfg_path.exists() {
            MouseConfig::from_file(&cfg_path).unwrap_or_default()
        } else {
            MouseConfig::default()
        };
        let dpi_text_inputs = default_cfg
            .dpi_stages
            .iter()
            .map(|s| s.dpi.to_string())
            .collect();
        let dpi_y_text_inputs = default_cfg
            .dpi_stages
            .iter()
            .map(|s| s.effective_y().to_string())
            .collect();

        let mut profiles = [
            default_cfg.clone(),
            default_cfg.clone(),
            default_cfg.clone(),
            default_cfg.clone(),
            default_cfg.clone(),
        ];
        for (i, p) in profiles.iter_mut().enumerate() {
            p.active_profile = (i + 1) as u8;
        }

        let mut app = Self {
            config: default_cfg.clone(),
            synced_config: default_cfg,
            profiles,
            dpi_text_inputs,
            dpi_y_text_inputs,
            binding_button_idx: None,
            firmware_update_path: String::new(),
            firmware_is_dongle: false,
            active_tab: Tab::Buttons,
            device: None,
            status: None,
            firmware_version: "Unknown".into(),
            battery_level: None,
            is_charging: false,
            battery_voltage_mv: None,
            last_battery_poll: Instant::now(),
            connection_status: "Disconnected (Offline Mode)".into(),
            is_wireless: false,
            is_device_online: false,
            hovered_button: None,
            start_time: Instant::now(),
            selected_macro_idx: 0,
            is_recording_macro: false,
            last_macro_record_time: None,
            pairing_state: PairingState::Idle,
            pairing_start: None,
            last_pairing_poll: Instant::now(),
            new_macro_name: String::new(),
            new_key_input: "Space".into(),
            new_delay_ms: 20,
            profile_names: [
                "Default".into(),
                "Gaming".into(),
                "FPS / Esports".into(),
                "MOBA".into(),
                "Work / Daily".into(),
            ],
            is_renaming_profile: false,
            profile_rename_buf: String::new(),
            test_clicks_count: 0,
            test_last_click_time: None,
            test_last_click_delta_ms: None,
            test_double_click_detected: false,
            test_recent_clicks: Vec::new(),
            test_last_button_name: "None".into(),
            profile_custom_path: "icarus-profile.toml".into(),
            auto_apply_lighting: true,
            last_lighting_apply: Instant::now(),
            pending_lighting_sync: false,
            pending_dongle_sync: false,
        };

        app.refresh_device();
        app
    }

    /// Refresh device detection and read live hardware status
    pub(crate) fn refresh_device(&mut self) {
        match IcarusDevice::open() {
            Ok(dev) => {
                let conn_lower = dev.connection.to_lowercase();
                self.is_wireless = conn_lower.contains("wireless");
                let is_online = dev.read_device_online().unwrap_or(true);
                self.is_device_online = is_online;

                if self.is_wireless && !is_online {
                    self.connection_status =
                        self.config.language.dongle_ready_mouse_offline().into();
                    let hint = match self.config.language {
                        AppLanguage::English => {
                            "2.4G Dongle connected, but mouse is offline/asleep. Turn power switch under mouse ON or move mouse."
                        }
                        AppLanguage::Indonesian => {
                            "Dongle 2.4G terhubung, namun mouse offline/sleep. Nyalakan saklar di bawah mouse atau gerakkan mouse."
                        }
                    };
                    self.set_status(hint, false);
                } else {
                    let conn_label = match self.config.language {
                        AppLanguage::English => format!("Connected ({})", dev.connection),
                        AppLanguage::Indonesian => format!("Terhubung ({})", dev.connection),
                    };
                    self.connection_status = conn_label;
                    if let Ok(status) = dev.read_battery_status() {
                        self.battery_level = Some(status.level);
                        self.is_charging = status.is_charging;
                        self.battery_voltage_mv = if status.voltage_mv > 0 {
                            Some(status.voltage_mv)
                        } else {
                            None
                        };
                    } else if !self.is_wireless {
                        self.battery_level = Some(100);
                        self.is_charging = true;
                        self.battery_voltage_mv = Some(4200);
                    }
                    let conn_msg = match self.config.language {
                        AppLanguage::English => "Connected to ICARUS V2 MINI",
                        AppLanguage::Indonesian => "Terhubung ke ICARUS V2 MINI",
                    };
                    self.set_status(conn_msg, true);
                }

                if is_online && let Ok(fw) = dev.read_firmware_version() {
                    self.firmware_version = fw;
                }

                if is_online && let Ok(mut live_cfg) = dev.read_config() {
                    // Preserve desktop-level and host-persisted settings loaded from config.toml
                    live_cfg.language = self.config.language;
                    live_cfg.macros = std::mem::take(&mut self.config.macros);
                    live_cfg.auto_profile_rules =
                        std::mem::take(&mut self.config.auto_profile_rules);
                    live_cfg.auto_profile_enabled = self.config.auto_profile_enabled;
                    live_cfg.dongle_rgb = self.config.dongle_rgb;
                    live_cfg.led_sleep_seconds = self.config.led_sleep_seconds;
                    live_cfg.active_profile = self.config.active_profile.clamp(1, 5);

                    self.config = live_cfg;
                    self.sync_dpi_text_inputs();
                    self.sync_active_profile_slot();
                    self.synced_config = self.config.clone();
                }

                self.device = Some(dev);
            }
            Err(_) => {
                self.device = None;
                self.connection_status = self.config.language.disconnected().into();
                self.battery_level = None;
                self.is_charging = false;
                self.battery_voltage_mv = None;
                self.is_wireless = false;
                self.is_device_online = false;
            }
        }
    }

    /// Set a temporary status notification
    pub(crate) fn set_status(&mut self, msg: impl Into<String>, success: bool) {
        self.status = Some((msg.into(), success, Instant::now()));
    }

    /// Apply all current GUI settings to the physical mouse
    pub(crate) fn apply_to_device(&mut self) {
        if let Err(err) = self.config.validate_buttons() {
            self.set_status(format!("⚠️ {}", err), false);
            return;
        }
        self.config.validate();
        if let Some(ref dev) = self.device {
            let online = dev.read_device_online().unwrap_or(true);
            self.is_device_online = online;

            if self.is_wireless && !online {
                // Still apply 4K dongle RGB to the receiver and save local config
                let _ = dev.apply_dongle_rgb(&self.config.dongle_rgb);
                self.sync_active_profile_slot();
                let _ = self
                    .config
                    .to_file(crate::daemon::get_default_config_path());
                self.set_status(self.config.language.status_mouse_offline_hint(), false);
                return;
            }

            match dev.apply_config(&self.config) {
                Ok(_) => {
                    self.synced_config = self.config.clone();
                    self.sync_active_profile_slot();
                    let _ = self
                        .config
                        .to_file(crate::daemon::get_default_config_path());
                    let msg = match self.config.language {
                        AppLanguage::English => "Settings applied successfully to mouse!",
                        AppLanguage::Indonesian => "Pengaturan berhasil diterapkan ke mouse!",
                    };
                    self.set_status(msg, true);
                }
                Err(e) => {
                    let msg = match self.config.language {
                        AppLanguage::English => format!("Error applying settings: {}", e),
                        AppLanguage::Indonesian => format!("Gagal menerapkan pengaturan: {}", e),
                    };
                    self.set_status(msg, false);
                }
            }
        } else {
            let msg = match self.config.language {
                AppLanguage::English => "Mouse not connected. Connect mouse to apply changes.",
                AppLanguage::Indonesian => {
                    "Mouse tidak terhubung. Hubungkan mouse untuk menerapkan perubahan."
                }
            };
            self.set_status(msg, false);
        }
    }

    /// Read all current settings from the physical mouse
    pub(crate) fn read_from_device(&mut self) {
        if let Some(ref dev) = self.device {
            let online = dev.read_device_online().unwrap_or(true);
            self.is_device_online = online;

            if self.is_wireless && !online {
                self.set_status(self.config.language.status_mouse_offline_hint(), false);
                return;
            }

            match dev.read_config() {
                Ok(mut cfg) => {
                    // Preserve UI-only and desktop-level settings not stored in mouse registers
                    cfg.language = self.config.language;
                    cfg.macros = std::mem::take(&mut self.config.macros);
                    cfg.auto_profile_rules = std::mem::take(&mut self.config.auto_profile_rules);
                    cfg.auto_profile_enabled = self.config.auto_profile_enabled;
                    cfg.dongle_rgb = self.config.dongle_rgb;
                    cfg.led_sleep_seconds = self.config.led_sleep_seconds;
                    cfg.active_profile = self.config.active_profile.clamp(1, 5);

                    self.synced_config = cfg.clone();
                    self.config = cfg;
                    if let Ok(status) = dev.read_battery_status() {
                        self.battery_level = Some(status.level);
                        self.is_charging = status.is_charging;
                        if status.voltage_mv > 0 {
                            self.battery_voltage_mv = Some(status.voltage_mv);
                        }
                    }
                    self.sync_dpi_text_inputs();
                    self.sync_active_profile_slot();
                    let msg = match self.config.language {
                        AppLanguage::English => "Configuration loaded from mouse!",
                        AppLanguage::Indonesian => "Konfigurasi berhasil dimuat dari mouse!",
                    };
                    self.set_status(msg, true);
                }
                Err(e) => {
                    // Invalidate handle if communication failed with HID error
                    if let Some(crate::protocol::ProtocolError::Hid(_)) =
                        e.downcast_ref::<crate::protocol::ProtocolError>()
                    {
                        self.device = None;
                        self.connection_status = self.config.language.disconnected().into();
                    }
                    let msg = match self.config.language {
                        AppLanguage::English => format!("Failed to read from mouse: {}", e),
                        AppLanguage::Indonesian => format!("Gagal membaca dari mouse: {}", e),
                    };
                    self.set_status(msg, false);
                }
            }
        } else {
            self.refresh_device();
        }
    }

    /// Synchronize active profile slot in `self.profiles` with current `self.config`
    pub(crate) fn sync_active_profile_slot(&mut self) {
        let idx = (self.config.active_profile as usize).saturating_sub(1);
        if idx < self.profiles.len() {
            self.profiles[idx] = self.config.clone();
        }
    }

    /// Synchronize editable DPI text buffers with active config
    pub(crate) fn sync_dpi_text_inputs(&mut self) {
        self.dpi_text_inputs = self
            .config
            .dpi_stages
            .iter()
            .map(|s| s.dpi.to_string())
            .collect();
        self.dpi_y_text_inputs = self
            .config
            .dpi_stages
            .iter()
            .map(|s| s.effective_y().to_string())
            .collect();
    }

    /// Apply active lighting configuration directly to physical mouse in real time
    pub(crate) fn apply_lighting_to_device(&mut self) {
        self.pending_lighting_sync = false;
        self.last_lighting_apply = Instant::now();
        if self.is_wireless && !self.is_device_online {
            return;
        }
        let mut fatal_err = false;
        if let Some(ref dev) = self.device {
            let active_idx = self.config.active_dpi_stage;
            let _ = dev.set_dpi_stage_color(active_idx, self.config.dpi_led.color);
            if let Err(e) = dev.set_dpi_led(&self.config.dpi_led)
                && let Some(crate::protocol::ProtocolError::Hid(_)) =
                    e.downcast_ref::<crate::protocol::ProtocolError>()
            {
                fatal_err = true;
            }
        }
        if fatal_err {
            self.device = None;
            self.connection_status = self.config.language.disconnected().into();
        }
    }

    /// Apply 4K dongle RGB configuration directly to receiver hardware in real time
    pub(crate) fn apply_dongle_to_device(&mut self) {
        self.pending_dongle_sync = false;
        self.last_lighting_apply = Instant::now();
        let mut fatal_err = false;
        if let Some(ref dev) = self.device
            && let Err(e) = dev.apply_dongle_rgb(&self.config.dongle_rgb)
            && let Some(crate::protocol::ProtocolError::Hid(_)) =
                e.downcast_ref::<crate::protocol::ProtocolError>()
        {
            fatal_err = true;
        }
        if fatal_err {
            self.device = None;
            self.connection_status = self.config.language.disconnected().into();
        }
    }

    /// Queue a throttled mouse lighting hardware update (prevents USB flooding during slider drags)
    pub(crate) fn queue_lighting_sync(&mut self) {
        self.pending_lighting_sync = true;
    }

    /// Queue a throttled 4K dongle RGB hardware update (prevents USB flooding during slider drags)
    pub(crate) fn queue_dongle_sync(&mut self) {
        self.pending_dongle_sync = true;
    }

    /// Flush any pending real-time lighting updates when throttle interval elapses or pointer drag ends
    pub(crate) fn flush_pending_lighting_sync(&mut self, pointer_down: bool) {
        if !self.pending_lighting_sync && !self.pending_dongle_sync {
            return;
        }
        if !pointer_down
            || self.last_lighting_apply.elapsed() >= std::time::Duration::from_millis(120)
        {
            if self.pending_lighting_sync {
                self.apply_lighting_to_device();
            }
            if self.pending_dongle_sync {
                self.apply_dongle_to_device();
            }
        }
    }
}
