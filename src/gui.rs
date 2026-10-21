//! Graphical User Interface (GUI) for ICARUS V2 MINI.
//!
//! A native Linux desktop configuration tool recreating the original
//! Windows application with all reverse-engineered features, original icons, and graphics.

use crate::config::{
    ButtonAction, Color, LedMode, LiftOffDistance, MacroEvent, MacroEventType, MacroItem,
    MacroRepeatMode, MouseConfig, PollingRate,
};
use crate::device::IcarusDevice;
use eframe::egui;
use std::time::{Duration, Instant};

/// Standalone native vector toggle switch with smooth animated slide transition.
fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(38.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
    let radius = 0.5 * rect.height();

    // Background track color interpolated between off (slate 203, 213, 225) and on (sky blue 2, 132, 199)
    let off_col = egui::Color32::from_rgb(203, 213, 225);
    let on_col = egui::Color32::from_rgb(2, 132, 199);
    let bg_col = egui::Color32::from_rgb(
        (off_col.r() as f32 + (on_col.r() as f32 - off_col.r() as f32) * how_on) as u8,
        (off_col.g() as f32 + (on_col.g() as f32 - off_col.g() as f32) * how_on) as u8,
        (off_col.b() as f32 + (on_col.b() as f32 - off_col.b() as f32) * how_on) as u8,
    );
    ui.painter().rect_filled(rect, radius, bg_col);

    // Circular white knob with subtle boundary stroke
    let circle_x =
        (rect.left() + radius) + ((rect.right() - radius) - (rect.left() + radius)) * how_on;
    let center = egui::pos2(circle_x, rect.center().y);
    ui.painter()
        .circle_filled(center, radius - 2.5, egui::Color32::WHITE);
    ui.painter().circle_stroke(
        center,
        radius - 2.5,
        egui::Stroke::new(0.5_f32, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30)),
    );

    response
}

/// Convert HSV values (h: 0.0..1.0, s: 0.0..1.0, v: 0.0..1.0) to RGB Color32.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> egui::Color32 {
    let c = v * s;
    let h_prime = (h % 1.0) * 6.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    egui::Color32::from_rgb(
        ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

/// Return category badge text and distinct accent color for button actions.
fn action_badge(action: &ButtonAction) -> (&'static str, egui::Color32) {
    match action {
        ButtonAction::MouseButton(_) => ("CLICK", egui::Color32::from_rgb(2, 132, 199)),
        ButtonAction::DpiSwitch => ("DPI", egui::Color32::from_rgb(6, 182, 212)),
        ButtonAction::FireKey { .. } => ("FIRE", egui::Color32::from_rgb(239, 68, 68)),
        ButtonAction::Media(_) => ("MEDIA", egui::Color32::from_rgb(245, 158, 11)),
        ButtonAction::Keyboard { .. } => ("KEY", egui::Color32::from_rgb(139, 92, 246)),
        ButtonAction::Macro(_) => ("MACRO", egui::Color32::from_rgb(16, 185, 129)),
        ButtonAction::Disabled => ("OFF", egui::Color32::from_rgb(148, 163, 184)),
    }
}

/// Active navigation tab in the desktop application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Buttons,
    Sensor,
    Lighting,
    Macro,
    Settings,
}

/// Pairing state for 2.4G wireless dongle tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingState {
    Idle,
    Pairing,
    Success,
    #[allow(dead_code)]
    Failed,
}

/// The main application state.
pub struct IcarusApp {
    /// Active mouse configuration
    config: MouseConfig,
    /// Last saved or hardware-synced mouse configuration for unsaved changes detection
    synced_config: MouseConfig,
    /// Currently selected tab
    active_tab: Tab,
    /// Connection to mouse hardware (if available)
    device: Option<IcarusDevice>,
    /// Status message (text, is_success, timestamp)
    status: Option<(String, bool, Instant)>,
    /// Detected firmware version
    firmware_version: String,
    /// Battery level percentage (if wireless)
    battery_level: Option<u8>,
    /// Connection status text
    connection_status: String,
    /// Whether connection is wireless
    is_wireless: bool,
    /// Whether the wireless mouse is actively online and responding
    is_device_online: bool,
    /// Currently hovered button index (0..5) for interactive two-way visual mapping
    hovered_button: Option<usize>,
    /// App start time for animations (LED breathing, neon rainbow cycle)
    start_time: Instant,
    /// Selected macro index in Macro Manager
    selected_macro_idx: usize,
    /// Whether macro recording mode is currently active
    is_recording_macro: bool,
    /// 2.4G wireless pairing state
    pairing_state: PairingState,
    /// Start time of current pairing attempt
    pairing_start: Option<Instant>,
    /// Buffer for adding a new macro name
    new_macro_name: String,
    /// Buffer for inserting a key event
    new_key_input: String,
    /// Buffer for inserting delay ms
    new_delay_ms: u32,
}

impl IcarusApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders for embedded PNGs
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Configure clean white light gaming theme
        Self::apply_theme(&cc.egui_ctx);

        let default_cfg = MouseConfig::default();

        let mut app = Self {
            config: default_cfg.clone(),
            synced_config: default_cfg,
            active_tab: Tab::Buttons,
            device: None,
            status: None,
            firmware_version: "Unknown".into(),
            battery_level: None,
            connection_status: "Disconnected (Offline Mode)".into(),
            is_wireless: false,
            is_device_online: false,
            hovered_button: None,
            start_time: Instant::now(),
            selected_macro_idx: 0,
            is_recording_macro: false,
            pairing_state: PairingState::Idle,
            pairing_start: None,
            new_macro_name: String::new(),
            new_key_input: "Space".into(),
            new_delay_ms: 20,
        };

        app.refresh_device();
        app
    }

    /// Set up custom clean white theme for the application
    fn apply_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::light();

        visuals.panel_fill = egui::Color32::from_rgb(255, 255, 255);
        visuals.window_fill = egui::Color32::from_rgb(248, 250, 252);
        visuals.extreme_bg_color = egui::Color32::from_rgb(241, 245, 249);
        visuals.faint_bg_color = egui::Color32::from_rgb(248, 250, 252);

        visuals.override_text_color = Some(egui::Color32::from_rgb(15, 23, 42));

        visuals.selection.bg_fill = egui::Color32::from_rgb(2, 132, 199);
        visuals.selection.stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(14, 165, 233));

        visuals.slider_trailing_fill = true;

        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
        visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240));
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(15, 23, 42));
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

        // Visible light slate rail for sliders and inactive controls
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(226, 232, 240);
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225));
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(100, 116, 139));
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(219, 234, 254);
        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199));
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199));
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);

        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(2, 132, 199);
        visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199));
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(15, 23, 42));
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);

        visuals.window_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240));

        ctx.set_visuals(visuals);

        ctx.style_mut(|style| {
            style.spacing.slider_rail_height = 8.0;
        });
    }

    /// Refresh device detection and read live hardware status
    fn refresh_device(&mut self) {
        match IcarusDevice::open() {
            Ok(dev) => {
                let conn_lower = dev.connection.to_lowercase();
                self.is_wireless = conn_lower.contains("wireless");
                let is_online = dev.read_device_online().unwrap_or(true);
                self.is_device_online = is_online;

                if self.is_wireless && !is_online {
                    self.connection_status = "Dongle Ready (Mouse Offline)".into();
                    self.set_status(
                        "Dongle 2.4G terhubung, namun mouse offline/sleep. Nyalakan saklar di bawah mouse atau gerakkan mouse.",
                        false,
                    );
                } else {
                    self.connection_status = format!("Connected ({})", dev.connection);
                    if let Ok(b) = dev.read_battery_level() {
                        self.battery_level = Some(b);
                    } else if self.battery_level.is_none() {
                        self.battery_level = Some(100);
                    }
                    self.set_status("Connected to ICARUS V2 MINI", true);
                }

                if let Ok(fw) = dev.read_firmware_version() {
                    self.firmware_version = fw;
                }

                if is_online {
                    #[allow(clippy::collapsible_if)]
                    if let Ok(live_cfg) = dev.read_config() {
                        self.config = live_cfg.clone();
                        self.synced_config = live_cfg;
                    }
                }

                self.device = Some(dev);
            }
            Err(_) => {
                self.device = None;
                self.connection_status = "Disconnected (Offline Mode)".into();
                self.battery_level = None;
                self.is_wireless = false;
                self.is_device_online = false;
            }
        }
    }

    /// Set a temporary status notification
    fn set_status(&mut self, msg: impl Into<String>, success: bool) {
        self.status = Some((msg.into(), success, Instant::now()));
    }

    /// Apply all current GUI settings to the physical mouse
    fn apply_to_device(&mut self) {
        self.config.validate();
        if let Some(ref dev) = self.device {
            let online = dev.read_device_online().unwrap_or(true);
            self.is_device_online = online;

            match dev.apply_config(&self.config) {
                Ok(_) => {
                    self.synced_config = self.config.clone();
                    self.set_status("Settings applied successfully to mouse!", true);
                }
                Err(e) => self.set_status(format!("Error applying settings: {}", e), false),
            }
        } else {
            self.set_status(
                "Mouse not connected. Connect mouse to apply changes.",
                false,
            );
        }
    }

    /// Read all current settings from the physical mouse
    fn read_from_device(&mut self) {
        if let Some(ref dev) = self.device {
            let online = dev.read_device_online().unwrap_or(true);
            self.is_device_online = online;

            if self.is_wireless && !online {
                self.set_status(
                    "Mouse sedang offline/sleep. Gerakkan mouse atau nyalakan saklar terlebih dahulu.",
                    false,
                );
                return;
            }

            match dev.read_config() {
                Ok(cfg) => {
                    self.config = cfg.clone();
                    self.synced_config = cfg;
                    self.battery_level = dev.read_battery_level().ok().filter(|&b| b > 0);
                    self.set_status("Configuration loaded from mouse!", true);
                }
                Err(e) => self.set_status(format!("Failed to read from mouse: {}", e), false),
            }
        } else {
            self.refresh_device();
        }
    }
}

impl eframe::App for IcarusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update simulated pairing state if in progress
        if self.pairing_state == PairingState::Pairing
            && let Some(start) = self.pairing_start
            && start.elapsed() >= Duration::from_millis(1800)
        {
            self.pairing_state = PairingState::Success;
            self.set_status("2.4G Wireless Dongle paired successfully!", true);
        }

        // Top Header Bar featuring original reverse-engineered Logo, Profile Selector & Connection Indicators
        egui::TopBottomPanel::top("header_panel")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(255, 255, 255))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(226, 232, 240),
                    ))
                    .inner_margin(egui::Margin::symmetric(20.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Original ICARUS Logo Image
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Dialog/Logo.png"))
                            .max_width(28.0)
                            .max_height(28.0),
                    );

                    ui.add_space(8.0);

                    // App Title & Tagline
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("ICARUS V2 MINI")
                                    .size(18.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(2, 132, 199)),
                            );
                            ui.label(
                                egui::RichText::new("PAW3395 LINUX SOFTWARE")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(100, 116, 139)),
                            );
                        });
                    });

                    ui.add_space(24.0);

                    // Segmented Pill Profile Selector (Profiles 1 to 5)
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                        ui.label(
                            egui::RichText::new("Profile:")
                                .size(12.5)
                                .color(egui::Color32::from_rgb(71, 85, 105))
                                .strong(),
                        );
                        ui.add_space(2.0);

                        for p in 1..=5 {
                            let is_active = self.config.active_profile == p;
                            let pill_btn = egui::Button::new(
                                egui::RichText::new(format!("{}", p))
                                    .size(12.5)
                                    .strong()
                                    .color(if is_active {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::from_rgb(51, 65, 85)
                                    }),
                            )
                            .fill(if is_active {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(241, 245, 249)
                            })
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                if is_active {
                                    egui::Color32::from_rgb(2, 132, 199)
                                } else {
                                    egui::Color32::from_rgb(226, 232, 240)
                                },
                            ))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(28.0, 26.0));

                            if ui
                                .add(pill_btn)
                                .on_hover_text(format!("Switch to Profile {}", p))
                                .clicked()
                            {
                                self.config.active_profile = p;
                                self.set_status(format!("Switched to Profile {}", p), true);
                            }
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Scan Devices Button
                        let scan_btn = egui::Button::new(
                            egui::RichText::new("Scan Devices")
                                .size(12.0)
                                .strong()
                                .color(egui::Color32::from_rgb(51, 65, 85)),
                        )
                        .fill(egui::Color32::from_rgb(241, 245, 249))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(203, 213, 225),
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(96.0, 28.0));

                        if ui.add(scan_btn).clicked() {
                            self.refresh_device();
                        }

                        ui.add_space(8.0);

                        // Battery Indicator Pill
                        if let Some(battery) = self.battery_level {
                            let (bg_color, fg_color, bar_color) = if battery > 50 {
                                (
                                    egui::Color32::from_rgb(240, 253, 244),
                                    egui::Color32::from_rgb(22, 163, 74),
                                    egui::Color32::from_rgb(34, 197, 94),
                                )
                            } else if battery > 20 {
                                (
                                    egui::Color32::from_rgb(254, 243, 199),
                                    egui::Color32::from_rgb(180, 83, 9),
                                    egui::Color32::from_rgb(245, 158, 11),
                                )
                            } else {
                                (
                                    egui::Color32::from_rgb(254, 242, 242),
                                    egui::Color32::from_rgb(220, 38, 38),
                                    egui::Color32::from_rgb(239, 68, 68),
                                )
                            };

                            let (pill_rect, _) = ui
                                .allocate_exact_size(egui::vec2(104.0, 28.0), egui::Sense::hover());
                            ui.painter().rect_filled(pill_rect, 6.0, bg_color);
                            ui.painter().rect_stroke(
                                pill_rect,
                                6.0,
                                egui::Stroke::new(1.0_f32, bar_color),
                            );

                            // Mini progress bar inside the pill
                            let bar_width = 24.0;
                            let bar_height = 8.0;
                            let bar_rect = egui::Rect::from_min_size(
                                egui::pos2(pill_rect.left() + 8.0, pill_rect.center().y - 4.0),
                                egui::vec2(bar_width, bar_height),
                            );
                            ui.painter().rect_filled(
                                bar_rect,
                                2.0,
                                egui::Color32::from_rgb(226, 232, 240),
                            );
                            let fill_w = bar_width * (battery.min(100) as f32 / 100.0);
                            let fill_rect = egui::Rect::from_min_size(
                                bar_rect.min,
                                egui::vec2(fill_w, bar_height),
                            );
                            ui.painter().rect_filled(fill_rect, 2.0, bar_color);

                            ui.painter().text(
                                egui::pos2(pill_rect.left() + 38.0, pill_rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                format!("{}%", battery),
                                egui::FontId::proportional(12.0),
                                fg_color,
                            );

                            ui.add_space(8.0);
                        }

                        // Connection Status Pill Badge
                        let is_connected = self.device.is_some();
                        let is_dongle_offline = is_connected && self.is_wireless && !self.is_device_online;

                        let (pill_bg, pill_stroke, dot_color, text_color, label_text, pill_width) = if !is_connected {
                            (
                                egui::Color32::from_rgb(254, 242, 242),
                                egui::Color32::from_rgb(254, 202, 202),
                                egui::Color32::from_rgb(239, 68, 68),
                                egui::Color32::from_rgb(220, 38, 38),
                                "Offline Mode",
                                145.0,
                            )
                        } else if is_dongle_offline {
                            (
                                egui::Color32::from_rgb(254, 243, 199),
                                egui::Color32::from_rgb(253, 230, 138),
                                egui::Color32::from_rgb(245, 158, 11),
                                egui::Color32::from_rgb(180, 83, 9),
                                "Dongle Ready (Mouse Offline)",
                                205.0,
                            )
                        } else {
                            (
                                egui::Color32::from_rgb(240, 253, 244),
                                egui::Color32::from_rgb(187, 247, 208),
                                egui::Color32::from_rgb(34, 197, 94),
                                egui::Color32::from_rgb(22, 163, 74),
                                self.connection_status.as_str(),
                                175.0,
                            )
                        };

                        let (stat_pill, pill_resp) =
                            ui.allocate_exact_size(egui::vec2(pill_width, 28.0), egui::Sense::hover());
                        ui.painter().rect_filled(stat_pill, 6.0, pill_bg);
                        ui.painter().rect_stroke(
                            stat_pill,
                            6.0,
                            egui::Stroke::new(1.0_f32, pill_stroke),
                        );

                        // Indicator Dot
                        let dot_pos = egui::pos2(stat_pill.left() + 12.0, stat_pill.center().y);
                        ui.painter().circle_filled(dot_pos, 4.5, dot_color);

                        ui.painter().text(
                            egui::pos2(stat_pill.left() + 24.0, stat_pill.center().y),
                            egui::Align2::LEFT_CENTER,
                            label_text,
                            egui::FontId::proportional(11.5),
                            text_color,
                        );

                        if is_dongle_offline {
                            pill_resp.on_hover_text(
                                "Dongle 2.4G terdeteksi, tetapi mouse tidak merespons.\n\
                                 Pastikan saklar di bawah mouse posisi ON (2.4G), gerakkan mouse untuk wake up, atau colok kabel jika baterai habis.",
                            );
                        }
                    });
                });

                ui.add_space(10.0);

                // Modern Floating Pill Tab Navigation Bar
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                    let tabs = [
                        Tab::Buttons,
                        Tab::Sensor,
                        Tab::Lighting,
                        Tab::Macro,
                        Tab::Settings,
                    ];

                    for tab in tabs {
                        let is_active = self.active_tab == tab;

                        let (icon_image, label_text) = match (tab, is_active) {
                            (Tab::Buttons, true) => (
                                egui::include_image!("../assets/Dialog/HomeChecked.png"),
                                "Buttons",
                            ),
                            (Tab::Buttons, false) => {
                                (egui::include_image!("../assets/Dialog/Home.png"), "Buttons")
                            }
                            (Tab::Sensor, true) => (
                                egui::include_image!("../assets/Dialog/SensorChecked.png"),
                                "DPI & Sensor",
                            ),
                            (Tab::Sensor, false) => (
                                egui::include_image!("../assets/Dialog/Sensor.png"),
                                "DPI & Sensor",
                            ),
                            (Tab::Lighting, true) => (
                                egui::include_image!("../assets/Dialog/LightChecked.png"),
                                "Lighting",
                            ),
                            (Tab::Lighting, false) => (
                                egui::include_image!("../assets/Dialog/Light.png"),
                                "Lighting",
                            ),
                            (Tab::Macro, true) => (
                                egui::include_image!("../assets/Dialog/MacroChecked.png"),
                                "Macro",
                            ),
                            (Tab::Macro, false) => {
                                (egui::include_image!("../assets/Dialog/Macro.png"), "Macro")
                            }
                            (Tab::Settings, true) => (
                                egui::include_image!("../assets/Dialog/SettingClick.png"),
                                "Settings & 2.4G",
                            ),
                            (Tab::Settings, false) => (
                                egui::include_image!("../assets/Dialog/Setting.png"),
                                "Settings & 2.4G",
                            ),
                        };

                        let text = egui::RichText::new(label_text).size(13.0).strong().color(
                            if is_active {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(71, 85, 105)
                            },
                        );

                        let btn = egui::Button::image_and_text(
                            egui::Image::new(icon_image).max_height(18.0),
                            text,
                        )
                        .fill(if is_active {
                            egui::Color32::from_rgb(240, 249, 255)
                        } else {
                            egui::Color32::from_rgb(255, 255, 255)
                        })
                        .stroke(if is_active {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240))
                        })
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(138.0, 36.0));

                        if ui.add(btn).clicked() {
                            self.active_tab = tab;
                        }
                    }
                });
            });

        // Bottom Action Footer Bar with Unsaved Changes Indicator
        egui::TopBottomPanel::bottom("footer_panel")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(255, 255, 255))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(226, 232, 240),
                    ))
                    .inner_margin(egui::Margin::symmetric(20.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let has_unsaved = self.config != self.synced_config;

                    // Unsaved Changes Pill Badge
                    if has_unsaved {
                        let (unsaved_pill, _) =
                            ui.allocate_exact_size(egui::vec2(148.0, 28.0), egui::Sense::hover());
                        ui.painter().rect_filled(
                            unsaved_pill,
                            6.0,
                            egui::Color32::from_rgb(254, 243, 199),
                        );
                        ui.painter().rect_stroke(
                            unsaved_pill,
                            6.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(245, 158, 11)),
                        );

                        let dot_pt =
                            egui::pos2(unsaved_pill.left() + 10.0, unsaved_pill.center().y);
                        ui.painter().circle_filled(
                            dot_pt,
                            4.0,
                            egui::Color32::from_rgb(217, 119, 6),
                        );

                        ui.painter().text(
                            egui::pos2(unsaved_pill.left() + 20.0, unsaved_pill.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Unsaved Changes",
                            egui::FontId::proportional(12.0),
                            egui::Color32::from_rgb(180, 83, 9),
                        );
                        ui.add_space(8.0);
                    }

                    // Status feedback message
                    if let Some((msg, success, time)) = &self.status
                        && time.elapsed() < Duration::from_secs(6)
                    {
                        let col = if *success {
                            egui::Color32::from_rgb(22, 163, 74)
                        } else {
                            egui::Color32::from_rgb(220, 38, 38)
                        };
                        ui.label(egui::RichText::new(msg).color(col).strong());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Apply button (Primary) with dynamic accent highlight if unsaved changes exist
                        let apply_btn = egui::Button::new(
                            egui::RichText::new("Apply Changes")
                                .size(13.5)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(if has_unsaved {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(3, 105, 161)
                        })
                        .stroke(if has_unsaved {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(56, 189, 248))
                        } else {
                            egui::Stroke::NONE
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(142.0, 36.0));

                        if ui.add(apply_btn).clicked() {
                            self.apply_to_device();
                        }

                        // Read from mouse button
                        let read_btn = egui::Button::new(
                            egui::RichText::new("Read from Mouse")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(51, 65, 85)),
                        )
                        .fill(egui::Color32::from_rgb(241, 245, 249))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(203, 213, 225),
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(130.0, 36.0));

                        if ui.add(read_btn).clicked() {
                            self.read_from_device();
                        }

                        // Reset button
                        let reset_btn = egui::Button::new(
                            egui::RichText::new("Reset Defaults")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(100, 116, 139)),
                        )
                        .fill(egui::Color32::from_rgb(248, 250, 252))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(226, 232, 240),
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(120.0, 36.0));

                        if ui
                            .add(reset_btn)
                            .on_hover_text("Reset configuration to factory defaults")
                            .clicked()
                        {
                            self.config = MouseConfig::default();
                            self.set_status(
                                "Reset to factory defaults (click Apply to save)",
                                true,
                            );
                        }
                    });
                });
            });

        // Main Content Area
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(248, 250, 252))
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);
                    match self.active_tab {
                        Tab::Buttons => self.render_buttons_tab(ui),
                        Tab::Sensor => self.render_sensor_tab(ui),
                        Tab::Lighting => self.render_lighting_tab(ui),
                        Tab::Macro => self.render_macro_tab(ui),
                        Tab::Settings => self.render_settings_tab(ui),
                    }
                    ui.add_space(16.0);
                });
            });
    }
}

impl IcarusApp {
    /// Render Tab 1: Button Mapping with interactive 3D mouse visualizer and 6 ordered button cards
    fn render_buttons_tab(&mut self, ui: &mut egui::Ui) {
        let mut next_hovered = None;

        // Button hardware definitions (Name, Physical role note, Hardware button index 0..5)
        let button_definitions = [
            ("Button 1: Left Click", "Klik Kiri", 0),
            ("Button 2: Right Click", "Klik Kanan", 1),
            ("Button 3: Middle Click (Wheel)", "Klik Roda Tengah", 2),
            ("Button 4: Backward (Side 1)", "Tombol Samping Belakang", 3),
            ("Button 5: Forward (Side 2)", "Tombol Samping Depan", 4),
            ("Button 6: DPI Cycle Button", "Bawah Kaki Mouse (Underside)", 5),
        ];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;

            // ==========================================
            // LEFT COLUMN: Interactive Mouse Visualizer & Debounce
            // ==========================================
            ui.vertical(|ui| {
                ui.set_width(410.0);

                // Mouse Showcase Card
                let showcase_frame = egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(226, 232, 240),
                    ))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 12.0));

                showcase_frame.show(ui, |ui| {
                    ui.set_width(382.0);

                    // Header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Mouse Hardware Visualizer")
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::from_rgb(2, 132, 199)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("PAW3395 • nRF52840")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(148, 163, 184)),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // 3D Mouse Image Canvas
                    let img = egui::Image::new(egui::include_image!(
                        "../assets/Buttoninterface/mouse0.png"
                    ))
                    .fit_to_exact_size(egui::vec2(340.0, 280.0));

                    let img_resp = ui.centered_and_justified(|ui| ui.add(img)).inner;
                    let rect = img_resp.rect;
                    let w = rect.width();
                    let h = rect.height();

                    // Pin coordinates calibrated to physical button locations on mouse0.png (432x356)
                    let pin_coords = [
                        (0, egui::pos2(rect.left() + w * 0.227, rect.top() + h * 0.433)), // 1: Left Click (Depan Kiri)
                        (1, egui::pos2(rect.left() + w * 0.417, rect.top() + h * 0.416)), // 2: Right Click (Posisi tombol kanan)
                        (2, egui::pos2(rect.left() + w * 0.326, rect.top() + h * 0.387)), // 3: Middle Wheel (Roda Scroll)
                        (3, egui::pos2(rect.left() + w * 0.685, rect.top() + h * 0.491)), // 4: Backward Side (Belakang)
                        (4, egui::pos2(rect.left() + w * 0.586, rect.top() + h * 0.519)), // 5: Forward Side (Depan)
                        (5, egui::pos2(rect.left() + w * 0.380, rect.top() + h * 0.690)), // 6: DPI Switch (Bawah Kaki Mouse)
                    ];

                    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

                    // Register hover and click interactions for pins
                    for (btn_idx, pin_pos) in &pin_coords {
                        let pin_rect = egui::Rect::from_center_size(*pin_pos, egui::vec2(24.0, 24.0));
                        let pin_interact = ui.interact(
                            pin_rect,
                            ui.id().with(("mouse_pin", *btn_idx)),
                            egui::Sense::click_and_drag(),
                        );
                        if pin_interact.hovered() || pointer_pos.is_some_and(|p| pin_rect.contains(p)) {
                            next_hovered = Some(*btn_idx);
                        }

                        let is_hl = self.hovered_button == Some(*btn_idx);
                        if is_hl {
                            // Pulsing / glowing outer halo
                            ui.painter().circle_filled(
                                *pin_pos,
                                14.0,
                                egui::Color32::from_rgba_unmultiplied(2, 132, 199, 65),
                            );
                            ui.painter().circle_filled(
                                *pin_pos,
                                10.0,
                                egui::Color32::from_rgb(2, 132, 199),
                            );
                            ui.painter().circle_stroke(
                                *pin_pos,
                                10.0,
                                egui::Stroke::new(1.8_f32, egui::Color32::WHITE),
                            );
                            ui.painter().text(
                                *pin_pos,
                                egui::Align2::CENTER_CENTER,
                                format!("{}", btn_idx + 1),
                                egui::FontId::proportional(11.0),
                                egui::Color32::WHITE,
                            );
                        } else {
                            ui.painter().circle_filled(
                                *pin_pos,
                                8.5,
                                egui::Color32::from_rgb(30, 41, 59),
                            );
                            ui.painter().circle_stroke(
                                *pin_pos,
                                8.5,
                                egui::Stroke::new(1.2_f32, egui::Color32::WHITE),
                            );
                            ui.painter().text(
                                *pin_pos,
                                egui::Align2::CENTER_CENTER,
                                format!("{}", btn_idx + 1),
                                egui::FontId::proportional(10.0),
                                egui::Color32::WHITE,
                            );
                        }

                        // Label underneath Pin 6 indicating it is on the underside
                        if *btn_idx == 5 {
                            let note_pos = egui::pos2(pin_pos.x + 14.0, pin_pos.y);
                            ui.painter().text(
                                note_pos,
                                egui::Align2::LEFT_CENTER,
                                "Bawah Kaki",
                                egui::FontId::proportional(10.0),
                                if is_hl {
                                    egui::Color32::from_rgb(2, 132, 199)
                                } else {
                                    egui::Color32::from_rgb(100, 116, 139)
                                },
                            );
                        }
                    }

                    ui.add_space(4.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("6 Programmable Hardware Buttons (Hover to highlight)")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(148, 163, 184)),
                        );
                    });
                });

                ui.add_space(10.0);

                // Debounce Click Filter Card
                let debounce_frame = egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(226, 232, 240),
                    ))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0));

                debounce_frame.show(ui, |ui| {
                    ui.set_width(382.0);

                    ui.label(
                        egui::RichText::new("Debounce Click Filter")
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(
                            "Anti-shake filter prevents double clicks (0 - 20 ms)",
                        )
                        .size(11.0)
                        .color(egui::Color32::from_rgb(100, 116, 139)),
                    );
                    ui.add_space(8.0);

                    // Slider row with live duration badge
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.config.debounce_ms, 0..=20)
                                .step_by(2.0)
                                .show_value(false),
                        );

                        let (badge_r, _) =
                            ui.allocate_exact_size(egui::vec2(52.0, 20.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(badge_r, 4.0, egui::Color32::from_rgb(224, 242, 254));
                        ui.painter().text(
                            badge_r.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{} ms", self.config.debounce_ms),
                            egui::FontId::proportional(11.5),
                            egui::Color32::from_rgb(2, 132, 199),
                        );
                    });

                    ui.add_space(6.0);

                    // Quick preset buttons
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        let presets = [
                            (2, "2ms Esports"),
                            (4, "4ms Fast"),
                            (8, "8ms Balanced"),
                            (12, "12ms Safe"),
                        ];
                        for (ms, name) in presets {
                            let is_cur = self.config.debounce_ms == ms;
                            let p_btn = egui::Button::new(
                                egui::RichText::new(name).size(10.5).color(if is_cur {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                }),
                            )
                            .fill(if is_cur {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(241, 245, 249)
                            })
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                if is_cur {
                                    egui::Color32::from_rgb(2, 132, 199)
                                } else {
                                    egui::Color32::from_rgb(203, 213, 225)
                                },
                            ))
                            .rounding(egui::Rounding::same(4.0));

                            if ui.add(p_btn).clicked() {
                                self.config.debounce_ms = ms;
                            }
                        }
                    });
                });
            });

            // ==========================================
            // RIGHT COLUMN: 6 Sequentially Numbered Cards
            // ==========================================
            ui.vertical(|ui| {
                ui.set_width(450.0);

                ui.heading(
                    egui::RichText::new("Button Configuration")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(2, 132, 199)),
                );
                ui.label(
                    egui::RichText::new("Configure action bindings for all 6 mouse buttons (1 to 6):")
                        .size(11.5)
                        .color(egui::Color32::from_rgb(100, 116, 139)),
                );
                ui.add_space(8.0);

                for (title, subnote, btn_idx) in button_definitions {
                    let is_card_hovered = self.hovered_button == Some(btn_idx);
                    let current_action = self
                        .config
                        .buttons
                        .get(btn_idx)
                        .cloned()
                        .unwrap_or(ButtonAction::Disabled);
                    let (cat_label, cat_color) = action_badge(&current_action);

                    let card_frame = egui::Frame::none()
                        .fill(if is_card_hovered {
                            egui::Color32::from_rgb(240, 249, 255)
                        } else {
                            egui::Color32::WHITE
                        })
                        .stroke(egui::Stroke::new(
                            if is_card_hovered { 1.5_f32 } else { 1.0_f32 },
                            if is_card_hovered {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(226, 232, 240)
                            },
                        ))
                        .rounding(egui::Rounding::same(7.0))
                        .inner_margin(egui::Margin::symmetric(10.0, 7.0));

                    let card_resp = card_frame.show(ui, |ui| {
                        ui.set_width(428.0);

                        // Row 1: Number Pill + Title + Subnote + Category Badge
                        ui.horizontal(|ui| {
                            let (p_rect, _) =
                                ui.allocate_exact_size(egui::vec2(22.0, 18.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                p_rect,
                                3.5,
                                if is_card_hovered {
                                    egui::Color32::from_rgb(2, 132, 199)
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                },
                            );
                            ui.painter().text(
                                p_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{}", btn_idx + 1),
                                egui::FontId::proportional(11.0),
                                egui::Color32::WHITE,
                            );

                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(title)
                                    .size(12.5)
                                    .strong()
                                    .color(egui::Color32::from_rgb(15, 23, 42)),
                            );

                            ui.label(
                                egui::RichText::new(format!("• {}", subnote))
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(148, 163, 184)),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (b_rect, _) =
                                    ui.allocate_exact_size(egui::vec2(46.0, 16.0), egui::Sense::hover());
                                ui.painter().rect_filled(
                                    b_rect,
                                    3.0,
                                    egui::Color32::from_rgba_unmultiplied(
                                        cat_color.r(),
                                        cat_color.g(),
                                        cat_color.b(),
                                        35,
                                    ),
                                );
                                ui.painter()
                                    .rect_stroke(b_rect, 3.0, egui::Stroke::new(1.0_f32, cat_color));
                                ui.painter().text(
                                    b_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    cat_label,
                                    egui::FontId::proportional(9.5),
                                    cat_color,
                                );
                            });
                        });

                        ui.add_space(4.0);

                        // Row 2: Action Selection ComboBox
                        let current_display = match &current_action {
                            ButtonAction::MouseButton(1) => "Left Click".to_string(),
                            ButtonAction::MouseButton(2) => "Right Click".to_string(),
                            ButtonAction::MouseButton(4) => "Middle Click".to_string(),
                            ButtonAction::MouseButton(8) => "Backward (Side 1)".to_string(),
                            ButtonAction::MouseButton(16) => "Forward (Side 2)".to_string(),
                            ButtonAction::DpiSwitch => "DPI Switch / Cycle".to_string(),
                            ButtonAction::FireKey { clicks, delay_ms } => {
                                format!("Fire Key ({}x, {}ms)", clicks, delay_ms)
                            }
                            ButtonAction::Media(code) => match code {
                                0xCD => "Play / Pause".to_string(),
                                0xE9 => "Volume Up".to_string(),
                                0xEA => "Volume Down".to_string(),
                                0xE2 => "Mute".to_string(),
                                0xB5 => "Next Track".to_string(),
                                0xB6 => "Prev Track".to_string(),
                                _ => format!("Media (0x{:02X})", code),
                            },
                            ButtonAction::Keyboard { usage, modifier } => {
                                let mod_str = match modifier {
                                    0x01 => "Ctrl+",
                                    0x02 => "Shift+",
                                    0x04 => "Alt+",
                                    0x08 => "Win+",
                                    _ => "",
                                };
                                let key_str = match usage {
                                    0x04 => "A",
                                    0x06 => "C",
                                    0x19 => "V",
                                    0x1B => "X",
                                    0x1D => "Z",
                                    _ => "Key",
                                };
                                format!("{}{}", mod_str, key_str)
                            }
                            ButtonAction::Macro(id) => format!("Macro #{}", id),
                            ButtonAction::Disabled => "Disabled".to_string(),
                            _ => "Custom Action".to_string(),
                        };

                        let mut action_category = match current_action {
                            ButtonAction::MouseButton(1) => 0,
                            ButtonAction::MouseButton(2) => 1,
                            ButtonAction::MouseButton(4) => 2,
                            ButtonAction::MouseButton(8) => 3,
                            ButtonAction::MouseButton(16) => 4,
                            ButtonAction::DpiSwitch => 5,
                            ButtonAction::FireKey { .. } => 6,
                            ButtonAction::Media(_) => 7,
                            ButtonAction::Keyboard { .. } => 8,
                            ButtonAction::Macro(_) => 9,
                            ButtonAction::Disabled => 10,
                            _ => 0,
                        };

                        let options = [
                            "Left Click",
                            "Right Click",
                            "Middle Click",
                            "Backward (Side 1)",
                            "Forward (Side 2)",
                            "DPI Switch / Cycle",
                            "Fire Key (Rapid Fire)",
                            "Media (Play / Pause)",
                            "Keyboard Key (Ctrl+A)",
                            "Macro Playback",
                            "Disabled",
                        ];

                        let prev_category = action_category;
                        egui::ComboBox::from_id_salt(format!("btn_combo_{}", btn_idx))
                            .selected_text(&current_display)
                            .width(300.0)
                            .show_ui(ui, |ui| {
                                for (opt_idx, opt_name) in options.iter().enumerate() {
                                    ui.selectable_value(&mut action_category, opt_idx, *opt_name);
                                }
                            });

                        if action_category != prev_category {
                            let new_action = match action_category {
                                0 => ButtonAction::MouseButton(1),
                                1 => ButtonAction::MouseButton(2),
                                2 => ButtonAction::MouseButton(4),
                                3 => ButtonAction::MouseButton(8),
                                4 => ButtonAction::MouseButton(16),
                                5 => ButtonAction::DpiSwitch,
                                6 => ButtonAction::FireKey {
                                    clicks: 3,
                                    delay_ms: 12,
                                },
                                7 => ButtonAction::Media(0xCD), // Play/Pause
                                8 => ButtonAction::Keyboard {
                                    usage: 0x04,
                                    modifier: 0x01,
                                }, // Ctrl+A
                                9 => ButtonAction::Macro(1),
                                10 => ButtonAction::Disabled,
                                _ => ButtonAction::MouseButton(1),
                            };
                            if btn_idx < self.config.buttons.len() {
                                self.config.buttons[btn_idx] = new_action;
                            }
                        }

                        // If Fire Key is active, show quick burst sliders
                        if let Some(ButtonAction::FireKey { clicks, delay_ms }) =
                            self.config.buttons.get_mut(btn_idx)
                        {
                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Clicks:").size(10.5));
                                ui.add(egui::Slider::new(clicks, 2..=15));
                                ui.label(egui::RichText::new("Delay:").size(10.5));
                                ui.add(egui::Slider::new(delay_ms, 5..=50).suffix("ms"));
                            });
                        }
                    });

                    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                    let rect = card_resp.response.rect;
                    if card_resp.response.hovered() || pointer_pos.is_some_and(|p| rect.contains(p)) {
                        next_hovered = Some(btn_idx);
                    }

                    ui.add_space(6.0);
                }
            });
        });

        self.hovered_button = next_hovered;
    }

    /// Render Tab 2: Sensor, Polling Rate, 2K/4K Modes, Continuous DPI Visualizer and Stages
    fn render_sensor_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Polling Rate 3x2 Grid, LOD & Advanced Features with Native Toggles
            ui.vertical(|ui| {
                ui.set_width(340.0);

                // Polling Rate Card: Modern 3x2 Latency Grid
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Polling Rate")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    ui.label("USB & wireless report frequency:");
                    ui.add_space(8.0);

                    let rate_cards = [
                        (PollingRate::Hz125, "125 Hz", "8.0 ms"),
                        (PollingRate::Hz250, "250 Hz", "4.0 ms"),
                        (PollingRate::Hz500, "500 Hz", "2.0 ms"),
                        (PollingRate::Hz1000, "1000 Hz", "1.0 ms"),
                        (PollingRate::Hz2000, "2000 Hz", "0.5 ms"),
                        (PollingRate::Hz4000, "4000 Hz", "0.25 ms"),
                    ];

                    egui::Grid::new("polling_grid")
                        .spacing(egui::vec2(6.0, 6.0))
                        .show(ui, |ui| {
                            for (i, (rate, label, lat)) in rate_cards.iter().enumerate() {
                                let is_active = self.config.polling_rate == *rate;
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(98.0, 48.0),
                                    egui::Sense::click(),
                                );

                                if resp.clicked() {
                                    self.config.polling_rate = *rate;
                                }

                                ui.painter().rect_filled(
                                    rect,
                                    6.0,
                                    if is_active {
                                        egui::Color32::from_rgb(2, 132, 199)
                                    } else {
                                        egui::Color32::from_rgb(241, 245, 249)
                                    },
                                );
                                ui.painter().rect_stroke(
                                    rect,
                                    6.0,
                                    egui::Stroke::new(
                                        if is_active { 1.5_f32 } else { 1.0_f32 },
                                        if is_active {
                                            egui::Color32::from_rgb(2, 132, 199)
                                        } else {
                                            egui::Color32::from_rgb(226, 232, 240)
                                        },
                                    ),
                                );

                                ui.painter().text(
                                    egui::pos2(rect.center().x, rect.top() + 14.0),
                                    egui::Align2::CENTER_CENTER,
                                    *label,
                                    egui::FontId::proportional(13.0),
                                    if is_active {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::from_rgb(15, 23, 42)
                                    },
                                );

                                ui.painter().text(
                                    egui::pos2(rect.center().x, rect.top() + 32.0),
                                    egui::Align2::CENTER_CENTER,
                                    *lat,
                                    egui::FontId::proportional(11.0),
                                    if is_active {
                                        egui::Color32::from_rgb(224, 242, 254)
                                    } else {
                                        egui::Color32::from_rgb(100, 116, 139)
                                    },
                                );

                                if (i + 1) % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });

                ui.add_space(10.0);

                // Lift-Off Distance (LOD) Dual Pill Selector
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Lift-Off Distance (LOD)")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Sensor tracking cut-off height:");
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let low_selected = self.config.lod == LiftOffDistance::Low;
                        let low_btn = egui::Button::new(
                            egui::RichText::new("1 mm (Low - Esports)").strong().color(
                                if low_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                },
                            ),
                        )
                        .fill(if low_selected {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(if low_selected {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240))
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(150.0, 32.0));

                        if ui.add(low_btn).clicked() {
                            self.config.lod = LiftOffDistance::Low;
                        }

                        let high_selected = self.config.lod == LiftOffDistance::High;
                        let high_btn = egui::Button::new(
                            egui::RichText::new("2 mm (High - Standard)")
                                .strong()
                                .color(if high_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                }),
                        )
                        .fill(if high_selected {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(if high_selected {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240))
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(150.0, 32.0));

                        if ui.add(high_btn).clicked() {
                            self.config.lod = LiftOffDistance::High;
                        }
                    });
                });

                ui.add_space(10.0);

                // Power & Sleep Management with Native Vector Toggle Switch
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Power & Sleep Management")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.add_space(6.0);

                    // Performance mode native toggle
                    ui.horizontal(|ui| {
                        toggle_switch(ui, &mut self.config.performance_mode);
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(if self.config.performance_mode {
                                "Performance Mode (High Responsiveness)"
                            } else {
                                "Office Mode (Power-Saving / Extended Battery)"
                            })
                            .strong()
                            .color(egui::Color32::from_rgb(15, 23, 42)),
                        );
                    });

                    ui.add_space(8.0);

                    // Sensor idle sleep timer dropdown
                    ui.horizontal(|ui| {
                        ui.label("Sensor Sleep Timer:");
                        let sleep_options = [
                            (10, "10 seconds"),
                            (30, "30 seconds"),
                            (60, "1 minute"),
                            (120, "2 minutes"),
                            (180, "3 minutes"),
                            (300, "5 minutes"),
                            (600, "10 minutes"),
                            (0, "Never"),
                        ];
                        let selected_label = sleep_options
                            .iter()
                            .find(|(s, _)| *s == self.config.sensor_sleep_seconds)
                            .map(|(_, l)| *l)
                            .unwrap_or("Custom");

                        egui::ComboBox::from_id_salt("sensor_sleep_combo")
                            .selected_text(selected_label)
                            .width(130.0)
                            .show_ui(ui, |ui| {
                                for (val, label) in sleep_options {
                                    ui.selectable_value(
                                        &mut self.config.sensor_sleep_seconds,
                                        val,
                                        label,
                                    );
                                }
                            });
                    });
                });

                ui.add_space(10.0);

                // Advanced Sensor Toggles (Motion Sync, Angle Snapping, Ripple Control)
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Sensor Features")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.add_space(6.0);

                    // Motion Sync
                    ui.horizontal(|ui| {
                        toggle_switch(ui, &mut self.config.motion_sync);
                        ui.add_space(4.0);
                        ui.label("Motion Sync (USB Clock Synchronization)");
                    });

                    ui.add_space(6.0);

                    // Angle Snapping
                    ui.horizontal(|ui| {
                        toggle_switch(ui, &mut self.config.angle_snapping);
                        ui.add_space(4.0);
                        ui.label("Angle Snapping (Straight Line Correction)");
                    });

                    ui.add_space(6.0);

                    // Ripple Control
                    ui.horizontal(|ui| {
                        toggle_switch(ui, &mut self.config.ripple_control);
                        ui.add_space(4.0);
                        ui.label("Ripple Control (High DPI Smoothing)");
                    });
                });
            });

            ui.add_space(18.0);

            // Right Column: Continuous DPI Spectrum Bar and PAW3395 Stages
            ui.vertical(|ui| {
                ui.set_width(520.0);
                ui.heading(
                    egui::RichText::new("DPI Stages (PixArt PAW3395)")
                        .color(egui::Color32::from_rgb(2, 132, 199)),
                );
                ui.label("Configure up to 5 DPI stages with continuous logarithmic spectrum:");
                ui.add_space(6.0);

                let default_colors = [
                    egui::Color32::from_rgb(255, 60, 60),   // Stage 1: Red
                    egui::Color32::from_rgb(0, 230, 230),   // Stage 2: Cyan
                    egui::Color32::from_rgb(60, 240, 80),   // Stage 3: Green
                    egui::Color32::from_rgb(255, 255, 255), // Stage 4: White
                    egui::Color32::from_rgb(255, 230, 40),  // Stage 5: Yellow
                ];

                // Continuous Horizontal DPI Spectrum Visualizer Bar
                let (spec_rect, _spec_resp) =
                    ui.allocate_exact_size(egui::vec2(510.0, 48.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(spec_rect, 6.0, egui::Color32::from_rgb(241, 245, 249));
                ui.painter().rect_stroke(
                    spec_rect,
                    6.0,
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240)),
                );

                let rail_y = spec_rect.top() + 24.0;
                let rail_left = spec_rect.left() + 28.0;
                let rail_right = spec_rect.right() - 28.0;
                let rail_width = rail_right - rail_left;

                // Continuous rail track
                ui.painter().line_segment(
                    [
                        egui::pos2(rail_left, rail_y),
                        egui::pos2(rail_right, rail_y),
                    ],
                    egui::Stroke::new(4.0_f32, egui::Color32::from_rgb(203, 213, 225)),
                );

                // Helper to map DPI to x position logarithmically
                let dpi_to_x = |dpi: u16| -> f32 {
                    let min_ln = (50.0_f32).ln();
                    let max_ln = (26000.0_f32).ln();
                    let cur_ln = (dpi.clamp(50, 26000) as f32).ln();
                    let t = (cur_ln - min_ln) / (max_ln - min_ln);
                    rail_left + rail_width * t.clamp(0.0, 1.0)
                };

                // Ticks for key milestones
                let milestones = [
                    (400, "400"),
                    (800, "800"),
                    (1600, "1.6k"),
                    (3200, "3.2k"),
                    (6400, "6.4k"),
                    (12000, "12k"),
                    (26000, "26k"),
                ];
                for (d, label) in milestones {
                    let x = dpi_to_x(d);
                    ui.painter().line_segment(
                        [egui::pos2(x, rail_y - 3.0), egui::pos2(x, rail_y + 3.0)],
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.painter().text(
                        egui::pos2(x, rail_y + 12.0),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(9.0),
                        egui::Color32::from_rgb(100, 116, 139),
                    );
                }

                // Paint Stage markers
                for (s_idx, s) in self.config.dpi_stages.iter().enumerate() {
                    if !s.enabled {
                        continue;
                    }
                    let sx = dpi_to_x(s.dpi);
                    let pt = egui::pos2(sx, rail_y);
                    let scol = default_colors
                        .get(s_idx)
                        .copied()
                        .unwrap_or(egui::Color32::WHITE);
                    let is_active = self.config.active_dpi_stage == s_idx as u8;

                    if is_active {
                        // Halo ring
                        ui.painter().circle_filled(
                            pt,
                            10.0,
                            egui::Color32::from_rgba_unmultiplied(2, 132, 199, 90),
                        );
                        ui.painter().circle_filled(pt, 6.5, scol);
                        ui.painter().circle_stroke(
                            pt,
                            6.5,
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(15, 23, 42)),
                        );

                        // Stage Callout Tag above the dot
                        let tag_rect = egui::Rect::from_center_size(
                            egui::pos2(sx, rail_y - 14.0),
                            egui::vec2(44.0, 14.0),
                        );
                        ui.painter().rect_filled(
                            tag_rect,
                            3.0,
                            egui::Color32::from_rgb(2, 132, 199),
                        );
                        ui.painter().text(
                            tag_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("S{}", s_idx + 1),
                            egui::FontId::proportional(9.5),
                            egui::Color32::WHITE,
                        );
                    } else {
                        ui.painter().circle_filled(pt, 5.0, scol);
                        ui.painter().circle_stroke(
                            pt,
                            5.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(71, 85, 105)),
                        );
                    }
                }

                ui.add_space(10.0);

                // 5 DPI Stage Configuration Cards
                for i in 0..self.config.dpi_stages.len() {
                    let is_active_stage = self.config.active_dpi_stage == i as u8;
                    let is_enabled = self.config.dpi_stages[i].enabled;
                    let stage_color = default_colors
                        .get(i)
                        .copied()
                        .unwrap_or(egui::Color32::WHITE);

                    let frame = egui::Frame::none()
                        .fill(if is_active_stage {
                            egui::Color32::from_rgb(240, 249, 255)
                        } else if !is_enabled {
                            egui::Color32::from_rgb(250, 250, 250)
                        } else {
                            egui::Color32::from_rgb(255, 255, 255)
                        })
                        .stroke(egui::Stroke::new(
                            if is_active_stage { 1.5_f32 } else { 1.0_f32 },
                            if is_active_stage {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(226, 232, 240)
                            },
                        ))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0));

                    frame.show(ui, |ui| {
                        // Row 1: Radio, Color Indicator, Title, ACTIVE badge, Enable Checkbox
                        ui.horizontal(|ui| {
                            if ui.radio(is_active_stage, "").clicked() {
                                self.config.active_dpi_stage = i as u8;
                                self.config.dpi_stages[i].enabled = true;
                            }

                            // Distinct circular color chip with dark outline
                            let (chip_rect, _) = ui
                                .allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                            ui.painter()
                                .circle_filled(chip_rect.center(), 7.0, stage_color);
                            ui.painter().circle_stroke(
                                chip_rect.center(),
                                7.0,
                                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(148, 163, 184)),
                            );

                            ui.add_space(4.0);

                            ui.label(
                                egui::RichText::new(format!("Stage {}", i + 1))
                                    .size(13.5)
                                    .color(if is_active_stage {
                                        egui::Color32::from_rgb(2, 132, 199)
                                    } else if !is_enabled {
                                        egui::Color32::from_rgb(148, 163, 184)
                                    } else {
                                        egui::Color32::from_rgb(15, 23, 42)
                                    })
                                    .strong(),
                            );

                            if is_active_stage {
                                let (badge_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(54.0, 18.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    badge_rect,
                                    4.0,
                                    egui::Color32::from_rgb(2, 132, 199),
                                );
                                ui.painter().text(
                                    badge_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "ACTIVE",
                                    egui::FontId::proportional(10.5),
                                    egui::Color32::WHITE,
                                );
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .checkbox(&mut self.config.dpi_stages[i].enabled, "Enable")
                                        .changed()
                                    {
                                        let enabled_count = self
                                            .config
                                            .dpi_stages
                                            .iter()
                                            .filter(|s| s.enabled)
                                            .count();
                                        if enabled_count == 0 {
                                            self.config.dpi_stages[i].enabled = true;
                                        } else if !self.config.dpi_stages[i].enabled
                                            && self.config.active_dpi_stage == i as u8
                                            && let Some(first) = self
                                                .config
                                                .dpi_stages
                                                .iter()
                                                .position(|s| s.enabled)
                                        {
                                            self.config.active_dpi_stage = first as u8;
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(6.0);

                        // Row 2: Stepping buttons, Wide Slider with trailing progress track, and Crisp Value Badge
                        ui.horizontal(|ui| {
                            let minus_btn = egui::Button::new(
                                egui::RichText::new("-50")
                                    .size(11.5)
                                    .color(egui::Color32::from_rgb(15, 23, 42)),
                            )
                            .min_size(egui::vec2(38.0, 24.0))
                            .fill(egui::Color32::from_rgb(241, 245, 249))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgb(203, 213, 225),
                            ))
                            .rounding(egui::Rounding::same(4.0));

                            if ui
                                .add_enabled(is_enabled, minus_btn)
                                .on_hover_text("Decrease by 50 DPI")
                                .clicked()
                            {
                                let cur = self.config.dpi_stages[i].dpi;
                                self.config.dpi_stages[i].dpi = cur.saturating_sub(50).max(50);
                            }

                            let mut dpi_val = self.config.dpi_stages[i].dpi as i32;
                            let slider = egui::Slider::new(&mut dpi_val, 50..=26000)
                                .step_by(50.0)
                                .show_value(false)
                                .trailing_fill(true)
                                .logarithmic(true);

                            let slider_resp = ui.add_sized(egui::vec2(250.0, 24.0), slider);
                            if is_enabled && slider_resp.changed() {
                                let snapped = crate::dpi::snap_dpi(dpi_val.clamp(50, 26000) as u16);
                                self.config.dpi_stages[i].dpi = snapped;
                            }

                            let plus_btn = egui::Button::new(
                                egui::RichText::new("+50")
                                    .size(11.5)
                                    .color(egui::Color32::from_rgb(15, 23, 42)),
                            )
                            .min_size(egui::vec2(38.0, 24.0))
                            .fill(egui::Color32::from_rgb(241, 245, 249))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgb(203, 213, 225),
                            ))
                            .rounding(egui::Rounding::same(4.0));

                            if ui
                                .add_enabled(is_enabled, plus_btn)
                                .on_hover_text("Increase by 50 DPI")
                                .clicked()
                            {
                                let cur = self.config.dpi_stages[i].dpi;
                                self.config.dpi_stages[i].dpi = cur.saturating_add(50).min(26000);
                            }

                            ui.add_space(8.0);

                            // Dedicated Crisp DPI Value Box
                            let (val_rect, _) = ui
                                .allocate_exact_size(egui::vec2(92.0, 24.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                val_rect,
                                4.0,
                                if is_active_stage {
                                    egui::Color32::from_rgb(224, 242, 254)
                                } else {
                                    egui::Color32::from_rgb(248, 250, 252)
                                },
                            );
                            ui.painter().rect_stroke(
                                val_rect,
                                4.0,
                                egui::Stroke::new(
                                    1.0_f32,
                                    if is_active_stage {
                                        egui::Color32::from_rgb(186, 230, 253)
                                    } else {
                                        egui::Color32::from_rgb(226, 232, 240)
                                    },
                                ),
                            );
                            ui.painter().text(
                                val_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{} DPI", self.config.dpi_stages[i].dpi),
                                egui::FontId::proportional(12.5),
                                if is_active_stage {
                                    egui::Color32::from_rgb(2, 132, 199)
                                } else if !is_enabled {
                                    egui::Color32::from_rgb(148, 163, 184)
                                } else {
                                    egui::Color32::from_rgb(15, 23, 42)
                                },
                            );
                        });
                    });
                    ui.add_space(6.0);
                }
            });
        });
    }

    /// Render Tab 3: Lighting, Live Animated LED Illumination Preview, 6 Esports Presets & 14 Color Chips
    fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        // Request repaint if animated mode is running
        if self.config.dpi_led.mode == LedMode::Breathing
            || self.config.dpi_led.mode == LedMode::Neon
        {
            ui.ctx().request_repaint();
        }

        let elapsed = self.start_time.elapsed().as_secs_f32();
        let live_preview_color = match self.config.dpi_led.mode {
            LedMode::Solid => egui::Color32::from_rgb(
                self.config.dpi_led.color.r,
                self.config.dpi_led.color.g,
                self.config.dpi_led.color.b,
            ),
            LedMode::Breathing => {
                let speed_mult = self.config.dpi_led.breath_speed as f32 * 0.35;
                let factor = 0.25 + 0.75 * (0.5 + 0.5 * (elapsed * speed_mult).sin());
                egui::Color32::from_rgb(
                    (self.config.dpi_led.color.r as f32 * factor) as u8,
                    (self.config.dpi_led.color.g as f32 * factor) as u8,
                    (self.config.dpi_led.color.b as f32 * factor) as u8,
                )
            }
            LedMode::Neon => {
                let speed_mult = self.config.dpi_led.breath_speed as f32 * 0.12;
                let hue = (elapsed * speed_mult) % 1.0;
                hsv_to_rgb(hue, 0.9, 1.0)
            }
            LedMode::Off => egui::Color32::from_rgb(71, 85, 105),
        };

        ui.horizontal(|ui| {
            // Left Column: Live Illumination Preview Card, Effect Mode & Timers
            ui.vertical(|ui| {
                ui.set_width(380.0);

                // Live Illumination Preview Glow Card
                ui.group(|ui| {
                    ui.set_width(360.0);
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new("Live Illumination Preview")
                                .color(egui::Color32::from_rgb(2, 132, 199)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (badge_r, _) = ui
                                .allocate_exact_size(egui::vec2(60.0, 20.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                badge_r,
                                4.0,
                                egui::Color32::from_rgb(241, 245, 249),
                            );
                            ui.painter().text(
                                badge_r.center(),
                                egui::Align2::CENTER_CENTER,
                                match self.config.dpi_led.mode {
                                    LedMode::Solid => "SOLID",
                                    LedMode::Breathing => "PULSE",
                                    LedMode::Neon => "NEON",
                                    LedMode::Off => "OFF",
                                },
                                egui::FontId::proportional(11.0),
                                egui::Color32::from_rgb(51, 65, 85),
                            );
                        });
                    });

                    ui.add_space(8.0);

                    // High-tech illuminated glow preview box
                    let (glow_rect, _) =
                        ui.allocate_exact_size(egui::vec2(340.0, 72.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(glow_rect, 8.0, egui::Color32::from_rgb(15, 23, 42));

                    // Soft radial ambient glow
                    let center_pt = glow_rect.center();
                    if self.config.dpi_led.mode != LedMode::Off {
                        ui.painter().circle_filled(
                            center_pt,
                            28.0,
                            egui::Color32::from_rgba_unmultiplied(
                                live_preview_color.r(),
                                live_preview_color.g(),
                                live_preview_color.b(),
                                60,
                            ),
                        );
                        ui.painter().circle_filled(
                            center_pt,
                            18.0,
                            egui::Color32::from_rgba_unmultiplied(
                                live_preview_color.r(),
                                live_preview_color.g(),
                                live_preview_color.b(),
                                120,
                            ),
                        );
                    }

                    // Core LED circle
                    ui.painter()
                        .circle_filled(center_pt, 9.0, live_preview_color);
                    ui.painter().circle_stroke(
                        center_pt,
                        9.0,
                        egui::Stroke::new(1.5_f32, egui::Color32::WHITE),
                    );

                    // Live hex readout text
                    ui.painter().text(
                        egui::pos2(glow_rect.right() - 14.0, glow_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        format!(
                            "#{:02X}{:02X}{:02X}",
                            live_preview_color.r(),
                            live_preview_color.g(),
                            live_preview_color.b()
                        ),
                        egui::FontId::monospace(12.0),
                        egui::Color32::WHITE,
                    );
                });

                ui.add_space(10.0);

                // LED Effect Mode Card
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("LED Effect Mode")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Select DPI LED lighting style:");
                    ui.add_space(8.0);

                    let modes = [
                        (LedMode::Solid, "Solid", "Constant static illumination"),
                        (LedMode::Breathing, "Breathing", "Pulsing glow animation"),
                        (
                            LedMode::Neon,
                            "Neon / Cycle",
                            "Rainbow dynamic color transition",
                        ),
                        (LedMode::Off, "Off", "Disable mouse lighting"),
                    ];

                    for (mode, label, desc) in modes {
                        let is_active = self.config.dpi_led.mode == mode;
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} — {}", label, desc))
                                .size(12.5)
                                .color(if is_active {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                })
                                .strong(),
                        )
                        .fill(if is_active {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(if is_active {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240))
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(340.0, 34.0));

                        if ui.add(btn).clicked() {
                            self.config.dpi_led.mode = mode;
                        }
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Brightness, Speed & Sleep")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.add_space(4.0);

                    ui.label(format!("Brightness: {}", self.config.dpi_led.brightness));
                    ui.add(egui::Slider::new(
                        &mut self.config.dpi_led.brightness,
                        0..=10,
                    ));

                    ui.add_space(6.0);

                    ui.label(format!(
                        "Breathing Speed: {}",
                        self.config.dpi_led.breath_speed
                    ));
                    ui.add(egui::Slider::new(
                        &mut self.config.dpi_led.breath_speed,
                        1..=10,
                    ));

                    ui.add_space(6.0);

                    ui.label("LED Idle Sleep Timer:");
                    let sleep_options = [
                        (10, "10 seconds"),
                        (30, "30 seconds"),
                        (60, "1 minute"),
                        (120, "2 minutes"),
                        (180, "3 minutes"),
                        (300, "5 minutes"),
                        (0, "Never"),
                    ];
                    let selected_label = sleep_options
                        .iter()
                        .find(|(s, _)| *s == self.config.led_sleep_seconds)
                        .map(|(_, l)| *l)
                        .unwrap_or("Custom");

                    egui::ComboBox::from_id_salt("led_sleep_combo")
                        .selected_text(selected_label)
                        .width(130.0)
                        .show_ui(ui, |ui| {
                            for (val, label) in sleep_options {
                                ui.selectable_value(&mut self.config.led_sleep_seconds, val, label);
                            }
                        });
                });
            });

            ui.add_space(18.0);

            // Right Column: 6 Gaming Color Presets, Original ColorBoard, 14 Reverse Engineered Chips & Sliders
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(400.0);
                    ui.heading(
                        egui::RichText::new("LED Color Palette")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    ui.add_space(6.0);
                    ui.label("Esports Color Presets:");
                    ui.add_space(4.0);

                    // 6 Gaming Esports Color Preset Pills
                    let gaming_presets = [
                        ("Icarus Cyan", Color::new(0, 240, 255)),
                        ("Crimson Red", Color::new(255, 42, 42)),
                        ("Toxic Green", Color::new(0, 255, 102)),
                        ("Cyber Purple", Color::new(168, 85, 247)),
                        ("Solar Gold", Color::new(251, 191, 36)),
                        ("Arctic White", Color::new(255, 255, 255)),
                    ];

                    egui::Grid::new("presets_grid")
                        .spacing(egui::vec2(6.0, 6.0))
                        .show(ui, |ui| {
                            for (p_idx, (p_name, p_col)) in gaming_presets.iter().enumerate() {
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(116.0, 28.0),
                                    egui::Sense::click(),
                                );
                                if resp.clicked() {
                                    self.config.dpi_led.color = *p_col;
                                }

                                ui.painter().rect_filled(
                                    rect,
                                    4.0,
                                    egui::Color32::from_rgb(241, 245, 249),
                                );
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(203, 213, 225),
                                    ),
                                );

                                // Mini color dot
                                let dot_pos = egui::pos2(rect.left() + 10.0, rect.center().y);
                                ui.painter().circle_filled(
                                    dot_pos,
                                    5.0,
                                    egui::Color32::from_rgb(p_col.r, p_col.g, p_col.b),
                                );
                                ui.painter().circle_stroke(
                                    dot_pos,
                                    5.0,
                                    egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(148, 163, 184),
                                    ),
                                );

                                ui.painter().text(
                                    egui::pos2(rect.left() + 20.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    *p_name,
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::from_rgb(15, 23, 42),
                                );

                                if (p_idx + 1) % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                        });

                    ui.add_space(10.0);

                    // Original ColorBoard graphic banner
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Light/ColorBoard.png"))
                            .max_width(360.0),
                    );

                    ui.add_space(8.0);
                    ui.label("Reverse-engineered software color chips (Color A – N):");
                    ui.add_space(6.0);

                    // 14 Individual Reverse-Engineered Color Swatches (Row 1: A-G, Row 2: H-N)
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        let swatches_row1 = [
                            (
                                egui::include_image!("../assets/Light/ColorA.png"),
                                Color::new(255, 0, 0),
                                "Color A: Red",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorB.png"),
                                Color::new(0, 255, 255),
                                "Color B: Cyan",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorC.png"),
                                Color::new(0, 255, 0),
                                "Color C: Green",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorD.png"),
                                Color::new(255, 0, 255),
                                "Color D: Magenta",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorE.png"),
                                Color::new(255, 255, 0),
                                "Color E: Yellow",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorF.png"),
                                Color::new(255, 80, 0),
                                "Color F: Orange",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorG.png"),
                                Color::new(0, 0, 255),
                                "Color G: Blue",
                            ),
                        ];

                        for (img, col, name) in swatches_row1 {
                            let btn = egui::ImageButton::new(
                                egui::Image::new(img).max_width(26.0).max_height(26.0),
                            );
                            if ui.add(btn).on_hover_text(name).clicked() {
                                self.config.dpi_led.color = col;
                            }
                        }
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        let swatches_row2 = [
                            (
                                egui::include_image!("../assets/Light/ColorH.png"),
                                Color::new(227, 0, 127),
                                "Color H: Pink",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorI.png"),
                                Color::new(46, 166, 223),
                                "Color I: Sky",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorJ.png"),
                                Color::new(96, 25, 133),
                                "Color J: Purple",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorK.png"),
                                Color::new(255, 255, 255),
                                "Color K: White",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorL.png"),
                                Color::new(139, 178, 36),
                                "Color L: Lime",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorM.png"),
                                Color::new(255, 155, 51),
                                "Color M: Amber",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorN.png"),
                                Color::new(208, 113, 173),
                                "Color N: Orchid",
                            ),
                        ];

                        for (img, col, name) in swatches_row2 {
                            let btn = egui::ImageButton::new(
                                egui::Image::new(img).max_width(26.0).max_height(26.0),
                            );
                            if ui.add(btn).on_hover_text(name).clicked() {
                                self.config.dpi_led.color = col;
                            }
                        }
                    });

                    ui.add_space(12.0);
                    ui.label("Fine Tuning (Custom RGB Sliders):");

                    // Red slider with Add/Reduce buttons
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("-5").strong()).clicked() {
                            self.config.dpi_led.color.r =
                                self.config.dpi_led.color.r.saturating_sub(5);
                        }

                        ui.label("Red:  ");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.r, 0..=255));

                        if ui.button(egui::RichText::new("+5").strong()).clicked() {
                            self.config.dpi_led.color.r =
                                self.config.dpi_led.color.r.saturating_add(5);
                        }
                    });

                    // Green slider with Add/Reduce buttons
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("-5").strong()).clicked() {
                            self.config.dpi_led.color.g =
                                self.config.dpi_led.color.g.saturating_sub(5);
                        }

                        ui.label("Green:");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.g, 0..=255));

                        if ui.button(egui::RichText::new("+5").strong()).clicked() {
                            self.config.dpi_led.color.g =
                                self.config.dpi_led.color.g.saturating_add(5);
                        }
                    });

                    // Blue slider with Add/Reduce buttons
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("-5").strong()).clicked() {
                            self.config.dpi_led.color.b =
                                self.config.dpi_led.color.b.saturating_sub(5);
                        }

                        ui.label("Blue: ");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.b, 0..=255));

                        if ui.button(egui::RichText::new("+5").strong()).clicked() {
                            self.config.dpi_led.color.b =
                                self.config.dpi_led.color.b.saturating_add(5);
                        }
                    });

                    ui.add_space(8.0);

                    // Dynamic preview chip and hex code
                    ui.horizontal(|ui| {
                        ui.label("Active Color:");
                        let preview_col = egui::Color32::from_rgb(
                            self.config.dpi_led.color.r,
                            self.config.dpi_led.color.g,
                            self.config.dpi_led.color.b,
                        );
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(80.0, 24.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, preview_col);
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225)),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "#{:02X}{:02X}{:02X}",
                                self.config.dpi_led.color.r,
                                self.config.dpi_led.color.g,
                                self.config.dpi_led.color.b
                            ))
                            .monospace()
                            .strong(),
                        );
                    });
                });
            });
        });
    }

    /// Render Tab 4: Macro Manager with visual flowchart sequence cards & delay presets
    fn render_macro_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Macro Library List
            ui.vertical(|ui| {
                ui.set_width(320.0);
                if self.selected_macro_idx >= self.config.macros.len() {
                    self.selected_macro_idx = self.config.macros.len().saturating_sub(1);
                }
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Macro Library")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Stored macro sequences:");
                    ui.add_space(6.0);

                    // Macro sequence items in library
                    let num_macros = self.config.macros.len();
                    for idx in 0..num_macros {
                        let is_selected = self.selected_macro_idx == idx;
                        let macro_name = self
                            .config
                            .macros
                            .get(idx)
                            .map(|m| m.name.clone())
                            .unwrap_or_default();

                        let item_btn = egui::Button::new(
                            egui::RichText::new(&macro_name)
                                .size(13.0)
                                .color(if is_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                })
                                .strong(),
                        )
                        .fill(if is_selected {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(if is_selected {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240))
                        })
                        .min_size(egui::vec2(290.0, 30.0));

                        if ui.add(item_btn).clicked() {
                            self.selected_macro_idx = idx;
                        }
                        ui.add_space(2.0);
                    }

                    ui.add_space(8.0);

                    // New Macro creation
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_macro_name);
                        if ui.button("Add Macro").clicked()
                            && !self.new_macro_name.trim().is_empty()
                        {
                            self.config.macros.push(MacroItem {
                                name: self.new_macro_name.trim().to_string(),
                                repeat_mode: MacroRepeatMode::Times(1),
                                auto_insert_delay: true,
                                default_delay_ms: 15,
                                events: vec![
                                    MacroEvent {
                                        event_type: MacroEventType::KeyDown,
                                        key: "Space".into(),
                                        delay_ms: 0,
                                    },
                                    MacroEvent {
                                        event_type: MacroEventType::Delay,
                                        key: String::new(),
                                        delay_ms: 15,
                                    },
                                    MacroEvent {
                                        event_type: MacroEventType::KeyUp,
                                        key: "Space".into(),
                                        delay_ms: 0,
                                    },
                                ],
                            });
                            self.selected_macro_idx = self.config.macros.len() - 1;
                            self.new_macro_name.clear();
                        }
                    });

                    // Delete Macro
                    if self.config.macros.len() > 1 && ui.button("Delete Selected").clicked() {
                        if self.selected_macro_idx < self.config.macros.len() {
                            self.config.macros.remove(self.selected_macro_idx);
                        }
                        if self.selected_macro_idx >= self.config.macros.len() {
                            self.selected_macro_idx = self.config.macros.len().saturating_sub(1);
                        }
                    }
                });

                ui.add_space(10.0);

                // Macro Repeat Mode
                if let Some(mac) = self.config.macros.get_mut(self.selected_macro_idx) {
                    ui.group(|ui| {
                        ui.heading(
                            egui::RichText::new("Repeat Settings")
                                .color(egui::Color32::from_rgb(2, 132, 199)),
                        );
                        ui.add_space(4.0);

                        let mut mode_code = match mac.repeat_mode {
                            MacroRepeatMode::Times(_) => 0,
                            MacroRepeatMode::WhileHeld => 1,
                            MacroRepeatMode::Toggle => 2,
                        };

                        ui.radio_value(&mut mode_code, 0, "Repeat N Times");
                        if mode_code == 0 {
                            let mut times = match mac.repeat_mode {
                                MacroRepeatMode::Times(n) => n,
                                _ => 1,
                            };
                            ui.horizontal(|ui| {
                                ui.label("Count:");
                                ui.add(egui::Slider::new(&mut times, 1..=50));
                            });
                            mac.repeat_mode = MacroRepeatMode::Times(times);
                        }

                        if ui
                            .radio_value(&mut mode_code, 1, "Repeat While Held Down (ReUntilLoose)")
                            .clicked()
                        {
                            mac.repeat_mode = MacroRepeatMode::WhileHeld;
                        }

                        if ui
                            .radio_value(&mut mode_code, 2, "Toggle on Click (ReUntilClick)")
                            .clicked()
                        {
                            mac.repeat_mode = MacroRepeatMode::Toggle;
                        }

                        ui.add_space(6.0);
                        ui.checkbox(
                            &mut mac.auto_insert_delay,
                            "Auto-insert Delay (AutoInsertDelay)",
                        );
                        if mac.auto_insert_delay {
                            ui.horizontal(|ui| {
                                ui.label("Default Delay:");
                                ui.add(
                                    egui::Slider::new(&mut mac.default_delay_ms, 5..=100)
                                        .suffix(" ms"),
                                );
                            });
                        }
                    });
                }
            });

            ui.add_space(18.0);

            // Right Column: Flowchart-style visual timeline sequence cards
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(420.0);

                    // Top Action Bar with clean Record, Stop and Quick Delay Presets
                    ui.horizontal(|ui| {
                        let rec_btn = egui::Button::new(
                            egui::RichText::new("Record")
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(220, 38, 38))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(80.0, 28.0));

                        if ui
                            .add(rec_btn)
                            .on_hover_text("Start recording keystrokes")
                            .clicked()
                        {
                            self.is_recording_macro = true;
                            self.set_status("[REC] Recording macro... press Stop when done", true);
                        }

                        let stop_btn = egui::Button::new(
                            egui::RichText::new("Stop")
                                .color(egui::Color32::from_rgb(15, 23, 42))
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(241, 245, 249))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(203, 213, 225),
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(70.0, 28.0));

                        if ui
                            .add(stop_btn)
                            .on_hover_text("Stop recording keystrokes")
                            .clicked()
                        {
                            self.is_recording_macro = false;
                            self.set_status("Stopped macro recording", true);
                        }

                        if self.is_recording_macro {
                            ui.label(
                                egui::RichText::new("[REC]")
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .strong(),
                            );
                        }
                    });

                    ui.add_space(8.0);
                    ui.heading(
                        egui::RichText::new("Timeline Sequence")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    let mut event_to_remove = None;
                    let mut move_up = None;
                    let mut move_down = None;

                    if let Some(mac) = self.config.macros.get_mut(self.selected_macro_idx) {
                        let num_events = mac.events.len();
                        egui::ScrollArea::vertical()
                            .max_height(240.0)
                            .show(ui, |ui| {
                                for (e_idx, event) in mac.events.iter_mut().enumerate() {
                                    let item_frame = egui::Frame::none()
                                        .fill(egui::Color32::from_rgb(255, 255, 255))
                                        .stroke(egui::Stroke::new(
                                            1.0_f32,
                                            egui::Color32::from_rgb(226, 232, 240),
                                        ))
                                        .rounding(egui::Rounding::same(6.0))
                                        .inner_margin(egui::Margin::symmetric(10.0, 6.0));

                                    item_frame.show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Flowchart Node Badge with original icons
                                            match event.event_type {
                                                MacroEventType::KeyDown => {
                                                    ui.add(
                                                        egui::Image::new(egui::include_image!(
                                                            "../assets/Macro/MacroDown.png"
                                                        ))
                                                        .max_height(16.0),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "Key Down: {}",
                                                            event.key
                                                        ))
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(
                                                            2, 132, 199,
                                                        )),
                                                    );
                                                }
                                                MacroEventType::KeyUp => {
                                                    ui.add(
                                                        egui::Image::new(egui::include_image!(
                                                            "../assets/Macro/MacroUp.png"
                                                        ))
                                                        .max_height(16.0),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "Key Up:   {}",
                                                            event.key
                                                        ))
                                                        .color(egui::Color32::from_rgb(
                                                            100, 116, 139,
                                                        )),
                                                    );
                                                }
                                                MacroEventType::Delay => {
                                                    ui.add(
                                                        egui::Image::new(egui::include_image!(
                                                            "../assets/Macro/MacroTime.png"
                                                        ))
                                                        .max_height(16.0),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new("Delay:")
                                                            .color(egui::Color32::from_rgb(
                                                                180, 83, 9,
                                                            ))
                                                            .strong(),
                                                    );
                                                    ui.add(
                                                        egui::DragValue::new(&mut event.delay_ms)
                                                            .suffix(" ms"),
                                                    );
                                                }
                                            }

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .small_button("Delete")
                                                        .on_hover_text("Delete event")
                                                        .clicked()
                                                    {
                                                        event_to_remove = Some(e_idx);
                                                    }

                                                    if ui
                                                        .small_button("Down")
                                                        .on_hover_text("Move down")
                                                        .clicked()
                                                        && e_idx + 1 < num_events
                                                    {
                                                        move_down = Some(e_idx);
                                                    }

                                                    if ui
                                                        .small_button("Up")
                                                        .on_hover_text("Move up")
                                                        .clicked()
                                                        && e_idx > 0
                                                    {
                                                        move_up = Some(e_idx);
                                                    }
                                                },
                                            );
                                        });
                                    });
                                    ui.add_space(3.0);
                                }
                            });

                        // Reorder / remove handlers
                        if let Some(idx) = event_to_remove {
                            mac.events.remove(idx);
                        }
                        if let Some(idx) = move_up {
                            mac.events.swap(idx, idx - 1);
                        }
                        if let Some(idx) = move_down {
                            mac.events.swap(idx, idx + 1);
                        }

                        ui.add_space(8.0);

                        // Manual event insertion tools
                        ui.horizontal(|ui| {
                            ui.label("Insert Key:");
                            ui.text_edit_singleline(&mut self.new_key_input);
                            if ui.button("Down").clicked() {
                                mac.events.push(MacroEvent {
                                    event_type: MacroEventType::KeyDown,
                                    key: self.new_key_input.clone(),
                                    delay_ms: 0,
                                });
                            }
                            if ui.button("Up").clicked() {
                                mac.events.push(MacroEvent {
                                    event_type: MacroEventType::KeyUp,
                                    key: self.new_key_input.clone(),
                                    delay_ms: 0,
                                });
                            }
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Quick Delay:");
                            let delay_presets = [10, 25, 50, 100];
                            for d in delay_presets {
                                if ui.button(format!("+{}ms", d)).clicked() {
                                    mac.events.push(MacroEvent {
                                        event_type: MacroEventType::Delay,
                                        key: String::new(),
                                        delay_ms: d,
                                    });
                                }
                            }
                        });

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.label("Custom Delay:");
                            ui.add(egui::DragValue::new(&mut self.new_delay_ms).suffix(" ms"));
                            if ui.button("Add Delay").clicked() {
                                mac.events.push(MacroEvent {
                                    event_type: MacroEventType::Delay,
                                    key: String::new(),
                                    delay_ms: self.new_delay_ms,
                                });
                            }
                        });
                    }
                });
            });
        });
    }

    /// Render Tab 5: Settings, 2.4G Pairing Tool, Profile Import/Export, and Linux Udev Rules
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: 2.4G Pairing Tool & Profile Management
            ui.vertical(|ui| {
                ui.set_width(380.0);

                // 2.4G Wireless Pairing Tool Card
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("2.4G Wireless Pairing Tool")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    ui.label("Pair your ICARUS V2 MINI mouse to its 2.4G/4G wireless dongle:");
                    ui.add_space(8.0);

                    // Pairing status indicator pill
                    match self.pairing_state {
                        PairingState::Idle => {
                            ui.horizontal(|ui| {
                                let (badge_r, _) = ui.allocate_exact_size(
                                    egui::vec2(64.0, 22.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    badge_r,
                                    4.0,
                                    egui::Color32::from_rgb(224, 242, 254),
                                );
                                ui.painter().text(
                                    badge_r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "READY",
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::from_rgb(2, 132, 199),
                                );
                                ui.label("Dongle is waiting for pairing command");
                            });
                        }
                        PairingState::Pairing => {
                            ui.horizontal(|ui| {
                                let (badge_r, _) = ui.allocate_exact_size(
                                    egui::vec2(76.0, 22.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    badge_r,
                                    4.0,
                                    egui::Color32::from_rgb(254, 243, 199),
                                );
                                ui.painter().text(
                                    badge_r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "PAIRING...",
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::from_rgb(180, 83, 9),
                                );
                                ui.label("Keep mouse close to dongle");
                            });
                        }
                        PairingState::Success => {
                            ui.horizontal(|ui| {
                                let (badge_r, _) = ui.allocate_exact_size(
                                    egui::vec2(64.0, 22.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    badge_r,
                                    4.0,
                                    egui::Color32::from_rgb(240, 253, 244),
                                );
                                ui.painter().text(
                                    badge_r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "PAIRED",
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::from_rgb(22, 163, 74),
                                );
                                ui.label("Dongle Paired Successfully!");
                            });
                        }
                        PairingState::Failed => {
                            ui.horizontal(|ui| {
                                let (badge_r, _) = ui.allocate_exact_size(
                                    egui::vec2(64.0, 22.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    badge_r,
                                    4.0,
                                    egui::Color32::from_rgb(254, 242, 242),
                                );
                                ui.painter().text(
                                    badge_r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "FAILED",
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::from_rgb(220, 38, 38),
                                );
                                ui.label("Pairing Failed. Try again.");
                            });
                        }
                    }

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Start 2.4G Pairing").clicked() {
                            self.pairing_state = PairingState::Pairing;
                            self.pairing_start = Some(Instant::now());
                            self.set_status("Scanning for wireless dongle...", true);
                        }

                        if self.pairing_state != PairingState::Idle
                            && ui.button("Reset State").clicked()
                        {
                            self.pairing_state = PairingState::Idle;
                        }
                    });
                });

                ui.add_space(10.0);

                // Profile Management
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Profile Import / Export")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Save or load custom mouse configuration files:");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Export Profile (TOML)").clicked() {
                            match toml::to_string_pretty(&self.config) {
                                Ok(toml_str) => {
                                    let file_path = "icarus-profile.toml";
                                    if std::fs::write(file_path, toml_str).is_ok() {
                                        self.set_status(
                                            format!("Saved profile to {}", file_path),
                                            true,
                                        );
                                    } else {
                                        self.set_status("Failed to write profile file", false);
                                    }
                                }
                                Err(e) => self.set_status(
                                    format!("Failed to serialize profile: {}", e),
                                    false,
                                ),
                            }
                        }

                        if ui.button("Import Profile (TOML)").clicked() {
                            let file_path = "icarus-profile.toml";
                            match std::fs::read_to_string(file_path) {
                                Ok(content) => match toml::from_str::<MouseConfig>(&content) {
                                    Ok(mut cfg) => {
                                        cfg.validate();
                                        self.config = cfg;
                                        if self.selected_macro_idx >= self.config.macros.len() {
                                            self.selected_macro_idx = 0;
                                        }
                                        self.set_status(
                                            format!("Loaded profile from {}", file_path),
                                            true,
                                        );
                                    }
                                    Err(e) => self.set_status(format!("Parse error: {}", e), false),
                                },
                                Err(_) => {
                                    self.set_status("File 'icarus-profile.toml' not found", false)
                                }
                            }
                        }
                    });
                });

                ui.add_space(10.0);

                // Udev Permissions Guide
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("USB Permissions (Udev)")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("To communicate with the mouse without root/sudo:");
                    ui.add_space(4.0);

                    ui.code("sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/");
                    ui.code("sudo udevadm control --reload-rules && sudo udevadm trigger");

                    ui.add_space(6.0);
                    if ui.button("Generate Udev Rules").clicked() {
                        let rule = r#"# ICARUS V2 MINI Gaming Mouse — allow non-root HID access
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f511", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f511", MODE="0666"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f512", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f512", MODE="0666"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f510", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f510", MODE="0666"
"#;
                        if std::fs::write("99-icarus-mouse.rules", rule).is_ok() {
                            self.set_status(
                                "Generated 99-icarus-mouse.rules in current directory",
                                true,
                            );
                        }
                    }
                });
            });

            ui.add_space(18.0);

            // Right Column: Hardware Overview & Reverse Engineering Architecture Details
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(400.0);
                    ui.heading(
                        egui::RichText::new("Hardware & Firmware Specs")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.add_space(8.0);

                    let specs = [
                        ("Device", "ICARUS V2 MINI Wireless Gaming Mouse"),
                        ("Sensor", "PixArt PAW3395 (26,000 DPI, 650 IPS, 50G)"),
                        ("Mouse MCU", "Nordic nRF52840 (ARM Cortex-M4F @ 64MHz)"),
                        ("1K Dongle MCU", "CX52650N (2.4GHz Wireless)"),
                        ("4K Dongle MCU", "WCH CH32V305 (RISC-V High Speed)"),
                        ("USB Vendor ID", "0x3554"),
                        ("Wired PID", "0xF511 (USB Direct)"),
                        ("2.4G PID", "0xF512 (Standard Dongle)"),
                        ("4G PID", "0xF510 (High-Speed Dongle)"),
                        ("HID Endpoint", "Interface 1 (Vendor Specific)"),
                        ("Protocol Packet", "17 bytes (XOR Checksummed)"),
                    ];

                    for (k, v) in specs {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", k))
                                    .strong()
                                    .color(egui::Color32::from_rgb(71, 85, 105)),
                            );
                            ui.label(
                                egui::RichText::new(v).color(egui::Color32::from_rgb(15, 23, 42)),
                            );
                        });
                        ui.add_space(2.0);
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Firmware:")
                                .strong()
                                .color(egui::Color32::from_rgb(71, 85, 105)),
                        );
                        ui.label(&self.firmware_version);
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Connection:")
                                .strong()
                                .color(egui::Color32::from_rgb(71, 85, 105)),
                        );
                        ui.label(&self.connection_status);
                    });

                    if let Some(bat) = self.battery_level {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Battery:")
                                    .strong()
                                    .color(egui::Color32::from_rgb(71, 85, 105)),
                            );
                            ui.label(format!("{}%", bat));
                        });
                    }
                });
            });
        });
    }
}

/// Run the native Desktop GUI window.
pub fn run_gui() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([900.0, 620.0])
            .with_title("ICARUS V2 MINI — Mouse Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "ICARUS V2 MINI",
        native_options,
        Box::new(|cc| Ok(Box::new(IcarusApp::new(cc)))),
    )
}
