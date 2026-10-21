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
    /// Selected mouse view image index (0: top, 1: angle1, 2: angle2, 3: side)
    mouse_view_idx: usize,
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

        // Configure sleek dark gaming theme matching original software
        Self::apply_theme(&cc.egui_ctx);

        let mut app = Self {
            config: MouseConfig::default(),
            active_tab: Tab::Buttons,
            device: None,
            status: None,
            firmware_version: "Unknown".into(),
            battery_level: None,
            connection_status: "Disconnected (Offline Mode)".into(),
            is_wireless: false,
            mouse_view_idx: 0,
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
                self.connection_status = format!("Connected ({})", dev.connection);

                if let Ok(fw) = dev.read_firmware_version() {
                    self.firmware_version = fw;
                }
                self.battery_level = dev.read_battery_level().ok();

                if let Ok(live_cfg) = dev.read_config() {
                    self.config = live_cfg;
                }

                self.set_status("Connected to ICARUS V2 MINI", true);
                self.device = Some(dev);
            }
            Err(_) => {
                self.device = None;
                self.connection_status = "Disconnected (Offline Mode)".into();
                self.battery_level = None;
                self.is_wireless = false;
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
            match dev.apply_config(&self.config) {
                Ok(_) => self.set_status("Settings applied successfully to mouse!", true),
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
            match dev.read_config() {
                Ok(cfg) => {
                    self.config = cfg;
                    self.battery_level = dev.read_battery_level().ok();
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

                    // App Title
                    ui.label(
                        egui::RichText::new("ICARUS V2 MINI")
                            .size(19.0)
                            .strong()
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    ui.label(
                        egui::RichText::new("NATIVE LINUX REIMPLEMENTATION")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(100, 116, 139)),
                    );

                    ui.add_space(20.0);

                    // Original ProjectChoose / Profile Selector (Profiles 1 to 5)
                    ui.label(
                        egui::RichText::new("Profile:")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(51, 65, 85)),
                    );
                    let profiles = [
                        (1, "Profile 1"),
                        (2, "Profile 2"),
                        (3, "Profile 3"),
                        (4, "Profile 4"),
                        (5, "Profile 5"),
                    ];
                    let current_profile_name = format!("Profile {}", self.config.active_profile);
                    egui::ComboBox::from_id_salt("header_profile_combo")
                        .selected_text(current_profile_name)
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            for (p_id, p_name) in profiles {
                                if ui
                                    .selectable_value(&mut self.config.active_profile, p_id, p_name)
                                    .clicked()
                                {
                                    self.set_status(format!("Switched to {}", p_name), true);
                                }
                            }
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connection Status with clean dot indicator
                        let is_connected = self.device.is_some();

                        let (dot_rect, _resp) =
                            ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                        let dot_color = if is_connected {
                            egui::Color32::from_rgb(34, 197, 94)
                        } else {
                            egui::Color32::from_rgb(239, 68, 68)
                        };
                        ui.painter()
                            .circle_filled(dot_rect.center(), 4.5, dot_color);

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&self.connection_status)
                                .size(12.0)
                                .color(if is_connected {
                                    egui::Color32::from_rgb(22, 163, 74)
                                } else {
                                    egui::Color32::from_rgb(220, 38, 38)
                                })
                                .strong(),
                        );

                        // Battery Indicator
                        if let Some(battery) = self.battery_level {
                            ui.separator();
                            let battery_color = if battery > 50 {
                                egui::Color32::from_rgb(22, 163, 74)
                            } else if battery > 20 {
                                egui::Color32::from_rgb(217, 119, 6)
                            } else {
                                egui::Color32::from_rgb(220, 38, 38)
                            };

                            ui.label(
                                egui::RichText::new(format!("Battery: {}%", battery))
                                    .size(12.0)
                                    .color(battery_color)
                                    .strong(),
                            );
                        }

                        ui.separator();

                        // Scan Device button (No emojis!)
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
                        .rounding(egui::Rounding::same(5.0));

                        if ui.add(scan_btn).clicked() {
                            self.refresh_device();
                        }
                    });
                });

                ui.add_space(8.0);

                // Tab Navigation Bar with all 5 Original Reverse-Engineered Tabs
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

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
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(136.0, 36.0));

                        if ui.add(btn).clicked() {
                            self.active_tab = tab;
                        }
                    }
                });
            });

        // Bottom Action Footer Bar with Original Reset Image
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
                        // Apply button (Primary)
                        let apply_btn = egui::Button::new(
                            egui::RichText::new("Apply Changes")
                                .size(13.5)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(2, 132, 199))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(140.0, 36.0));

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
                    .fill(egui::Color32::from_rgb(248, 249, 250))
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
    /// Render Tab 1: Button Mapping with embedded mouse views & 6 programmable buttons
    fn render_buttons_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Visual Mouse Graphic with multi-angle selector
            ui.vertical(|ui| {
                ui.set_width(430.0);
                ui.group(|ui| {
                    ui.set_min_size(egui::vec2(410.0, 370.0));
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);

                        // Select active mouse image angle
                        match self.mouse_view_idx {
                            0 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse0.png"
                                    ))
                                    .max_width(380.0),
                                );
                            }
                            1 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse1.png"
                                    ))
                                    .max_width(330.0),
                                );
                            }
                            2 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse2.png"
                                    ))
                                    .max_width(330.0),
                                );
                            }
                            _ => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse3.png"
                                    ))
                                    .max_width(330.0),
                                );
                            }
                        }

                        ui.add_space(6.0);

                        // Angle thumbnails / view switch buttons
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                            let views = [
                                (0, "Top View"),
                                (1, "Angle 1"),
                                (2, "Angle 2"),
                                (3, "Side View"),
                            ];
                            for (idx, name) in views {
                                let is_active = self.mouse_view_idx == idx;
                                if ui
                                    .selectable_label(is_active, name)
                                    .on_hover_text(format!("Show {}", name))
                                    .clicked()
                                {
                                    self.mouse_view_idx = idx;
                                }
                            }
                        });

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("ICARUS V2 MINI — 6 Programmable Buttons")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 116, 139)),
                        );
                    });
                });

                ui.add_space(10.0);

                // Debounce Setting Card with Original FangDouCheck Graphic
                ui.group(|ui| {
                    ui.set_width(410.0);
                    ui.label(
                        egui::RichText::new("Debounce Click Filter")
                            .size(13.5)
                            .strong()
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Anti-shake filter: {} ms (prevents switch double-clicks)",
                            self.config.debounce_ms
                        ))
                        .size(12.0)
                        .color(egui::Color32::from_rgb(71, 85, 105)),
                    );
                    ui.add_space(4.0);
                    ui.add_sized(
                        egui::vec2(380.0, 24.0),
                        egui::Slider::new(&mut self.config.debounce_ms, 0..=20)
                            .suffix(" ms")
                            .step_by(2.0)
                            .trailing_fill(true),
                    );
                });
            });

            ui.add_space(18.0);

            // Right Column: Button Reassignments
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("Button Configuration")
                        .color(egui::Color32::from_rgb(2, 132, 199)),
                );
                ui.label("Configure action bindings for all 6 mouse buttons:");
                ui.add_space(10.0);

                let button_names = [
                    "Button 1: Left Click",
                    "Button 2: Right Click",
                    "Button 3: Middle Click (Wheel)",
                    "Button 4: Backward (Side 1)",
                    "Button 5: Forward (Side 2)",
                    "Button 6: DPI Cycle Button",
                ];

                for (idx, name) in button_names.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(380.0);
                        ui.horizontal(|ui| {
                            // Crisp number badge
                            let (rect, _response) = ui
                                .allocate_exact_size(egui::vec2(22.0, 20.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                4.0,
                                egui::Color32::from_rgb(2, 132, 199),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{}", idx + 1),
                                egui::FontId::proportional(12.0),
                                egui::Color32::WHITE,
                            );

                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(*name)
                                    .size(13.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(15, 23, 42)),
                            );
                        });

                        ui.add_space(4.0);

                        let current_action = self
                            .config
                            .buttons
                            .get(idx)
                            .cloned()
                            .unwrap_or(ButtonAction::Disabled);

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
                            "Back (Side 1)",
                            "Forward (Side 2)",
                            "DPI Switch / Cycle",
                            "Fire Key (Rapid Fire)",
                            "Media (Play / Pause)",
                            "Keyboard Key (Ctrl+A)",
                            "Macro Playback",
                            "Disabled",
                        ];

                        let prev_category = action_category;
                        egui::ComboBox::from_id_salt(format!("btn_action_combo_{}", idx))
                            .selected_text(options[action_category])
                            .width(280.0)
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
                            if idx < self.config.buttons.len() {
                                self.config.buttons[idx] = new_action;
                            }
                        }

                        // If Fire Key is selected, show burst configuration sliders
                        if let Some(ButtonAction::FireKey { clicks, delay_ms }) =
                            self.config.buttons.get_mut(idx)
                        {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("Clicks:");
                                ui.add(egui::Slider::new(clicks, 2..=15));
                                ui.label("Interval:");
                                ui.add(egui::Slider::new(delay_ms, 5..=50).suffix("ms"));
                            });
                        }
                    });
                    ui.add_space(4.0);
                }
            });
        });
    }

    /// Render Tab 2: Sensor, Polling Rate, 2K/4K Dongle Rates, and PAW3395 DPI Stages
    fn render_sensor_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Polling Rate & Advanced Features
            ui.vertical(|ui| {
                ui.set_width(330.0);

                // Polling Rate Card including 2000Hz and 4000Hz high-speed dongle modes
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Polling Rate")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    ui.label("USB & wireless report frequency:");
                    ui.add_space(8.0);

                    // Standard rates (125, 250, 500, 1000 Hz)
                    let standard_rates = [
                        (PollingRate::Hz125, "125 Hz", "8 ms latency"),
                        (PollingRate::Hz250, "250 Hz", "4 ms latency"),
                        (PollingRate::Hz500, "500 Hz", "2 ms latency"),
                        (PollingRate::Hz1000, "1000 Hz", "1 ms latency (Gaming)"),
                    ];

                    for (rate, label, desc) in standard_rates {
                        let is_selected = self.config.polling_rate == rate;

                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} — {}", label, desc))
                                .size(12.5)
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
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(300.0, 32.0));

                        if ui.add(btn).clicked() {
                            self.config.polling_rate = rate;
                        }
                        ui.add_space(3.0);
                    }

                    ui.add_space(4.0);
                    ui.label("High-Speed 4K Dongle Modes:");
                    ui.add_space(4.0);

                    // High-speed modes (2000 Hz & 4000 Hz)
                    ui.horizontal(|ui| {
                        let is_2k = self.config.polling_rate == PollingRate::Hz2000;
                        let btn_2k = egui::Button::new(
                            egui::RichText::new("2000 Hz (2K Mode)")
                                .strong()
                                .color(if is_2k {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                }),
                        )
                        .fill(if is_2k {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(if is_2k {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240))
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(146.0, 34.0));

                        if ui.add(btn_2k).clicked() {
                            self.config.polling_rate = PollingRate::Hz2000;
                        }

                        let is_4k = self.config.polling_rate == PollingRate::Hz4000;
                        let btn_4k = egui::Button::new(
                            egui::RichText::new("4000 Hz (4K Mode)")
                                .strong()
                                .color(if is_4k {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                }),
                        )
                        .fill(if is_4k {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(if is_4k {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240))
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(146.0, 34.0));

                        if ui.add(btn_4k).clicked() {
                            self.config.polling_rate = PollingRate::Hz4000;
                        }
                    });
                });

                ui.add_space(10.0);

                // Lift-Off Distance (LOD) Card
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Lift-Off Distance (LOD)")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Sensor tracking cut-off height:");
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let low_selected = self.config.lod == LiftOffDistance::Low;
                        let low_btn =
                            egui::Button::new(egui::RichText::new("1 mm (Low)").strong().color(
                                if low_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                },
                            ))
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
                            .min_size(egui::vec2(145.0, 30.0));

                        if ui.add(low_btn).clicked() {
                            self.config.lod = LiftOffDistance::Low;
                        }

                        let high_selected = self.config.lod == LiftOffDistance::High;
                        let high_btn =
                            egui::Button::new(egui::RichText::new("2 mm (High)").strong().color(
                                if high_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                },
                            ))
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
                            .min_size(egui::vec2(145.0, 30.0));

                        if ui.add(high_btn).clicked() {
                            self.config.lod = LiftOffDistance::High;
                        }
                    });
                });

                ui.add_space(10.0);

                // Performance Mode vs Office Mode & Sensor Sleep
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Power & Sleep Management")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.add_space(4.0);

                    // Performance mode toggle
                    ui.horizontal(|ui| {
                        let toggle_img = if self.config.performance_mode {
                            egui::include_image!("../assets/Sensor/Open.png")
                        } else {
                            egui::include_image!("../assets/Sensor/Close.png")
                        };
                        if ui
                            .add(egui::ImageButton::new(
                                egui::Image::new(toggle_img).max_height(18.0),
                            ))
                            .clicked()
                        {
                            self.config.performance_mode = !self.config.performance_mode;
                        }

                        ui.label(
                            egui::RichText::new(if self.config.performance_mode {
                                "Performance Mode (High Responsiveness)"
                            } else {
                                "Office Mode (Power-Saving / Extended Battery)"
                            })
                            .color(egui::Color32::from_rgb(15, 23, 42)),
                        );
                    });

                    ui.add_space(6.0);

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
                    ui.add_space(4.0);

                    // Motion Sync
                    ui.horizontal(|ui| {
                        let sync_img = if self.config.motion_sync {
                            egui::include_image!("../assets/Sensor/Open.png")
                        } else {
                            egui::include_image!("../assets/Sensor/Close.png")
                        };
                        if ui
                            .add(egui::ImageButton::new(
                                egui::Image::new(sync_img).max_height(16.0),
                            ))
                            .clicked()
                        {
                            self.config.motion_sync = !self.config.motion_sync;
                        }
                        ui.label("Motion Sync (Clock Synchronization)");
                    });

                    ui.add_space(4.0);

                    // Angle Snapping (StraightLine)
                    ui.horizontal(|ui| {
                        let snap_img = if self.config.angle_snapping {
                            egui::include_image!("../assets/Sensor/Open.png")
                        } else {
                            egui::include_image!("../assets/Sensor/Close.png")
                        };
                        if ui
                            .add(egui::ImageButton::new(
                                egui::Image::new(snap_img).max_height(16.0),
                            ))
                            .clicked()
                        {
                            self.config.angle_snapping = !self.config.angle_snapping;
                        }
                        ui.label("Angle Snapping (Straight Line Correction)");
                    });

                    ui.add_space(4.0);

                    // Ripple Control (RippeControl)
                    ui.horizontal(|ui| {
                        let ripple_img = if self.config.ripple_control {
                            egui::include_image!("../assets/Sensor/Open.png")
                        } else {
                            egui::include_image!("../assets/Sensor/Close.png")
                        };
                        if ui
                            .add(egui::ImageButton::new(
                                egui::Image::new(ripple_img).max_height(16.0),
                            ))
                            .clicked()
                        {
                            self.config.ripple_control = !self.config.ripple_control;
                        }
                        ui.label("Ripple Control (High DPI Smoothing)");
                    });
                });
            });

            ui.add_space(18.0);

            // Right Column: DPI Stages with PAW3395 Stepping and Add/Reduce buttons
            ui.vertical(|ui| {
                ui.set_width(520.0);
                ui.heading(
                    egui::RichText::new("DPI Stages (PixArt PAW3395)")
                        .color(egui::Color32::from_rgb(2, 132, 199)),
                );
                ui.label("Configure up to 5 DPI stages (50 – 26,000 DPI):");
                ui.add_space(8.0);

                let default_colors = [
                    egui::Color32::from_rgb(255, 60, 60),   // Stage 1: Red
                    egui::Color32::from_rgb(0, 230, 230),   // Stage 2: Cyan
                    egui::Color32::from_rgb(60, 240, 80),   // Stage 3: Green
                    egui::Color32::from_rgb(255, 255, 255), // Stage 4: White
                    egui::Color32::from_rgb(255, 230, 40),  // Stage 5: Yellow
                ];

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

                            // Distinct circular color chip with dark outline (so white stage 4 is sharp)
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
                                    }),
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

    /// Render Tab 3: Lighting and LED Effects with original ColorBoard banner and 14 individual color chips
    fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Lighting Mode and Timers
            ui.vertical(|ui| {
                ui.set_width(380.0);

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

            // Right Column: ColorBoard banner and 14 individual ColorA–ColorN swatch buttons
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(400.0);
                    ui.heading(
                        egui::RichText::new("LED Color Palette")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    ui.add_space(4.0);

                    // Original ColorBoard graphic banner
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Light/ColorBoard.png"))
                            .max_width(360.0),
                    );

                    ui.add_space(8.0);
                    ui.label("Click any preset swatch from the original software palette:");
                    ui.add_space(6.0);

                    // 14 Individual Reverse-Engineered Color Swatches (Row 1: A-G, Row 2: H-N)
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        // ColorA to ColorG
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

                        // ColorH to ColorN
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
                        ui.label("Preview:");
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
                        ui.label(format!(
                            "#{:02X}{:02X}{:02X}",
                            self.config.dpi_led.color.r,
                            self.config.dpi_led.color.g,
                            self.config.dpi_led.color.b
                        ));
                    });
                });
            });
        });
    }

    /// Render Tab 4: Macro Manager with original Record, Stop, KeyDown, KeyUp, and Time icons
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

                    // New Macro ("NewHong") creation
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

                    // Delete Macro ("DeleteHong")
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

                // Macro Repeat Mode ("ReNumber", "ReUntilLoose", "ReUntilClick")
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

            // Right Column: Event List with Record / Stop buttons & timeline
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(420.0);

                    // Top Action Bar with clean Record and Stop buttons
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
                        egui::RichText::new("Event Sequence")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    let mut event_to_remove = None;
                    let mut move_up = None;
                    let mut move_down = None;

                    if let Some(mac) = self.config.macros.get_mut(self.selected_macro_idx) {
                        let num_events = mac.events.len();
                        egui::ScrollArea::vertical()
                            .max_height(220.0)
                            .show(ui, |ui| {
                                for (e_idx, event) in mac.events.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        // Original Event Type Icons
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
                                                    .strong(),
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
                                                    .color(egui::Color32::from_rgb(100, 116, 139)),
                                                );
                                            }
                                            MacroEventType::Delay => {
                                                ui.add(
                                                    egui::Image::new(egui::include_image!(
                                                        "../assets/Macro/MacroTime.png"
                                                    ))
                                                    .max_height(16.0),
                                                );
                                                ui.label("Delay:");
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
                                    ui.separator();
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

                        ui.add_space(6.0);

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

                        ui.horizontal(|ui| {
                            ui.label("Insert Delay:");
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

                // 2.4G Wireless Pairing Tool
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("2.4G Wireless Pairing Tool")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    ui.label("Pair your ICARUS V2 MINI mouse to its 2.4G/4G wireless dongle:");
                    ui.add_space(8.0);

                    // Pairing status indicator
                    match self.pairing_state {
                        PairingState::Idle => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("[Ready]")
                                        .color(egui::Color32::from_rgb(2, 132, 199))
                                        .strong(),
                                );
                                ui.label("Dongle is waiting for pairing command");
                            });
                        }
                        PairingState::Pairing => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("[Pairing...]")
                                        .color(egui::Color32::from_rgb(234, 88, 12))
                                        .strong(),
                                );
                                ui.label("Pairing in progress... Keep mouse close to dongle");
                            });
                        }
                        PairingState::Success => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("[Paired]")
                                        .color(egui::Color32::from_rgb(22, 163, 74))
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new("Dongle Paired Successfully!")
                                        .color(egui::Color32::from_rgb(22, 163, 74))
                                        .strong(),
                                );
                            });
                        }
                        PairingState::Failed => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("[Failed]")
                                        .color(egui::Color32::from_rgb(220, 38, 38))
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new("Pairing Failed. Try again.")
                                        .color(egui::Color32::from_rgb(220, 38, 38))
                                        .strong(),
                                );
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
                    ui.add_space(6.0);

                    ui.label("Device:           ICARUS V2 MINI Wireless Gaming Mouse");
                    ui.label("Sensor:           PixArt PAW3395 (26,000 DPI, 650 IPS, 50G)");
                    ui.label("Mouse MCU:        Nordic nRF52840 (ARM Cortex-M4F @ 64MHz)");
                    ui.label("1K Dongle MCU:    CX52650N (2.4GHz Wireless)");
                    ui.label("4K Dongle MCU:    WCH CH32V305 (RISC-V High Speed)");
                    ui.label("USB Vendor ID:    0x3554");
                    ui.label("Wired PID:        0xF511 (USB Direct)");
                    ui.label("2.4G PID:         0xF512 (Standard Dongle)");
                    ui.label("4G PID:           0xF510 (High-Speed Dongle)");
                    ui.label("HID Endpoint:     Interface 1 (Vendor Specific)");
                    ui.label("Protocol Packet:  17 bytes (XOR Checksummed)");
                    ui.label(format!("Firmware:         {}", self.firmware_version));
                    ui.label(format!("Connection:       {}", self.connection_status));

                    if let Some(bat) = self.battery_level {
                        ui.label(format!("Battery:          {}%", bat));
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
            .with_inner_size([960.0, 660.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("ICARUS V2 MINI — Mouse Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "ICARUS V2 MINI",
        native_options,
        Box::new(|cc| Ok(Box::new(IcarusApp::new(cc)))),
    )
}
