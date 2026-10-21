//! Graphical User Interface (GUI) for ICARUS V2 MINI.
//!
//! A native Linux desktop configuration tool designed with the sleek,
//! modern visual aesthetics of Logitech G HUB for high-performance mouse customization.

use crate::config::{ButtonAction, Color, LedMode, LiftOffDistance, MouseConfig, PollingRate};
use crate::device::IcarusDevice;
use eframe::egui;
use std::time::{Duration, Instant};

/// Active navigation tab in the desktop application (G HUB style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Assignments,
    Sensitivity,
    Lightsync,
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
    /// Connection type label
    connection_type: String,
    /// On-board memory mode active toggle
    onboard_memory: bool,
}

impl IcarusApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders for embedded PNGs
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Apply Logitech G HUB dark aesthetic theme
        Self::apply_ghub_theme(&cc.egui_ctx);

        let mut app = Self {
            config: MouseConfig::default(),
            active_tab: Tab::Assignments,
            device: None,
            status: None,
            firmware_version: "1.0.1".into(),
            battery_level: None,
            connection_status: "Disconnected".into(),
            connection_type: "OFFLINE".into(),
            onboard_memory: true,
        };

        app.refresh_device();
        app
    }

    /// Set up Logitech G HUB-inspired sleek dark theme
    fn apply_ghub_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Logitech G HUB Signature Charcoal/Pitch Dark background
        visuals.panel_fill = egui::Color32::from_rgb(15, 16, 21);
        visuals.window_fill = egui::Color32::from_rgb(18, 19, 25);

        // G HUB Signature Electric Cyan accent (#00E5FF)
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 229, 255);
        visuals.selection.stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 229, 255));

        // Widget styling
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(26, 28, 37);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(220, 225, 235));
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(28, 30, 40);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(190, 195, 210));
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);

        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(38, 42, 56);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 255, 255));
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);

        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 200, 230);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(15, 16, 21));
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        ctx.set_visuals(visuals);
    }

    /// Refresh device detection and read live hardware status
    fn refresh_device(&mut self) {
        match IcarusDevice::open() {
            Ok(dev) => {
                self.connection_status = "CONNECTED".into();
                self.connection_type = dev.connection.to_uppercase();
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
                self.connection_status = "DISCONNECTED".into();
                self.connection_type = "OFFLINE MODE".into();
                self.battery_level = None;
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
                Ok(_) => self.set_status("✓ On-board memory synchronized successfully!", true),
                Err(e) => self.set_status(format!("Error applying settings: {}", e), false),
            }
        } else {
            self.set_status(
                "Mouse not connected. Connect mouse to sync on-board memory.",
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
        // G HUB Signature Left Sidebar Navigation Rail
        egui::SidePanel::left("ghub_sidebar")
            .exact_width(84.0)
            .resizable(false)
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(12, 13, 17))
                    .inner_margin(egui::Margin::symmetric(10.0, 16.0)),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // G HUB Style Brand Logo
                    ui.label(
                        egui::RichText::new("G")
                            .size(28.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );

                    ui.label(
                        egui::RichText::new("ICARUS")
                            .size(9.0)
                            .strong()
                            .color(egui::Color32::from_rgb(140, 145, 165)),
                    );

                    ui.add_space(26.0);

                    // Nav item: Lightsync (RGB)
                    self.render_sidebar_tab(
                        ui,
                        Tab::Lightsync,
                        "⚡",
                        "LIGHT",
                        "LIGHTSYNC RGB Effects",
                    );
                    ui.add_space(14.0);

                    // Nav item: Assignments (Buttons)
                    self.render_sidebar_tab(
                        ui,
                        Tab::Assignments,
                        "⌨",
                        "KEYS",
                        "Button Assignments",
                    );
                    ui.add_space(14.0);

                    // Nav item: Sensitivity (DPI)
                    self.render_sidebar_tab(
                        ui,
                        Tab::Sensitivity,
                        "🎯",
                        "DPI",
                        "Sensitivity & Sensor (DPI)",
                    );

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(10.0);

                        // Settings tab at bottom
                        self.render_sidebar_tab(
                            ui,
                            Tab::Settings,
                            "⚙",
                            "SETUP",
                            "Device Settings & Profiles",
                        );

                        ui.add_space(16.0);

                        // Battery indicator badge in sidebar
                        if let Some(bat) = self.battery_level {
                            let bat_col = if bat > 50 {
                                egui::Color32::from_rgb(0, 230, 118)
                            } else if bat > 20 {
                                egui::Color32::from_rgb(255, 179, 0)
                            } else {
                                egui::Color32::from_rgb(255, 61, 0)
                            };
                            ui.label(
                                egui::RichText::new(format!("🔋\n{}%", bat))
                                    .size(11.0)
                                    .color(bat_col)
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("⚡\nUSB")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(100, 105, 120)),
                            );
                        }
                    });
                });
            });

        // Top Navigation / Breadcrumb Bar
        egui::TopBottomPanel::top("ghub_topbar")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(15, 16, 21))
                    .inner_margin(egui::Margin::symmetric(24.0, 16.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Profile / Breadcrumbs
                    ui.label(
                        egui::RichText::new("ICARUS V2 MINI")
                            .size(16.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );

                    ui.label(
                        egui::RichText::new("▸")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(100, 105, 120)),
                    );

                    ui.label(
                        egui::RichText::new("DEFAULT PROFILE")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );

                    ui.label(
                        egui::RichText::new(if self.onboard_memory {
                            "🔒 ON-BOARD MEMORY: ON"
                        } else {
                            "🔓 ON-BOARD MEMORY: OFF"
                        })
                        .size(10.0)
                        .color(egui::Color32::from_rgb(140, 145, 165)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connection Pill Badge (G HUB Lightspeed style)
                        let is_connected = self.device.is_some();
                        let dot_color = if is_connected {
                            egui::Color32::from_rgb(0, 230, 118) // bright green
                        } else {
                            egui::Color32::from_rgb(255, 61, 0) // red
                        };

                        let pill_frame = egui::Frame::none()
                            .fill(egui::Color32::from_rgb(24, 26, 35))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgb(38, 42, 56),
                            ))
                            .rounding(egui::Rounding::same(12.0))
                            .inner_margin(egui::Margin::symmetric(12.0, 5.0));

                        pill_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("●").size(10.0).color(dot_color));
                                ui.label(
                                    egui::RichText::new(&self.connection_type)
                                        .size(11.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(210, 215, 230)),
                                );
                            });
                        });

                        ui.add_space(8.0);

                        // Scan button
                        if ui.button("⟳ Refresh").clicked() {
                            self.refresh_device();
                        }
                    });
                });
            });

        // Bottom Action Footer Bar
        egui::TopBottomPanel::bottom("ghub_footer")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(15, 16, 21))
                    .inner_margin(egui::Margin::symmetric(24.0, 14.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Status feedback message
                    if let Some((msg, success, time)) = &self.status
                        && time.elapsed() < Duration::from_secs(6)
                    {
                        let col = if *success {
                            egui::Color32::from_rgb(0, 230, 118)
                        } else {
                            egui::Color32::from_rgb(255, 61, 0)
                        };
                        ui.label(egui::RichText::new(msg).color(col).strong());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // G HUB Cyan Action Button
                        let apply_btn = egui::Button::new(
                            egui::RichText::new("SAVE TO ON-BOARD MEMORY")
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::from_rgb(12, 13, 17)),
                        )
                        .fill(egui::Color32::from_rgb(0, 229, 255))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(220.0, 36.0));

                        if ui.add(apply_btn).clicked() {
                            self.apply_to_device();
                        }

                        // Read from mouse button
                        let read_btn = egui::Button::new(
                            egui::RichText::new("SYNC FROM MOUSE")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(200, 205, 220)),
                        )
                        .fill(egui::Color32::from_rgb(26, 28, 38))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(42, 46, 62),
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(150.0, 36.0));

                        if ui.add(read_btn).clicked() {
                            self.read_from_device();
                        }

                        // Reset defaults button
                        let reset_btn = egui::Button::new(
                            egui::RichText::new("RESTORE DEFAULTS")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(140, 145, 165)),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(130.0, 36.0));

                        if ui.add(reset_btn).clicked() {
                            self.config = MouseConfig::default();
                            self.set_status("Restored default profile settings", true);
                        }
                    });
                });
            });

        // Main Center Stage
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(18, 19, 25))
                    .inner_margin(egui::Margin::symmetric(28.0, 20.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.active_tab {
                        Tab::Assignments => self.render_assignments_view(ui),
                        Tab::Sensitivity => self.render_sensitivity_view(ui),
                        Tab::Lightsync => self.render_lightsync_view(ui),
                        Tab::Settings => self.render_settings_view(ui),
                    }
                    ui.add_space(20.0);
                });
            });
    }
}

impl IcarusApp {
    /// Render vertical sidebar button in G HUB style
    fn render_sidebar_tab(
        &mut self,
        ui: &mut egui::Ui,
        tab: Tab,
        icon: &str,
        label: &str,
        tooltip: &str,
    ) {
        let is_active = self.active_tab == tab;
        let bg_fill = if is_active {
            egui::Color32::from_rgb(26, 28, 38)
        } else {
            egui::Color32::TRANSPARENT
        };

        let border_stroke = if is_active {
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 229, 255))
        } else {
            egui::Stroke::NONE
        };

        let btn = egui::Button::new(
            egui::RichText::new(format!("{}\n{}", icon, label))
                .size(11.0)
                .strong()
                .color(if is_active {
                    egui::Color32::from_rgb(0, 229, 255)
                } else {
                    egui::Color32::from_rgb(140, 145, 165)
                }),
        )
        .fill(bg_fill)
        .stroke(border_stroke)
        .rounding(egui::Rounding::same(8.0))
        .min_size(egui::vec2(64.0, 56.0));

        let resp = ui.add(btn);
        if resp.clicked() {
            self.active_tab = tab;
        }
        resp.on_hover_text(tooltip);
    }

    /// VIEW 1: ASSIGNMENTS (G HUB Button Mapping Stage with Mouse Visual & Pins)
    fn render_assignments_view(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("ASSIGNMENTS")
                .size(22.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new("CLICK PINS OR BUTTON CARDS TO ASSIGN CUSTOM ACTIONS & MACROS")
                .size(11.0)
                .color(egui::Color32::from_rgb(140, 145, 165)),
        );
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            // Center Stage: Mouse Visual Graphic
            ui.vertical(|ui| {
                ui.set_width(460.0);
                let container = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(14, 15, 20))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(28, 30, 40),
                    ))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(20.0, 20.0));

                container.show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("TOP VIEW")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(0, 229, 255))
                                .strong(),
                        );
                        ui.add_space(8.0);

                        // The high-res mouse graphic
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/Buttoninterface/mouse0.png"
                            ))
                            .max_width(360.0),
                        );

                        ui.add_space(12.0);

                        // Pin legends
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(10.0, 6.0);
                            for (idx, label) in [
                                (1, "Left Click"),
                                (2, "Right Click"),
                                (3, "Middle Click"),
                                (4, "Back"),
                                (5, "Forward"),
                                (6, "DPI Switch"),
                            ] {
                                ui.label(
                                    egui::RichText::new(format!("● {} {}", idx, label))
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(170, 175, 195)),
                                );
                            }
                        });
                    });
                });
            });

            ui.add_space(24.0);

            // Right Stage: Assignments List
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("COMMAND ASSIGNMENTS")
                        .size(14.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 229, 255)),
                );
                ui.add_space(8.0);

                let button_labels = [
                    (0, "PRIMARY CLICK", "Button 1 • Left Mouse Click"),
                    (1, "SECONDARY CLICK", "Button 2 • Right Mouse Click"),
                    (2, "MIDDLE CLICK", "Button 3 • Scroll Wheel Click"),
                    (3, "BACK", "Button 4 • Forward Thumb Button"),
                    (4, "FORWARD", "Button 5 • Backward Thumb Button"),
                    (5, "DPI CYCLE", "Button 6 • Top DPI Switch Button"),
                ];

                for (idx, title, desc) in button_labels {
                    let card = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(24, 26, 36))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(34, 38, 52),
                        ))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0));

                    card.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Cyan badge index
                            ui.label(
                                egui::RichText::new(format!(" {} ", idx + 1))
                                    .size(12.0)
                                    .strong()
                                    .background_color(egui::Color32::from_rgb(0, 229, 255))
                                    .color(egui::Color32::from_rgb(12, 13, 17)),
                            );

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(title)
                                        .size(12.0)
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                );
                                ui.label(
                                    egui::RichText::new(desc)
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(120, 125, 145)),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let current_action = self
                                        .config
                                        .buttons
                                        .get(idx)
                                        .cloned()
                                        .unwrap_or(ButtonAction::Disabled);

                                    let mut sel_idx = match current_action {
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
                                        "DPI Cycle",
                                        "Play/Pause",
                                        "Disabled",
                                    ];

                                    let prev = sel_idx;
                                    egui::ComboBox::from_id_salt(format!("ghub_btn_{}", idx))
                                        .selected_text(
                                            egui::RichText::new(options[sel_idx])
                                                .color(egui::Color32::from_rgb(0, 229, 255))
                                                .strong(),
                                        )
                                        .width(160.0)
                                        .show_ui(ui, |ui| {
                                            for (i, opt) in options.iter().enumerate() {
                                                ui.selectable_value(&mut sel_idx, i, *opt);
                                            }
                                        });

                                    if sel_idx != prev {
                                        let new_act = match sel_idx {
                                            0 => ButtonAction::MouseButton(1),
                                            1 => ButtonAction::MouseButton(2),
                                            2 => ButtonAction::MouseButton(4),
                                            3 => ButtonAction::MouseButton(8),
                                            4 => ButtonAction::MouseButton(16),
                                            5 => ButtonAction::DpiSwitch,
                                            6 => ButtonAction::Media(0xCD),
                                            7 => ButtonAction::Disabled,
                                            _ => ButtonAction::MouseButton(1),
                                        };
                                        if idx < self.config.buttons.len() {
                                            self.config.buttons[idx] = new_act;
                                        }
                                    }
                                },
                            );
                        });
                    });
                    ui.add_space(6.0);
                }
            });
        });
    }

    /// VIEW 2: SENSITIVITY (DPI) (Signature Logitech G HUB Horizontal Stepper & Report Rate Cards)
    fn render_sensitivity_view(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("SENSITIVITY (DPI)")
                .size(22.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new("PIXART PAW3395 OPTICAL SENSOR • 50 TO 26,000 DPI")
                .size(11.0)
                .color(egui::Color32::from_rgb(140, 145, 165)),
        );
        ui.add_space(20.0);

        // G HUB Signature DPI Stepper Timeline Card
        let dpi_card = egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 24, 32))
            .stroke(egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgb(32, 36, 48),
            ))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(24.0, 20.0));

        dpi_card.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("DPI SPEED PRESETS")
                        .size(13.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let active_stage_idx = self.config.active_dpi_stage as usize;
                    let active_dpi = self
                        .config
                        .dpi_stages
                        .get(active_stage_idx)
                        .map(|s| s.dpi)
                        .unwrap_or(800);

                    ui.label(
                        egui::RichText::new(format!("CURRENT DPI: {} DPI", active_dpi))
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                });
            });

            ui.add_space(16.0);

            // Horizontal DPI Presets (G HUB diamond step badges)
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(16.0, 0.0);

                for i in 0..self.config.dpi_stages.len() {
                    let is_active = self.config.active_dpi_stage == i as u8;
                    let dpi_val = self.config.dpi_stages[i].dpi;
                    let is_enabled = self.config.dpi_stages[i].enabled;

                    let stage_card = egui::Frame::none()
                        .fill(if is_active {
                            egui::Color32::from_rgb(0, 229, 255)
                        } else if is_enabled {
                            egui::Color32::from_rgb(28, 32, 44)
                        } else {
                            egui::Color32::from_rgb(20, 22, 28)
                        })
                        .stroke(if is_active {
                            egui::Stroke::new(2.0_f32, egui::Color32::WHITE)
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(40, 44, 58))
                        })
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(18.0, 12.0));

                    stage_card.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let text_col = if is_active {
                                egui::Color32::from_rgb(12, 13, 17) // dark on cyan
                            } else if is_enabled {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(100, 105, 120)
                            };

                            ui.label(
                                egui::RichText::new(format!("DPI {}", i + 1))
                                    .size(10.0)
                                    .strong()
                                    .color(text_col),
                            );

                            ui.label(
                                egui::RichText::new(format!("{}", dpi_val))
                                    .size(18.0)
                                    .strong()
                                    .color(text_col),
                            );

                            if is_active {
                                ui.label(
                                    egui::RichText::new("● DEFAULT")
                                        .size(9.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(12, 13, 17)),
                                );
                            } else if ui
                                .button(
                                    egui::RichText::new("SELECT")
                                        .size(9.0)
                                        .color(egui::Color32::from_rgb(0, 229, 255)),
                                )
                                .clicked()
                            {
                                self.config.active_dpi_stage = i as u8;
                            }
                        });
                    });
                }
            });

            ui.add_space(20.0);

            // Detailed sliders for each stage
            ui.label(
                egui::RichText::new("FINE SENSITIVITY ADJUSTMENT (50 - 26,000 DPI)")
                    .size(11.0)
                    .strong()
                    .color(egui::Color32::from_rgb(140, 145, 165)),
            );
            ui.add_space(8.0);

            for i in 0..self.config.dpi_stages.len() {
                ui.horizontal(|ui| {
                    let is_active = self.config.active_dpi_stage == i as u8;
                    ui.label(
                        egui::RichText::new(format!("Stage {}:", i + 1))
                            .size(12.0)
                            .strong()
                            .color(if is_active {
                                egui::Color32::from_rgb(0, 229, 255)
                            } else {
                                egui::Color32::from_rgb(180, 185, 200)
                            }),
                    );

                    let mut dpi_val = self.config.dpi_stages[i].dpi as i32;
                    let slider = egui::Slider::new(&mut dpi_val, 50..=26000)
                        .step_by(50.0)
                        .suffix(" DPI")
                        .logarithmic(true);

                    if ui.add(slider).changed() {
                        let snapped = crate::dpi::snap_dpi(dpi_val.clamp(50, 26000) as u16);
                        self.config.dpi_stages[i].dpi = snapped;
                    }

                    ui.checkbox(&mut self.config.dpi_stages[i].enabled, "Active");
                });
                ui.add_space(4.0);
            }
        });

        ui.add_space(20.0);

        // G HUB Signature Report Rate (Hz) Pill Buttons Card
        let rate_card = egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 24, 32))
            .stroke(egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgb(32, 36, 48),
            ))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(24.0, 18.0));

        rate_card.show(ui, |ui| {
            ui.label(
                egui::RichText::new("REPORT RATE (PER SECOND)")
                    .size(13.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.label(
                egui::RichText::new(
                    "Higher report rates increase tracking smoothness and reduce input latency.",
                )
                .size(11.0)
                .color(egui::Color32::from_rgb(140, 145, 165)),
            );
            ui.add_space(14.0);

            // G HUB 4-Pill Buttons (125, 250, 500, 1000)
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(12.0, 0.0);

                let rates = [
                    (PollingRate::Hz125, "125", "8 ms"),
                    (PollingRate::Hz250, "250", "4 ms"),
                    (PollingRate::Hz500, "500", "2 ms"),
                    (PollingRate::Hz1000, "1000", "1 ms"),
                ];

                for (r, label, latency) in rates {
                    let is_sel = self.config.polling_rate == r;

                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{}\n{}", label, latency))
                            .size(13.0)
                            .strong()
                            .color(if is_sel {
                                egui::Color32::from_rgb(12, 13, 17)
                            } else {
                                egui::Color32::WHITE
                            }),
                    )
                    .fill(if is_sel {
                        egui::Color32::from_rgb(0, 229, 255)
                    } else {
                        egui::Color32::from_rgb(28, 30, 42)
                    })
                    .stroke(if is_sel {
                        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 229, 255))
                    } else {
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(42, 46, 62))
                    })
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(egui::vec2(110.0, 52.0));

                    if ui.add(btn).clicked() {
                        self.config.polling_rate = r;
                    }
                }
            });
        });

        ui.add_space(20.0);

        // Sensor Advanced Features Card (HERO Sensor style)
        let advanced_card = egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 24, 32))
            .stroke(egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgb(32, 36, 48),
            ))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(24.0, 18.0));

        advanced_card.show(ui, |ui| {
            ui.label(
                egui::RichText::new("ADVANCED SENSOR TUNING")
                    .size(13.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(260.0);
                    ui.label(
                        egui::RichText::new("LIFT-OFF DISTANCE (LOD)")
                            .size(11.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let is_low = self.config.lod == LiftOffDistance::Low;
                        let low_btn = egui::Button::new("1 mm (Low)")
                            .fill(if is_low {
                                egui::Color32::from_rgb(0, 229, 255)
                            } else {
                                egui::Color32::from_rgb(28, 30, 42)
                            })
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(100.0, 32.0));

                        if ui.add(low_btn).clicked() {
                            self.config.lod = LiftOffDistance::Low;
                        }

                        let is_high = self.config.lod == LiftOffDistance::High;
                        let high_btn = egui::Button::new("2 mm (High)")
                            .fill(if is_high {
                                egui::Color32::from_rgb(0, 229, 255)
                            } else {
                                egui::Color32::from_rgb(28, 30, 42)
                            })
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(100.0, 32.0));

                        if ui.add(high_btn).clicked() {
                            self.config.lod = LiftOffDistance::High;
                        }
                    });
                });

                ui.add_space(24.0);

                ui.vertical(|ui| {
                    ui.set_width(260.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "CLICK DEBOUNCE: {} MS",
                            self.config.debounce_ms
                        ))
                        .size(11.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add_space(6.0);
                    ui.add(
                        egui::Slider::new(&mut self.config.debounce_ms, 0..=20)
                            .step_by(2.0)
                            .suffix(" ms"),
                    );
                });
            });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.config.motion_sync,
                    "Motion Sync (Harmonize polling with sensor frames)",
                );
                ui.add_space(16.0);
                ui.checkbox(
                    &mut self.config.angle_snapping,
                    "Angle Snapping (Path prediction)",
                );
                ui.add_space(16.0);
                ui.checkbox(&mut self.config.ripple_control, "Ripple Smoothing");
            });
        });
    }

    /// VIEW 3: LIGHTSYNC (G HUB RGB Lighting Experience)
    fn render_lightsync_view(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("LIGHTSYNC RGB")
                .size(22.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new("CUSTOMIZE PRIMARY LIGHTING EFFECTS AND ANIMATIONS")
                .size(11.0)
                .color(egui::Color32::from_rgb(140, 145, 165)),
        );
        ui.add_space(20.0);

        ui.horizontal(|ui| {
            // Left Column: Effect Controls
            ui.vertical(|ui| {
                ui.set_width(380.0);

                let fx_card = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(22, 24, 32))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(32, 36, 48),
                    ))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(20.0, 18.0));

                fx_card.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("EFFECT")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add_space(10.0);

                    let modes = [
                        (LedMode::Solid, "FIXED", "Static solid color illumination"),
                        (
                            LedMode::Breathing,
                            "BREATHING",
                            "Smooth pulsing glow animation",
                        ),
                        (LedMode::Off, "OFF", "Disable all RGB lighting"),
                    ];

                    for (m, title, desc) in modes {
                        let is_sel = self.config.dpi_led.mode == m;
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} — {}", title, desc))
                                .size(12.0)
                                .strong()
                                .color(if is_sel {
                                    egui::Color32::from_rgb(12, 13, 17)
                                } else {
                                    egui::Color32::WHITE
                                }),
                        )
                        .fill(if is_sel {
                            egui::Color32::from_rgb(0, 229, 255)
                        } else {
                            egui::Color32::from_rgb(28, 30, 42)
                        })
                        .stroke(if is_sel {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 229, 255))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(40, 44, 58))
                        })
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(340.0, 38.0));

                        if ui.add(btn).clicked() {
                            self.config.dpi_led.mode = m;
                        }
                        ui.add_space(6.0);
                    }

                    ui.add_space(14.0);

                    ui.label(
                        egui::RichText::new(format!(
                            "BRIGHTNESS ({}%)",
                            self.config.dpi_led.brightness * 10
                        ))
                        .size(11.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add(egui::Slider::new(
                        &mut self.config.dpi_led.brightness,
                        0..=10,
                    ));

                    ui.add_space(12.0);

                    ui.label(
                        egui::RichText::new(format!(
                            "RATE / SPEED (LEVEL {})",
                            self.config.dpi_led.breath_speed
                        ))
                        .size(11.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add(egui::Slider::new(
                        &mut self.config.dpi_led.breath_speed,
                        1..=10,
                    ));
                });
            });

            ui.add_space(20.0);

            // Right Column: Color Picker & G HUB Color Palette
            ui.vertical(|ui| {
                ui.set_width(400.0);

                let color_card = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(22, 24, 32))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(32, 36, 48),
                    ))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(20.0, 18.0));

                color_card.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("PRIMARY COLOR")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add_space(12.0);

                    // G HUB Signature Palette
                    let presets = [
                        ("Logitech Cyan", Color::new(0, 229, 255)),
                        ("Crimson Red", Color::RED),
                        ("Emerald Green", Color::GREEN),
                        ("Electric Blue", Color::new(0, 110, 255)),
                        ("Solar Yellow", Color::YELLOW),
                        ("Cyber Magenta", Color::MAGENTA),
                        ("Deep Purple", Color::PURPLE),
                        ("Pure White", Color::WHITE),
                        ("Neon Orange", Color::new(255, 110, 0)),
                        ("Hot Pink", Color::new(255, 20, 147)),
                    ];

                    ui.label(
                        egui::RichText::new("PALETTE PRESETS")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(140, 145, 165)),
                    );
                    ui.add_space(6.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                        for (name, col) in presets {
                            let egui_col = egui::Color32::from_rgb(col.r, col.g, col.b);
                            let is_current = self.config.dpi_led.color.r == col.r
                                && self.config.dpi_led.color.g == col.g
                                && self.config.dpi_led.color.b == col.b;

                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                            ui.painter().rect_filled(rect, 6.0, egui_col);

                            if is_current {
                                ui.painter().rect_stroke(
                                    rect,
                                    6.0,
                                    egui::Stroke::new(2.5_f32, egui::Color32::WHITE),
                                );
                            }

                            if resp.clicked() {
                                self.config.dpi_led.color = col;
                            }
                            resp.on_hover_text(name);
                        }
                    });

                    ui.add_space(18.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.label(
                        egui::RichText::new("RGB TUNING")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(140, 145, 165)),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("R: ");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.r, 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label("G: ");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.g, 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label("B: ");
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.b, 0..=255));
                    });

                    ui.add_space(14.0);

                    // Preview bar
                    ui.horizontal(|ui| {
                        let cur_col = egui::Color32::from_rgb(
                            self.config.dpi_led.color.r,
                            self.config.dpi_led.color.g,
                            self.config.dpi_led.color.b,
                        );
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(100.0, 32.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 6.0, cur_col);

                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "#{:02X}{:02X}{:02X}",
                                    self.config.dpi_led.color.r,
                                    self.config.dpi_led.color.g,
                                    self.config.dpi_led.color.b
                                ))
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new("ACTIVE COLOR")
                                    .size(9.0)
                                    .color(egui::Color32::from_rgb(140, 145, 165)),
                            );
                        });
                    });
                });
            });
        });
    }

    /// VIEW 4: SETTINGS & ON-BOARD MEMORY (G HUB Device Settings)
    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("DEVICE SETTINGS")
                .size(22.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new("ON-BOARD MEMORY SLOTS, HARDWARE INFORMATION & LINUX PERMISSIONS")
                .size(11.0)
                .color(egui::Color32::from_rgb(140, 145, 165)),
        );
        ui.add_space(20.0);

        ui.horizontal(|ui| {
            // Left Column: On-Board Profiles
            ui.vertical(|ui| {
                ui.set_width(380.0);

                let profile_card = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(22, 24, 32))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(32, 36, 48),
                    ))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(20.0, 18.0));

                profile_card.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("ON-BOARD MEMORY PROFILE")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Export or import complete hardware configurations to TOML files:",
                        )
                        .size(11.0)
                        .color(egui::Color32::from_rgb(140, 145, 165)),
                    );
                    ui.add_space(14.0);

                    if ui
                        .add(
                            egui::Button::new("💾 EXPORT PROFILE (TOML)")
                                .fill(egui::Color32::from_rgb(28, 30, 42))
                                .stroke(egui::Stroke::new(
                                    1.0_f32,
                                    egui::Color32::from_rgb(42, 46, 62),
                                ))
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(340.0, 36.0)),
                        )
                        .clicked()
                    {
                        match toml::to_string_pretty(&self.config) {
                            Ok(toml_str) => {
                                let path = "icarus-profile.toml";
                                if std::fs::write(path, toml_str).is_ok() {
                                    self.set_status(format!("✓ Profile saved to {}", path), true);
                                } else {
                                    self.set_status("Failed to write profile", false);
                                }
                            }
                            Err(e) => self.set_status(format!("Failed to serialize: {}", e), false),
                        }
                    }

                    ui.add_space(8.0);

                    if ui
                        .add(
                            egui::Button::new("📂 IMPORT PROFILE (TOML)")
                                .fill(egui::Color32::from_rgb(28, 30, 42))
                                .stroke(egui::Stroke::new(
                                    1.0_f32,
                                    egui::Color32::from_rgb(42, 46, 62),
                                ))
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(340.0, 36.0)),
                        )
                        .clicked()
                    {
                        let path = "icarus-profile.toml";
                        match std::fs::read_to_string(path) {
                            Ok(c) => match toml::from_str::<MouseConfig>(&c) {
                                Ok(cfg) => {
                                    self.config = cfg;
                                    self.set_status(
                                        format!("✓ Profile loaded from {}", path),
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

                ui.add_space(16.0);

                // Udev Permissions Card
                let udev_card = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(22, 24, 32))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(32, 36, 48),
                    ))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(20.0, 18.0));

                udev_card.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("LINUX USB PERMISSIONS (UDEV)")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Allows direct hardware access without sudo privileges:",
                        )
                        .size(11.0)
                        .color(egui::Color32::from_rgb(140, 145, 165)),
                    );
                    ui.add_space(8.0);

                    ui.code("sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/");
                    ui.code("sudo udevadm control --reload-rules && sudo udevadm trigger");
                    ui.add_space(10.0);

                    if ui
                        .add(
                            egui::Button::new("📋 Generate 99-icarus-mouse.rules")
                                .fill(egui::Color32::from_rgb(28, 30, 42))
                                .stroke(egui::Stroke::new(
                                    1.0_f32,
                                    egui::Color32::from_rgb(42, 46, 62),
                                ))
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(340.0, 32.0)),
                        )
                        .clicked()
                    {
                        let rule = r#"# ICARUS V2 MINI Gaming Mouse — allow non-root HID access
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f511", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f511", MODE="0666"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f512", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f512", MODE="0666"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f510", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f510", MODE="0666"
"#;
                        let _ = std::fs::write("99-icarus-mouse.rules", rule);
                        self.set_status("✓ Generated 99-icarus-mouse.rules", true);
                    }
                });
            });

            ui.add_space(20.0);

            // Right Column: Hardware Specifications & Diagnostics
            ui.vertical(|ui| {
                ui.set_width(400.0);

                let hw_card = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(22, 24, 32))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(32, 36, 48),
                    ))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(20.0, 18.0));

                hw_card.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("HARDWARE SPECIFICATIONS")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add_space(10.0);

                    let specs = [
                        ("Device Model", "ICARUS V2 MINI Ultra-Lightweight"),
                        (
                            "Optical Sensor",
                            "PixArt PAW3395 (26,000 DPI, 650 IPS, 50G)",
                        ),
                        ("Main Controller", "Nordic nRF52840 (ARM Cortex-M4F)"),
                        ("2.4G Receiver", "CX52650N High-Speed Wireless"),
                        ("4G Receiver", "WCH CH32V305 RISC-V Wireless"),
                        ("USB Vendor ID", "0x3554"),
                        ("Product IDs", "0xF511 (USB) / 0xF512 (2.4G) / 0xF510 (4G)"),
                        ("Firmware Rev", &self.firmware_version),
                        ("Connection Mode", &self.connection_type),
                    ];

                    for (k, v) in specs {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(k)
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(140, 145, 165)),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(v)
                                            .size(11.0)
                                            .strong()
                                            .color(egui::Color32::WHITE),
                                    );
                                },
                            );
                        });
                        ui.add_space(4.0);
                    }
                });
            });
        });
    }
}

/// Run the native Desktop GUI window with Logitech G HUB styling.
pub fn run_gui() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1020.0, 680.0])
            .with_min_inner_size([940.0, 600.0])
            .with_title("ICARUS V2 MINI — Gaming Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "ICARUS V2 MINI",
        native_options,
        Box::new(|cc| Ok(Box::new(IcarusApp::new(cc)))),
    )
}
