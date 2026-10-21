//! Graphical User Interface (GUI) for ICARUS V2 MINI.
//!
//! A native Linux desktop configuration tool recreating the original
//! Windows application with all reverse-engineered original icons and graphics.

use crate::config::{ButtonAction, Color, LedMode, LiftOffDistance, MouseConfig, PollingRate};
use crate::device::IcarusDevice;
use eframe::egui;
use std::time::{Duration, Instant};

/// Active navigation tab in the desktop application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Buttons,
    Sensor,
    Lighting,
    Settings,
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
}

impl IcarusApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders for embedded PNGs
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Configure custom sleek dark gaming theme
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
        };

        app.refresh_device();
        app
    }

    /// Set up custom dark theme matching the original gaming mouse software
    fn apply_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Dark background tones matching original software
        visuals.panel_fill = egui::Color32::from_rgb(22, 22, 28);
        visuals.window_fill = egui::Color32::from_rgb(28, 28, 36);

        // Electric cyan accent colors
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 160, 230);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 210, 255));

        // Widget styling
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(34, 34, 44);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(220, 220, 230));

        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 40, 52);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(200, 200, 215));
        visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(52, 54, 70);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 255, 255));
        visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);

        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 130, 200);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 255, 255));
        visuals.widgets.active.rounding = egui::Rounding::same(6.0);

        ctx.set_visuals(visuals);
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
        if let Some(ref dev) = self.device {
            match dev.apply_config(&self.config) {
                Ok(_) => self.set_status("✓ Settings applied successfully to mouse!", true),
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
                    self.set_status("✓ Configuration loaded from mouse!", true);
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
        // Top Header Bar featuring original reverse-engineered Logo and Icons
        egui::TopBottomPanel::top("header_panel")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(18, 18, 24))
                    .inner_margin(egui::Margin::symmetric(20.0, 14.0)),
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
                            .size(20.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );

                    ui.label(
                        egui::RichText::new("NATIVE LINUX CONFIGURATION")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(120, 120, 140)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connection Icon & Badge
                        let is_connected = self.device.is_some();

                        if is_connected {
                            if self.is_wireless {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/DialogChange/OnLineWifi.png"
                                    ))
                                    .max_height(18.0),
                                );
                            } else {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/DialogChange/OnLineUsb.png"
                                    ))
                                    .max_height(18.0),
                                );
                            }

                            ui.label(
                                egui::RichText::new(&self.connection_status)
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(46, 204, 113))
                                    .strong(),
                            );
                        } else {
                            ui.add(
                                egui::Image::new(egui::include_image!(
                                    "../assets/DialogChange/DisOnline.png"
                                ))
                                .max_height(18.0),
                            );

                            ui.label(
                                egui::RichText::new(&self.connection_status)
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .strong(),
                            );
                        }

                        // Original Battery icon & percentage if wireless
                        if let Some(battery) = self.battery_level {
                            ui.separator();
                            ui.add(
                                egui::Image::new(egui::include_image!(
                                    "../assets/Dialog/Battery.png"
                                ))
                                .max_height(16.0),
                            );

                            let battery_color = if battery > 50 {
                                egui::Color32::from_rgb(46, 204, 113)
                            } else if battery > 20 {
                                egui::Color32::from_rgb(241, 196, 15)
                            } else {
                                egui::Color32::from_rgb(231, 76, 60)
                            };

                            ui.label(
                                egui::RichText::new(format!("{}%", battery))
                                    .size(12.0)
                                    .color(battery_color)
                                    .strong(),
                            );
                        }

                        ui.separator();

                        // Reconnect button
                        if ui.button("🔄 Scan").clicked() {
                            self.refresh_device();
                        }
                    });
                });

                ui.add_space(10.0);

                // Tab Navigation with Original Reverse-Engineered Tab Icons
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                    let tabs = [Tab::Buttons, Tab::Sensor, Tab::Lighting, Tab::Settings];

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
                            (Tab::Settings, true) => (
                                egui::include_image!("../assets/Dialog/SettingClick.png"),
                                "Settings",
                            ),
                            (Tab::Settings, false) => (
                                egui::include_image!("../assets/Dialog/Setting.png"),
                                "Settings",
                            ),
                        };

                        let text = egui::RichText::new(label_text).size(13.0).strong().color(
                            if is_active {
                                egui::Color32::from_rgb(0, 210, 255)
                            } else {
                                egui::Color32::from_rgb(180, 180, 200)
                            },
                        );

                        let btn = egui::Button::image_and_text(
                            egui::Image::new(icon_image).max_height(18.0),
                            text,
                        )
                        .fill(if is_active {
                            egui::Color32::from_rgb(32, 42, 60)
                        } else {
                            egui::Color32::from_rgb(26, 26, 34)
                        })
                        .stroke(if is_active {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 180, 240))
                        } else {
                            egui::Stroke::NONE
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(145.0, 36.0));

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
                    .fill(egui::Color32::from_rgb(18, 18, 24))
                    .inner_margin(egui::Margin::symmetric(20.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Status feedback message
                    if let Some((msg, success, time)) = &self.status
                        && time.elapsed() < Duration::from_secs(6)
                    {
                        let col = if *success {
                            egui::Color32::from_rgb(46, 204, 113)
                        } else {
                            egui::Color32::from_rgb(231, 76, 60)
                        };
                        ui.label(egui::RichText::new(msg).color(col).strong());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Apply button (Primary)
                        let apply_btn = egui::Button::new(
                            egui::RichText::new("✓ Apply Changes")
                                .size(14.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(0, 140, 220))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(150.0, 36.0));

                        if ui.add(apply_btn).clicked() {
                            self.apply_to_device();
                        }

                        // Read from mouse button
                        let read_btn = egui::Button::new("📥 Read from Mouse")
                            .fill(egui::Color32::from_rgb(38, 38, 50))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(130.0, 36.0));

                        if ui.add(read_btn).clicked() {
                            self.read_from_device();
                        }

                        // Reset button displaying the original Reset graphic
                        let reset_img = egui::include_image!("../assets/Buttoninterface/Reset.png");
                        let reset_btn =
                            egui::ImageButton::new(egui::Image::new(reset_img).max_height(26.0));

                        if ui
                            .add(reset_btn)
                            .on_hover_text("Reset to factory defaults")
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
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);
                match self.active_tab {
                    Tab::Buttons => self.render_buttons_tab(ui),
                    Tab::Sensor => self.render_sensor_tab(ui),
                    Tab::Lighting => self.render_lighting_tab(ui),
                    Tab::Settings => self.render_settings_tab(ui),
                }
                ui.add_space(20.0);
            });
        });
    }
}

impl IcarusApp {
    /// Render Tab 1: Button Mapping with embedded mouse visual & original number badges
    fn render_buttons_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Visual Mouse Graphic
            ui.vertical(|ui| {
                ui.set_width(440.0);
                ui.group(|ui| {
                    ui.set_min_size(egui::vec2(420.0, 380.0));
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        // Display the embedded mouse graphic from reverse engineering
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/Buttoninterface/mouse0.png"
                            ))
                            .max_width(380.0),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("ICARUS V2 MINI — 6 Programmable Buttons")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(140, 140, 160)),
                        );
                    });
                });
            });

            ui.add_space(20.0);

            // Right Column: Button Reassignments
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("Button Configuration")
                        .color(egui::Color32::from_rgb(0, 210, 255)),
                );
                ui.label("Configure button actions and shortcuts:");
                ui.add_space(12.0);

                let button_names = [
                    "Button 1: Left Click",
                    "Button 2: Right Click",
                    "Button 3: Middle Click (Scroll Wheel)",
                    "Button 4: Backward (Side Button 1)",
                    "Button 5: Forward (Side Button 2)",
                    "Button 6: DPI Switch Button",
                ];

                for (idx, name) in button_names.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(360.0);
                        ui.horizontal(|ui| {
                            // Number badge
                            ui.label(
                                egui::RichText::new(format!(" {} ", idx + 1))
                                    .size(13.0)
                                    .strong()
                                    .background_color(egui::Color32::from_rgb(0, 140, 220))
                                    .color(egui::Color32::WHITE),
                            );

                            ui.label(egui::RichText::new(*name).strong());
                        });

                        ui.add_space(4.0);

                        let current_action = self
                            .config
                            .buttons
                            .get(idx)
                            .cloned()
                            .unwrap_or(ButtonAction::Disabled);
                        let mut action_type = match current_action {
                            ButtonAction::MouseButton(1) => 0,
                            ButtonAction::MouseButton(2) => 1,
                            ButtonAction::MouseButton(4) => 2,
                            ButtonAction::MouseButton(8) => 3,
                            ButtonAction::MouseButton(16) => 4,
                            ButtonAction::DpiSwitch => 5,
                            ButtonAction::Media(_) => 6,
                            ButtonAction::Disabled => 7,
                            _ => 0,
                        };

                        let options = [
                            "Left Click",
                            "Right Click",
                            "Middle Click",
                            "Back (Side 1)",
                            "Forward (Side 2)",
                            "DPI Switch",
                            "Media (Play/Pause)",
                            "Disabled",
                        ];

                        let prev_type = action_type;
                        egui::ComboBox::from_id_salt(format!("btn_combo_{}", idx))
                            .selected_text(options[action_type])
                            .width(260.0)
                            .show_ui(ui, |ui| {
                                for (opt_idx, opt_name) in options.iter().enumerate() {
                                    ui.selectable_value(&mut action_type, opt_idx, *opt_name);
                                }
                            });

                        if action_type != prev_type {
                            let new_action = match action_type {
                                0 => ButtonAction::MouseButton(1),
                                1 => ButtonAction::MouseButton(2),
                                2 => ButtonAction::MouseButton(4),
                                3 => ButtonAction::MouseButton(8),
                                4 => ButtonAction::MouseButton(16),
                                5 => ButtonAction::DpiSwitch,
                                6 => ButtonAction::Media(0xCD), // Play/Pause
                                7 => ButtonAction::Disabled,
                                _ => ButtonAction::MouseButton(1),
                            };
                            if idx < self.config.buttons.len() {
                                self.config.buttons[idx] = new_action;
                            }
                        }
                    });
                    ui.add_space(6.0);
                }
            });
        });
    }

    /// Render Tab 2: Sensor, Polling Rate, and DPI Stages
    fn render_sensor_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Polling Rate & Advanced Features
            ui.vertical(|ui| {
                ui.set_width(320.0);

                // Polling Rate Card
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new("Polling Rate")
                                .color(egui::Color32::from_rgb(0, 210, 255)),
                        );
                    });

                    ui.label("USB report frequency:");
                    ui.add_space(10.0);

                    let rates = [
                        (PollingRate::Hz125, "125 Hz", "8 ms latency"),
                        (PollingRate::Hz250, "250 Hz", "4 ms latency"),
                        (PollingRate::Hz500, "500 Hz", "2 ms latency"),
                        (PollingRate::Hz1000, "1000 Hz", "1 ms latency (Gaming)"),
                    ];

                    for (rate, label, desc) in rates {
                        let is_selected = self.config.polling_rate == rate;

                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} — {}", label, desc))
                                .size(13.0)
                                .color(if is_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(180, 180, 200)
                                })
                                .strong(),
                        )
                        .fill(if is_selected {
                            egui::Color32::from_rgb(0, 130, 200)
                        } else {
                            egui::Color32::from_rgb(32, 32, 42)
                        })
                        .stroke(if is_selected {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 210, 255))
                        } else {
                            egui::Stroke::NONE
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(280.0, 36.0));

                        if ui.add(btn).clicked() {
                            self.config.polling_rate = rate;
                        }
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(12.0);

                // Lift-Off Distance (LOD)
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Lift-Off Distance (LOD)")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.label("Sensor tracking cut-off height:");
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let low_selected = self.config.lod == LiftOffDistance::Low;
                        let low_btn = egui::Button::new("1 mm (Low)")
                            .fill(if low_selected {
                                egui::Color32::from_rgb(0, 130, 200)
                            } else {
                                egui::Color32::from_rgb(32, 32, 42)
                            })
                            .min_size(egui::vec2(130.0, 32.0));

                        if ui.add(low_btn).clicked() {
                            self.config.lod = LiftOffDistance::Low;
                        }

                        let high_selected = self.config.lod == LiftOffDistance::High;
                        let high_btn = egui::Button::new("2 mm (High)")
                            .fill(if high_selected {
                                egui::Color32::from_rgb(0, 130, 200)
                            } else {
                                egui::Color32::from_rgb(32, 32, 42)
                            })
                            .min_size(egui::vec2(130.0, 32.0));

                        if ui.add(high_btn).clicked() {
                            self.config.lod = LiftOffDistance::High;
                        }
                    });
                });

                ui.add_space(12.0);

                // Debounce Time
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Debounce Time")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.label(format!(
                        "Switch click filter: {} ms",
                        self.config.debounce_ms
                    ));
                    ui.add_space(6.0);
                    ui.add(
                        egui::Slider::new(&mut self.config.debounce_ms, 0..=20)
                            .suffix(" ms")
                            .step_by(2.0),
                    );
                });

                ui.add_space(12.0);

                // Sensor Feature Toggles
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Sensor Features")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.add_space(6.0);

                    ui.checkbox(
                        &mut self.config.motion_sync,
                        "Motion Sync (Synchronize reports with sensor)",
                    );
                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut self.config.angle_snapping,
                        "Angle Snapping (Line Straightening)",
                    );
                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut self.config.ripple_control,
                        "Ripple Control (High DPI smoothing)",
                    );
                });
            });

            ui.add_space(20.0);

            // Right Column: DPI Stages
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("DPI Stages (PixArt PAW3395)")
                        .color(egui::Color32::from_rgb(0, 210, 255)),
                );
                ui.label("Configure up to 5 DPI stages (50 – 26,000 DPI):");
                ui.add_space(10.0);

                let default_colors = [
                    egui::Color32::from_rgb(255, 60, 60),   // Stage 1: Red
                    egui::Color32::from_rgb(0, 230, 230),   // Stage 2: Cyan
                    egui::Color32::from_rgb(60, 240, 80),   // Stage 3: Green
                    egui::Color32::from_rgb(255, 255, 255), // Stage 4: White
                    egui::Color32::from_rgb(255, 230, 40),  // Stage 5: Yellow
                ];

                for i in 0..self.config.dpi_stages.len() {
                    let is_active_stage = self.config.active_dpi_stage == i as u8;
                    let stage_color = default_colors
                        .get(i)
                        .copied()
                        .unwrap_or(egui::Color32::WHITE);

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Active stage radio button
                            if ui.radio(is_active_stage, "").clicked() {
                                self.config.active_dpi_stage = i as u8;
                            }

                            // Stage indicator color box
                            let (rect, _response) = ui
                                .allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 3.0, stage_color);

                            // Stage title
                            ui.label(
                                egui::RichText::new(format!("Stage {}", i + 1))
                                    .size(14.0)
                                    .strong()
                                    .color(if is_active_stage {
                                        egui::Color32::from_rgb(0, 210, 255)
                                    } else {
                                        egui::Color32::WHITE
                                    }),
                            );

                            if is_active_stage {
                                ui.label(
                                    egui::RichText::new("ACTIVE")
                                        .size(11.0)
                                        .background_color(egui::Color32::from_rgb(0, 140, 220))
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                );
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.checkbox(&mut self.config.dpi_stages[i].enabled, "Enable");
                                },
                            );
                        });

                        ui.add_space(6.0);

                        // DPI slider
                        let mut dpi_val = self.config.dpi_stages[i].dpi as i32;
                        let slider = egui::Slider::new(&mut dpi_val, 50..=26000)
                            .step_by(50.0)
                            .suffix(" DPI")
                            .logarithmic(true);

                        if ui.add(slider).changed() {
                            let snapped = crate::dpi::snap_dpi(dpi_val.clamp(50, 26000) as u16);
                            self.config.dpi_stages[i].dpi = snapped;
                        }
                    });
                    ui.add_space(8.0);
                }
            });
        });
    }

    /// Render Tab 3: Lighting and LED Effects with original ColorBoard graphic
    fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(400.0);

                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("LED Effect Mode")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.label("Select DPI LED lighting style:");
                    ui.add_space(10.0);

                    let modes = [
                        (LedMode::Solid, "Solid", "Constant static illumination"),
                        (LedMode::Breathing, "Breathing", "Pulsing glow animation"),
                        (LedMode::Off, "Off", "Disable mouse lighting"),
                    ];

                    for (mode, label, desc) in modes {
                        let is_active = self.config.dpi_led.mode == mode;
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} — {}", label, desc))
                                .size(13.0)
                                .color(if is_active {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(180, 180, 200)
                                })
                                .strong(),
                        )
                        .fill(if is_active {
                            egui::Color32::from_rgb(0, 130, 200)
                        } else {
                            egui::Color32::from_rgb(32, 32, 42)
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(360.0, 36.0));

                        if ui.add(btn).clicked() {
                            self.config.dpi_led.mode = mode;
                        }
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(14.0);

                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Brightness & Speed")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.add_space(6.0);

                    ui.label(format!("Brightness: {}", self.config.dpi_led.brightness));
                    ui.add(egui::Slider::new(
                        &mut self.config.dpi_led.brightness,
                        0..=10,
                    ));

                    ui.add_space(8.0);

                    ui.label(format!(
                        "Breathing Speed: {}",
                        self.config.dpi_led.breath_speed
                    ));
                    ui.add(egui::Slider::new(
                        &mut self.config.dpi_led.breath_speed,
                        1..=10,
                    ));
                });
            });

            ui.add_space(20.0);

            // Right Column: Color Swatches with Original ColorBoard Graphic & Custom RGB
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(380.0);
                    ui.heading(
                        egui::RichText::new("LED Color")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );

                    ui.add_space(6.0);

                    // Display the reverse-engineered ColorBoard visual banner
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Light/ColorBoard.png"))
                            .max_width(340.0),
                    );

                    ui.add_space(8.0);
                    ui.label("Select from preset colors or customize RGB:");
                    ui.add_space(8.0);

                    // Preset Color Swatches
                    let presets = [
                        ("Red", Color::RED),
                        ("Cyan", Color::CYAN),
                        ("Green", Color::GREEN),
                        ("White", Color::WHITE),
                        ("Yellow", Color::YELLOW),
                        ("Magenta", Color::MAGENTA),
                        ("Purple", Color::PURPLE),
                        ("Blue", Color::new(0, 80, 255)),
                        ("Orange", Color::new(255, 128, 0)),
                        ("Pink", Color::new(255, 105, 180)),
                    ];

                    ui.label("Color Presets:");
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                        for (name, col) in presets {
                            let egui_col = egui::Color32::from_rgb(col.r, col.g, col.b);
                            let is_current = self.config.dpi_led.color.r == col.r
                                && self.config.dpi_led.color.g == col.g
                                && self.config.dpi_led.color.b == col.b;

                            let (rect, response) = ui
                                .allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
                            ui.painter().rect_filled(rect, 4.0, egui_col);

                            if is_current {
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    egui::Stroke::new(2.5_f32, egui::Color32::WHITE),
                                );
                            }

                            if response.clicked() {
                                self.config.dpi_led.color = col;
                            }
                            response.on_hover_text(name);
                        }
                    });

                    ui.add_space(14.0);
                    ui.label("RGB Sliders:");

                    ui.horizontal(|ui| {
                        ui.label("Red:  ");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.r, 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Green:");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.g, 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Blue: ");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.b, 0..=255));
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Preview:");
                        let preview_col = egui::Color32::from_rgb(
                            self.config.dpi_led.color.r,
                            self.config.dpi_led.color.g,
                            self.config.dpi_led.color.b,
                        );
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(80.0, 28.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, preview_col);
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

    /// Render Tab 4: Profiles, Udev Rules, and Hardware Information
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Profile Management
            ui.vertical(|ui| {
                ui.set_width(380.0);
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Profile Management")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.label("Save or load custom mouse configuration files:");
                    ui.add_space(10.0);

                    if ui.button("💾 Export Current Profile (TOML)").clicked() {
                        match toml::to_string_pretty(&self.config) {
                            Ok(toml_str) => {
                                let file_path = "icarus-profile.toml";
                                if std::fs::write(file_path, toml_str).is_ok() {
                                    self.set_status(
                                        format!("✓ Saved profile to {}", file_path),
                                        true,
                                    );
                                } else {
                                    self.set_status("Failed to write profile file", false);
                                }
                            }
                            Err(e) => self
                                .set_status(format!("Failed to serialize profile: {}", e), false),
                        }
                    }

                    ui.add_space(6.0);

                    if ui.button("📂 Import Profile (TOML)").clicked() {
                        let file_path = "icarus-profile.toml";
                        match std::fs::read_to_string(file_path) {
                            Ok(content) => match toml::from_str::<MouseConfig>(&content) {
                                Ok(cfg) => {
                                    self.config = cfg;
                                    self.set_status(
                                        format!("✓ Loaded profile from {}", file_path),
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

                ui.add_space(14.0);

                // Udev Permissions Guide
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("USB Permissions (Udev)")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.label("To communicate with the mouse without sudo, install the udev rule:");
                    ui.add_space(6.0);

                    ui.code("sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/");
                    ui.code("sudo udevadm control --reload-rules");
                    ui.code("sudo udevadm trigger");

                    ui.add_space(8.0);
                    if ui.button("📋 Generate 99-icarus-mouse.rules").clicked() {
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
                                "✓ Generated 99-icarus-mouse.rules in current directory",
                                true,
                            );
                        }
                    }
                });
            });

            ui.add_space(20.0);

            // Right Column: Hardware Overview
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(380.0);
                    ui.heading(
                        egui::RichText::new("Hardware & Firmware Info")
                            .color(egui::Color32::from_rgb(0, 210, 255)),
                    );
                    ui.add_space(8.0);

                    ui.label("Device:           ICARUS V2 MINI Gaming Mouse");
                    ui.label("Sensor:           PixArt PAW3395 (26,000 DPI, 650 IPS)");
                    ui.label("MCU:              Nordic nRF52840 (ARM Cortex-M4F)");
                    ui.label("2.4G Dongle:      CX52650N");
                    ui.label("4G Dongle:        WCH CH32V305");
                    ui.label("USB VID:          0x3554");
                    ui.label("USB PID:          0xF511 (Wired) / 0xF512 (Wireless)");
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
            .with_inner_size([940.0, 640.0])
            .with_min_inner_size([880.0, 580.0])
            .with_title("ICARUS V2 MINI — Mouse Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "ICARUS V2 MINI",
        native_options,
        Box::new(|cc| Ok(Box::new(IcarusApp::new(cc)))),
    )
}
