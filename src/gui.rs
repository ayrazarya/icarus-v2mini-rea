//! Graphical User Interface (GUI) for ICARUS V2 MINI.
//!
//! A modern, fresh, high-performance Linux desktop configuration application
//! preserving all reverse-engineered features, original icons, and hardware controls.

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

        // Apply modern, sleek dark gaming aesthetic
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

    /// Set up modern dark studio gaming theme with smooth gradients, cards, and glowing accents
    fn apply_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Canvas & panel colors
        visuals.panel_fill = egui::Color32::from_rgb(14, 17, 24);
        visuals.window_fill = egui::Color32::from_rgb(19, 23, 33);

        // Electric cyan accent
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 170, 240);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 220, 255));

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(24, 29, 41);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(220, 225, 240));

        // Inactive widgets (cards & controls)
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(28, 34, 48);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(190, 198, 220));
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);

        // Hovered states
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(38, 46, 66);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 255, 255));
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);

        // Active / Pressed states
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 140, 220);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 255, 255));
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

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

// ---------------------------------------------------------------------------
// Modern UI Design System Helpers
// ---------------------------------------------------------------------------

/// Render a modern card frame with rounded corners and subtle border stroke
fn modern_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(22, 27, 39))
        .stroke(egui::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 14),
        ))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(14.0))
        .show(ui, add_contents)
        .inner
}

/// Render a section title with an optional description subtitle
fn section_header(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>) {
    ui.label(
        egui::RichText::new(title)
            .size(15.0)
            .strong()
            .color(egui::Color32::from_rgb(0, 220, 255)),
    );
    if let Some(sub) = subtitle {
        ui.label(
            egui::RichText::new(sub)
                .size(11.5)
                .color(egui::Color32::from_rgb(150, 160, 185)),
        );
    }
    ui.add_space(8.0);
}

/// Modern pill-shaped toggle switch
fn modern_toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(38.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = if *on { 1.0 } else { 0.0 };
        let bg_fill = if *on {
            egui::Color32::from_rgb(0, 160, 240)
        } else {
            egui::Color32::from_rgb(44, 52, 70)
        };
        ui.painter().rect_filled(rect, 10.0, bg_fill);

        let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
        let circle_center = egui::pos2(circle_x, rect.center().y);
        let radius = 7.5;
        ui.painter()
            .circle_filled(circle_center, radius, egui::Color32::WHITE);
    }
    response
}

// ---------------------------------------------------------------------------
// eframe::App Implementation
// ---------------------------------------------------------------------------

impl eframe::App for IcarusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle 2.4G pairing timeout simulation
        if self.pairing_state == PairingState::Pairing
            && let Some(start) = self.pairing_start
            && start.elapsed() >= Duration::from_millis(1800)
        {
            self.pairing_state = PairingState::Success;
            self.set_status("✓ 2.4G Wireless Dongle paired successfully!", true);
        }

        // Top Modern Header Bar
        egui::TopBottomPanel::top("header_panel")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(14, 18, 26))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
                    ))
                    .inner_margin(egui::Margin::symmetric(24.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo container with subtle glowing backdrop
                    let logo_frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(22, 28, 42))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgba_unmultiplied(0, 210, 255, 50),
                        ))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(6.0));

                    logo_frame.show(ui, |ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!("../assets/Dialog/Logo.png"))
                                .max_width(26.0)
                                .max_height(26.0),
                        );
                    });

                    ui.add_space(8.0);

                    // Brand Typography
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("ICARUS")
                                    .size(18.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(0, 220, 255)),
                            );
                            ui.label(
                                egui::RichText::new("V2 MINI")
                                    .size(18.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new("PRO WIRELESS // PAW3395")
                                    .size(9.5)
                                    .color(egui::Color32::from_rgb(120, 130, 155)),
                            );
                        });
                    });

                    ui.add_space(20.0);

                    // Modern Profile Selector Capsule
                    let profile_frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(22, 28, 42))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0));

                    profile_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("PROFILES:")
                                    .size(10.5)
                                    .strong()
                                    .color(egui::Color32::from_rgb(140, 150, 175)),
                            );

                            for p in 1..=5 {
                                let is_active = self.config.active_profile == p;
                                let btn = egui::Button::new(
                                    egui::RichText::new(format!("{}", p))
                                        .size(11.0)
                                        .strong()
                                        .color(if is_active {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::from_rgb(160, 170, 195)
                                        }),
                                )
                                .fill(if is_active {
                                    egui::Color32::from_rgb(0, 140, 220)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .rounding(egui::Rounding::same(4.0))
                                .min_size(egui::vec2(22.0, 20.0));

                                if ui
                                    .add(btn)
                                    .on_hover_text(format!("Switch to Profile {}", p))
                                    .clicked()
                                {
                                    self.config.active_profile = p;
                                    self.set_status(format!("Active profile: Profile {}", p), true);
                                }
                            }
                        });
                    });

                    // Right Side: Hardware Connection & Battery Capsule
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Scan button
                        let scan_btn = egui::Button::new("🔄 Scan")
                            .fill(egui::Color32::from_rgb(28, 34, 48))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 14),
                            ))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::vec2(68.0, 28.0));

                        if ui
                            .add(scan_btn)
                            .on_hover_text("Scan for mouse connection")
                            .clicked()
                        {
                            self.refresh_device();
                        }

                        // Battery Indicator Pill
                        if let Some(battery) = self.battery_level {
                            ui.add_space(4.0);
                            let bat_frame = egui::Frame::none()
                                .fill(egui::Color32::from_rgb(22, 28, 42))
                                .rounding(egui::Rounding::same(6.0))
                                .inner_margin(egui::Margin::symmetric(8.0, 4.0));

                            bat_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if !self.is_wireless {
                                        ui.add(
                                            egui::Image::new(egui::include_image!(
                                                "../assets/Dialog/Charg.png"
                                            ))
                                            .max_height(14.0),
                                        );
                                    } else {
                                        ui.add(
                                            egui::Image::new(egui::include_image!(
                                                "../assets/Dialog/Battery.png"
                                            ))
                                            .max_height(14.0),
                                        );
                                    }

                                    let battery_color = if battery > 50 {
                                        egui::Color32::from_rgb(46, 204, 113)
                                    } else if battery > 20 {
                                        egui::Color32::from_rgb(241, 196, 15)
                                    } else {
                                        egui::Color32::from_rgb(231, 76, 60)
                                    };

                                    ui.label(
                                        egui::RichText::new(format!("{}%", battery))
                                            .size(11.5)
                                            .color(battery_color)
                                            .strong(),
                                    );
                                });
                            });
                        }

                        // Connection Status Pill
                        let is_connected = self.device.is_some();
                        let status_frame = egui::Frame::none()
                            .fill(egui::Color32::from_rgb(22, 28, 42))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                if is_connected {
                                    egui::Color32::from_rgba_unmultiplied(46, 204, 113, 60)
                                } else {
                                    egui::Color32::from_rgba_unmultiplied(231, 76, 60, 60)
                                },
                            ))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(10.0, 5.0));

                        status_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if is_connected {
                                    if self.is_wireless {
                                        ui.add(
                                            egui::Image::new(egui::include_image!(
                                                "../assets/DialogChange/OnLineWifi.png"
                                            ))
                                            .max_height(16.0),
                                        );
                                    } else {
                                        ui.add(
                                            egui::Image::new(egui::include_image!(
                                                "../assets/DialogChange/OnLineUsb.png"
                                            ))
                                            .max_height(16.0),
                                        );
                                    }
                                    ui.label(
                                        egui::RichText::new(&self.connection_status)
                                            .size(11.5)
                                            .color(egui::Color32::from_rgb(46, 204, 113))
                                            .strong(),
                                    );
                                } else {
                                    ui.add(
                                        egui::Image::new(egui::include_image!(
                                            "../assets/DialogChange/DisOnline.png"
                                        ))
                                        .max_height(16.0),
                                    );
                                    ui.label(
                                        egui::RichText::new(&self.connection_status)
                                            .size(11.5)
                                            .color(egui::Color32::from_rgb(231, 76, 60))
                                            .strong(),
                                    );
                                }
                            });
                        });
                    });
                });

                ui.add_space(10.0);

                // Modern Segmented Navigation Bar
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
                                "Buttons & Mapping",
                            ),
                            (Tab::Buttons, false) => (
                                egui::include_image!("../assets/Dialog/Home.png"),
                                "Buttons & Mapping",
                            ),
                            (Tab::Sensor, true) => (
                                egui::include_image!("../assets/Dialog/SensorChecked.png"),
                                "Sensor & DPI",
                            ),
                            (Tab::Sensor, false) => (
                                egui::include_image!("../assets/Dialog/Sensor.png"),
                                "Sensor & DPI",
                            ),
                            (Tab::Lighting, true) => (
                                egui::include_image!("../assets/Dialog/LightChecked.png"),
                                "RGB Lighting",
                            ),
                            (Tab::Lighting, false) => (
                                egui::include_image!("../assets/Dialog/Light.png"),
                                "RGB Lighting",
                            ),
                            (Tab::Macro, true) => (
                                egui::include_image!("../assets/Dialog/MacroChecked.png"),
                                "Macro Studio",
                            ),
                            (Tab::Macro, false) => (
                                egui::include_image!("../assets/Dialog/Macro.png"),
                                "Macro Studio",
                            ),
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
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(175, 185, 205)
                            },
                        );

                        let btn = egui::Button::image_and_text(
                            egui::Image::new(icon_image).max_height(17.0),
                            text,
                        )
                        .fill(if is_active {
                            egui::Color32::from_rgb(0, 140, 220)
                        } else {
                            egui::Color32::from_rgb(24, 30, 44)
                        })
                        .stroke(if is_active {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 220, 255))
                        } else {
                            egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8),
                            )
                        })
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(148.0, 36.0));

                        if ui.add(btn).clicked() {
                            self.active_tab = tab;
                        }
                    }
                });
            });

        // Bottom Action Footer Bar
        egui::TopBottomPanel::bottom("footer_panel")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(14, 18, 26))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
                    ))
                    .inner_margin(egui::Margin::symmetric(24.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Status feedback message toast
                    if let Some((msg, success, time)) = &self.status
                        && time.elapsed() < Duration::from_secs(6)
                    {
                        let (bg, border, col) = if *success {
                            (
                                egui::Color32::from_rgba_unmultiplied(46, 204, 113, 25),
                                egui::Color32::from_rgb(46, 204, 113),
                                egui::Color32::from_rgb(46, 204, 113),
                            )
                        } else {
                            (
                                egui::Color32::from_rgba_unmultiplied(231, 76, 60, 25),
                                egui::Color32::from_rgb(231, 76, 60),
                                egui::Color32::from_rgb(231, 76, 60),
                            )
                        };

                        egui::Frame::none()
                            .fill(bg)
                            .stroke(egui::Stroke::new(1.0_f32, border))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(msg).color(col).strong());
                            });
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Apply button (Primary Glowing Blue)
                        let apply_btn = egui::Button::new(
                            egui::RichText::new("✓ Apply Changes")
                                .size(13.5)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(0, 150, 235))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(0, 220, 255),
                        ))
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(150.0, 36.0));

                        if ui
                            .add(apply_btn)
                            .on_hover_text("Write changes to mouse hardware")
                            .clicked()
                        {
                            self.apply_to_device();
                        }

                        // Read from mouse button
                        let read_btn = egui::Button::new("📥 Read from Mouse")
                            .fill(egui::Color32::from_rgb(28, 34, 48))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 14),
                            ))
                            .rounding(egui::Rounding::same(8.0))
                            .min_size(egui::vec2(140.0, 36.0));

                        if ui
                            .add(read_btn)
                            .on_hover_text("Load current settings from mouse")
                            .clicked()
                        {
                            self.read_from_device();
                        }

                        // Factory Reset button with original Reset graphic
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

        // Central Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
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

// ---------------------------------------------------------------------------
// Page Renderers (Tab 1 to Tab 5)
// ---------------------------------------------------------------------------

impl IcarusApp {
    /// Render Tab 1: Button Mapping with interactive multi-angle mouse showcase
    fn render_buttons_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Interactive Mouse Showcase Pedestal
            ui.vertical(|ui| {
                ui.set_width(430.0);

                modern_card(ui, |ui| {
                    ui.set_min_size(egui::vec2(400.0, 360.0));
                    ui.vertical_centered(|ui| {
                        section_header(
                            ui,
                            "DEVICE SHOWCASE",
                            Some("Interactive angle previews of ICARUS V2 MINI"),
                        );

                        // Active View Image
                        match self.mouse_view_idx {
                            0 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse0.png"
                                    ))
                                    .max_width(360.0),
                                );
                            }
                            1 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse1.png"
                                    ))
                                    .max_width(320.0),
                                );
                            }
                            2 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse2.png"
                                    ))
                                    .max_width(320.0),
                                );
                            }
                            _ => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse3.png"
                                    ))
                                    .max_width(320.0),
                                );
                            }
                        }

                        ui.add_space(8.0);

                        // Perspective angle tabs
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                            let views = [
                                (0, "🔝 Top View"),
                                (1, "📐 Angle 1"),
                                (2, "📐 Angle 2"),
                                (3, "⬡ Side View"),
                            ];
                            for (idx, name) in views {
                                let is_active = self.mouse_view_idx == idx;
                                let btn = egui::Button::new(
                                    egui::RichText::new(name).size(11.0).color(if is_active {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::from_rgb(160, 170, 195)
                                    }),
                                )
                                .fill(if is_active {
                                    egui::Color32::from_rgb(0, 140, 220)
                                } else {
                                    egui::Color32::from_rgb(32, 38, 54)
                                })
                                .rounding(egui::Rounding::same(6.0));

                                if ui.add(btn).clicked() {
                                    self.mouse_view_idx = idx;
                                }
                            }
                        });
                    });
                });

                ui.add_space(10.0);

                // Debounce Card with original FangDouCheck graphic
                modern_card(ui, |ui| {
                    ui.set_width(400.0);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/Buttoninterface/FangDouCheck.png"
                            ))
                            .max_height(20.0),
                        );
                        ui.label(
                            egui::RichText::new("Debounce Click Filter")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 220, 255)),
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Switch contact filter: {} ms (reduces accidental double clicks)",
                            self.config.debounce_ms
                        ))
                        .size(11.5)
                        .color(egui::Color32::from_rgb(160, 170, 195)),
                    );
                    ui.add_space(4.0);
                    ui.add(
                        egui::Slider::new(&mut self.config.debounce_ms, 0..=20)
                            .suffix(" ms")
                            .step_by(2.0),
                    );
                });
            });

            ui.add_space(16.0);

            // Right Column: Button Reassignments Grid
            ui.vertical(|ui| {
                section_header(
                    ui,
                    "BUTTON ASSIGNMENTS",
                    Some("Remap the 6 hardware switches with gaming & productivity actions"),
                );

                let button_names = [
                    ("Button 1", "Left Click", "Primary click switch"),
                    ("Button 2", "Right Click", "Secondary click switch"),
                    ("Button 3", "Middle Click", "Scroll wheel click"),
                    ("Button 4", "Back", "Side thumb button 1"),
                    ("Button 5", "Forward", "Side thumb button 2"),
                    ("Button 6", "DPI Switch", "Top DPI cycle button"),
                ];

                for (idx, (btn_num, btn_default, btn_sub)) in button_names.iter().enumerate() {
                    modern_card(ui, |ui| {
                        ui.set_width(410.0);
                        ui.horizontal(|ui| {
                            // Number badge with original icon asset
                            ui.add(
                                egui::Image::new(egui::include_image!(
                                    "../assets/Buttoninterface/LabelNumber.png"
                                ))
                                .max_height(16.0),
                            );

                            ui.label(
                                egui::RichText::new(format!(" {} ", btn_num))
                                    .size(11.5)
                                    .strong()
                                    .background_color(egui::Color32::from_rgb(0, 140, 220))
                                    .color(egui::Color32::WHITE),
                            );

                            ui.label(
                                egui::RichText::new(*btn_default)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(*btn_sub)
                                            .size(10.5)
                                            .color(egui::Color32::from_rgb(130, 140, 165)),
                                    );
                                },
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
                            "Fire Key (Rapid Fire Burst)",
                            "Media (Play / Pause)",
                            "Keyboard Key (Ctrl+A)",
                            "Macro Playback",
                            "Disabled",
                        ];

                        let prev_category = action_category;
                        egui::ComboBox::from_id_salt(format!("btn_combo_{}", idx))
                            .selected_text(options[action_category])
                            .width(320.0)
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

                        // Fire Key Controls
                        if let ButtonAction::FireKey {
                            ref mut clicks,
                            ref mut delay_ms,
                        } = self.config.buttons[idx]
                        {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("Burst Count:");
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

    /// Render Tab 2: Sensor, Polling Rates (including 2K/4K), and PAW3395 DPI Stages
    fn render_sensor_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Polling Rate & Hardware Features
            ui.vertical(|ui| {
                ui.set_width(360.0);

                // Polling Rate Card
                modern_card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/Sensor/ReportRate.png"
                            ))
                            .max_height(20.0),
                        );
                        ui.label(
                            egui::RichText::new("POLLING RATE")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 220, 255)),
                        );
                    });
                    ui.label(
                        egui::RichText::new("USB report frequency & latency response")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(150, 160, 185)),
                    );
                    ui.add_space(8.0);

                    // Standard rates
                    let rates = [
                        (PollingRate::Hz125, "125 Hz", "8.0 ms latency"),
                        (PollingRate::Hz250, "250 Hz", "4.0 ms latency"),
                        (PollingRate::Hz500, "500 Hz", "2.0 ms latency"),
                        (PollingRate::Hz1000, "1000 Hz", "1.0 ms (Esports Standard)"),
                    ];

                    for (rate, label, desc) in rates {
                        let is_selected = self.config.polling_rate == rate;

                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} — {}", label, desc))
                                .size(12.0)
                                .color(if is_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(180, 190, 210)
                                })
                                .strong(),
                        )
                        .fill(if is_selected {
                            egui::Color32::from_rgb(0, 140, 220)
                        } else {
                            egui::Color32::from_rgb(32, 38, 54)
                        })
                        .stroke(if is_selected {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 220, 255))
                        } else {
                            egui::Stroke::NONE
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(320.0, 32.0));

                        if ui.add(btn).clicked() {
                            self.config.polling_rate = rate;
                        }
                        ui.add_space(3.0);
                    }

                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("High-Speed 4K Dongle Modes:")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(140, 150, 175)),
                    );
                    ui.add_space(4.0);

                    // High-speed rates with original graphics
                    ui.horizontal(|ui| {
                        let is_2k = self.config.polling_rate == PollingRate::Hz2000;
                        let btn_2k = egui::Button::image_and_text(
                            egui::Image::new(egui::include_image!(
                                "../assets/DialogChange/2KHz.png"
                            ))
                            .max_height(20.0),
                            egui::RichText::new("2000 Hz (0.5 ms)").strong(),
                        )
                        .fill(if is_2k {
                            egui::Color32::from_rgb(0, 140, 220)
                        } else {
                            egui::Color32::from_rgb(32, 38, 54)
                        })
                        .stroke(if is_2k {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 220, 255))
                        } else {
                            egui::Stroke::NONE
                        })
                        .min_size(egui::vec2(156.0, 34.0));

                        if ui.add(btn_2k).clicked() {
                            self.config.polling_rate = PollingRate::Hz2000;
                        }

                        let is_4k = self.config.polling_rate == PollingRate::Hz4000;
                        let btn_4k = egui::Button::image_and_text(
                            egui::Image::new(egui::include_image!(
                                "../assets/DialogChange/4KHz.png"
                            ))
                            .max_height(20.0),
                            egui::RichText::new("4000 Hz (0.25 ms)").strong(),
                        )
                        .fill(if is_4k {
                            egui::Color32::from_rgb(0, 140, 220)
                        } else {
                            egui::Color32::from_rgb(32, 38, 54)
                        })
                        .stroke(if is_4k {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 220, 255))
                        } else {
                            egui::Stroke::NONE
                        })
                        .min_size(egui::vec2(156.0, 34.0));

                        if ui.add(btn_4k).clicked() {
                            self.config.polling_rate = PollingRate::Hz4000;
                        }
                    });
                });

                ui.add_space(10.0);

                // Lift-Off Distance (LOD) Card
                modern_card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!("../assets/Sensor/LOD.png"))
                                .max_height(16.0),
                        );
                        ui.label(
                            egui::RichText::new("LIFT-OFF DISTANCE (LOD)")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 220, 255)),
                        );
                    });
                    ui.label(
                        egui::RichText::new("Sensor tracking cut-off altitude")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(150, 160, 185)),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let low_selected = self.config.lod == LiftOffDistance::Low;
                        let low_btn = egui::Button::new("1 mm (Low LOD)")
                            .fill(if low_selected {
                                egui::Color32::from_rgb(0, 140, 220)
                            } else {
                                egui::Color32::from_rgb(32, 38, 54)
                            })
                            .min_size(egui::vec2(154.0, 32.0));

                        if ui.add(low_btn).clicked() {
                            self.config.lod = LiftOffDistance::Low;
                        }

                        let high_selected = self.config.lod == LiftOffDistance::High;
                        let high_btn = egui::Button::new("2 mm (High LOD)")
                            .fill(if high_selected {
                                egui::Color32::from_rgb(0, 140, 220)
                            } else {
                                egui::Color32::from_rgb(32, 38, 54)
                            })
                            .min_size(egui::vec2(154.0, 32.0));

                        if ui.add(high_btn).clicked() {
                            self.config.lod = LiftOffDistance::High;
                        }
                    });
                });

                ui.add_space(10.0);

                // Power & Sensor Sleep Management
                modern_card(ui, |ui| {
                    section_header(
                        ui,
                        "POWER & SLEEP",
                        Some("Energy saving vs maximum tracking response"),
                    );

                    ui.horizontal(|ui| {
                        modern_toggle(ui, &mut self.config.performance_mode);
                        ui.label(
                            egui::RichText::new(if self.config.performance_mode {
                                "⚡ Performance Mode (0ms delay)"
                            } else {
                                "🍃 Office Eco Mode (Battery Save)"
                            })
                            .strong(),
                        );
                    });

                    ui.add_space(6.0);

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
                            .width(140.0)
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

                // Pro Sensor Feature Toggles
                modern_card(ui, |ui| {
                    section_header(
                        ui,
                        "PRO SENSOR ENHANCEMENTS",
                        Some("PixArt PAW3395 hardware features"),
                    );

                    ui.horizontal(|ui| {
                        modern_toggle(ui, &mut self.config.motion_sync);
                        ui.label("Motion Sync (Clock sync with SPI reports)");
                    });

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        modern_toggle(ui, &mut self.config.angle_snapping);
                        ui.label("Angle Snapping (Line straightening)");
                    });

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        modern_toggle(ui, &mut self.config.ripple_control);
                        ui.label("Ripple Control (High-DPI smoothing >5000)");
                    });
                });
            });

            ui.add_space(16.0);

            // Right Column: DPI Stages
            ui.vertical(|ui| {
                section_header(
                    ui,
                    "DPI STAGES (PIXART PAW3395)",
                    Some("Configure up to 5 stages from 50 to 26,000 DPI with step adjustments"),
                );

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

                    modern_card(ui, |ui| {
                        ui.set_width(420.0);
                        ui.horizontal(|ui| {
                            // Active radio
                            if ui.radio(is_active_stage, "").clicked() {
                                self.config.active_dpi_stage = i as u8;
                            }

                            // Color chip
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 7.0, stage_color);

                            // Stage title
                            ui.label(
                                egui::RichText::new(format!("Stage {}", i + 1))
                                    .size(13.5)
                                    .strong()
                                    .color(if is_active_stage {
                                        egui::Color32::from_rgb(0, 220, 255)
                                    } else {
                                        egui::Color32::WHITE
                                    }),
                            );

                            if is_active_stage {
                                ui.label(
                                    egui::RichText::new(" ACTIVE ")
                                        .size(10.5)
                                        .background_color(egui::Color32::from_rgb(0, 140, 220))
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                );
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    modern_toggle(ui, &mut self.config.dpi_stages[i].enabled);
                                    ui.label("Enabled:");
                                },
                            );
                        });

                        ui.add_space(4.0);

                        // DPI Slider with Add and Reduce buttons
                        ui.horizontal(|ui| {
                            let red_btn = egui::ImageButton::new(
                                egui::Image::new(egui::include_image!(
                                    "../assets/Sensor/Reduce.png"
                                ))
                                .max_height(14.0),
                            );
                            if ui.add(red_btn).on_hover_text("Decrease 50 DPI").clicked() {
                                let cur = self.config.dpi_stages[i].dpi;
                                self.config.dpi_stages[i].dpi = cur.saturating_sub(50).max(50);
                            }

                            let mut dpi_val = self.config.dpi_stages[i].dpi as i32;
                            let slider = egui::Slider::new(&mut dpi_val, 50..=26000)
                                .step_by(50.0)
                                .suffix(" DPI")
                                .logarithmic(true);

                            if ui.add(slider).changed() {
                                let snapped = crate::dpi::snap_dpi(dpi_val.clamp(50, 26000) as u16);
                                self.config.dpi_stages[i].dpi = snapped;
                            }

                            let add_btn = egui::ImageButton::new(
                                egui::Image::new(egui::include_image!("../assets/Sensor/Add.png"))
                                    .max_height(14.0),
                            );
                            if ui.add(add_btn).on_hover_text("Increase 50 DPI").clicked() {
                                let cur = self.config.dpi_stages[i].dpi;
                                self.config.dpi_stages[i].dpi = cur.saturating_add(50).min(26000);
                            }
                        });
                    });
                    ui.add_space(6.0);
                }
            });
        });
    }

    /// Render Tab 3: Lighting and LED Effects with ColorBoard and 14 individual color chips
    fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Lighting Mode and Sleep
            ui.vertical(|ui| {
                ui.set_width(380.0);

                modern_card(ui, |ui| {
                    section_header(
                        ui,
                        "LIGHTING MODE",
                        Some("Select dynamic illumination style for the mouse"),
                    );

                    let modes = [
                        (LedMode::Solid, "Solid", "Static continuous glow"),
                        (LedMode::Breathing, "Breathing", "Smooth pulsing animation"),
                        (
                            LedMode::Neon,
                            "Neon Spectrum",
                            "Continuous rainbow color cycle",
                        ),
                        (LedMode::Off, "Off", "Turn off LED to conserve power"),
                    ];

                    for (mode, label, desc) in modes {
                        let is_active = self.config.dpi_led.mode == mode;
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} — {}", label, desc))
                                .size(12.0)
                                .color(if is_active {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(180, 190, 210)
                                })
                                .strong(),
                        )
                        .fill(if is_active {
                            egui::Color32::from_rgb(0, 140, 220)
                        } else {
                            egui::Color32::from_rgb(32, 38, 54)
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

                modern_card(ui, |ui| {
                    section_header(
                        ui,
                        "BRIGHTNESS & SPEED",
                        Some("Adjust illumination intensity and animation rate"),
                    );

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

                    ui.label("LED Sleep Timer:");
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
                        .width(140.0)
                        .show_ui(ui, |ui| {
                            for (val, label) in sleep_options {
                                ui.selectable_value(&mut self.config.led_sleep_seconds, val, label);
                            }
                        });
                });
            });

            ui.add_space(16.0);

            // Right Column: Color Swatches & RGB Tuning
            ui.vertical(|ui| {
                modern_card(ui, |ui| {
                    ui.set_width(410.0);
                    section_header(
                        ui,
                        "COLOR PALETTE & CUSTOM RGB",
                        Some("Choose preset colors or customize Red, Green, and Blue channels"),
                    );

                    // Original ColorBoard Graphic Banner
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Light/ColorBoard.png"))
                            .max_width(380.0),
                    );

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Quick Color Presets (14 Swatches):")
                            .size(11.5)
                            .color(egui::Color32::from_rgb(160, 170, 195)),
                    );
                    ui.add_space(4.0);

                    // Row 1: ColorA to ColorG
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        let swatches_row1 = [
                            (
                                egui::include_image!("../assets/Light/ColorA.png"),
                                Color::new(255, 0, 0),
                                "Red",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorB.png"),
                                Color::new(0, 255, 255),
                                "Cyan",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorC.png"),
                                Color::new(0, 255, 0),
                                "Green",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorD.png"),
                                Color::new(255, 0, 255),
                                "Magenta",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorE.png"),
                                Color::new(255, 255, 0),
                                "Yellow",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorF.png"),
                                Color::new(255, 80, 0),
                                "Orange",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorG.png"),
                                Color::new(0, 0, 255),
                                "Blue",
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

                    // Row 2: ColorH to ColorN
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        let swatches_row2 = [
                            (
                                egui::include_image!("../assets/Light/ColorH.png"),
                                Color::new(227, 0, 127),
                                "Pink",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorI.png"),
                                Color::new(46, 166, 223),
                                "Sky",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorJ.png"),
                                Color::new(96, 25, 133),
                                "Purple",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorK.png"),
                                Color::new(255, 255, 255),
                                "White",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorL.png"),
                                Color::new(139, 178, 36),
                                "Lime",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorM.png"),
                                Color::new(255, 155, 51),
                                "Amber",
                            ),
                            (
                                egui::include_image!("../assets/Light/ColorN.png"),
                                Color::new(208, 113, 173),
                                "Orchid",
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

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Custom RGB Sliders:")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );

                    // Red slider
                    ui.horizontal(|ui| {
                        let red_reduce = egui::ImageButton::new(
                            egui::Image::new(egui::include_image!("../assets/Light/Reduce.png"))
                                .max_height(14.0),
                        );
                        if ui.add(red_reduce).clicked() {
                            self.config.dpi_led.color.r =
                                self.config.dpi_led.color.r.saturating_sub(5);
                        }
                        ui.label(
                            egui::RichText::new("R:")
                                .color(egui::Color32::from_rgb(255, 100, 100))
                                .strong(),
                        );
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.r, 0..=255));
                        let red_add = egui::ImageButton::new(
                            egui::Image::new(egui::include_image!("../assets/Light/Add.png"))
                                .max_height(14.0),
                        );
                        if ui.add(red_add).clicked() {
                            self.config.dpi_led.color.r =
                                self.config.dpi_led.color.r.saturating_add(5);
                        }
                    });

                    // Green slider
                    ui.horizontal(|ui| {
                        let grn_reduce = egui::ImageButton::new(
                            egui::Image::new(egui::include_image!("../assets/Light/Reduce.png"))
                                .max_height(14.0),
                        );
                        if ui.add(grn_reduce).clicked() {
                            self.config.dpi_led.color.g =
                                self.config.dpi_led.color.g.saturating_sub(5);
                        }
                        ui.label(
                            egui::RichText::new("G:")
                                .color(egui::Color32::from_rgb(100, 255, 100))
                                .strong(),
                        );
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.g, 0..=255));
                        let grn_add = egui::ImageButton::new(
                            egui::Image::new(egui::include_image!("../assets/Light/Add.png"))
                                .max_height(14.0),
                        );
                        if ui.add(grn_add).clicked() {
                            self.config.dpi_led.color.g =
                                self.config.dpi_led.color.g.saturating_add(5);
                        }
                    });

                    // Blue slider
                    ui.horizontal(|ui| {
                        let blu_reduce = egui::ImageButton::new(
                            egui::Image::new(egui::include_image!("../assets/Light/Reduce.png"))
                                .max_height(14.0),
                        );
                        if ui.add(blu_reduce).clicked() {
                            self.config.dpi_led.color.b =
                                self.config.dpi_led.color.b.saturating_sub(5);
                        }
                        ui.label(
                            egui::RichText::new("B:")
                                .color(egui::Color32::from_rgb(100, 180, 255))
                                .strong(),
                        );
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.b, 0..=255));
                        let blu_add = egui::ImageButton::new(
                            egui::Image::new(egui::include_image!("../assets/Light/Add.png"))
                                .max_height(14.0),
                        );
                        if ui.add(blu_add).clicked() {
                            self.config.dpi_led.color.b =
                                self.config.dpi_led.color.b.saturating_add(5);
                        }
                    });

                    ui.add_space(8.0);

                    // Color Preview Card
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/Light/PreviewColor.png"
                            ))
                            .max_height(20.0),
                        );
                        ui.label("Active Preview:");
                        let preview_col = egui::Color32::from_rgb(
                            self.config.dpi_led.color.r,
                            self.config.dpi_led.color.g,
                            self.config.dpi_led.color.b,
                        );
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(70.0, 24.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, preview_col);
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

    /// Render Tab 4: Macro Studio with Record, Stop, and Step Timeline
    fn render_macro_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Macro Library
            ui.vertical(|ui| {
                ui.set_width(320.0);

                modern_card(ui, |ui| {
                    section_header(
                        ui,
                        "MACRO LIBRARY",
                        Some("Stored macro scripts and execution sequences"),
                    );

                    let num_macros = self.config.macros.len();
                    for idx in 0..num_macros {
                        let is_selected = self.selected_macro_idx == idx;
                        let macro_name = self.config.macros[idx].name.clone();

                        let item_btn =
                            egui::Button::new(egui::RichText::new(&macro_name).size(12.5).color(
                                if is_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(180, 190, 210)
                                },
                            ))
                            .fill(if is_selected {
                                egui::Color32::from_rgb(0, 140, 220)
                            } else {
                                egui::Color32::from_rgb(32, 38, 54)
                            })
                            .min_size(egui::vec2(285.0, 30.0));

                        if ui.add(item_btn).clicked() {
                            self.selected_macro_idx = idx;
                        }
                        ui.add_space(2.0);
                    }

                    ui.add_space(6.0);

                    // Create New Macro
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_macro_name);
                        if ui.button("➕ New").clicked() && !self.new_macro_name.trim().is_empty()
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
                    if self.config.macros.len() > 1 && ui.button("🗑️ Delete Selected").clicked()
                    {
                        self.config.macros.remove(self.selected_macro_idx);
                        if self.selected_macro_idx >= self.config.macros.len() {
                            self.selected_macro_idx = self.config.macros.len() - 1;
                        }
                    }
                });

                ui.add_space(10.0);

                // Repeat Mode Settings
                if let Some(mac) = self.config.macros.get_mut(self.selected_macro_idx) {
                    modern_card(ui, |ui| {
                        section_header(ui, "REPEAT SETTINGS", Some("Execution behavior and loops"));

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

            ui.add_space(16.0);

            // Right Column: Event Timeline
            ui.vertical(|ui| {
                modern_card(ui, |ui| {
                    ui.set_width(430.0);

                    // Recording Action Bar
                    ui.horizontal(|ui| {
                        let record_img = egui::include_image!("../assets/Macro/Record.png");
                        let stop_img = egui::include_image!("../assets/Macro/Stop.png");

                        let rec_btn =
                            egui::ImageButton::new(egui::Image::new(record_img).max_height(28.0));
                        if ui
                            .add(rec_btn)
                            .on_hover_text("Start recording keystrokes")
                            .clicked()
                        {
                            self.is_recording_macro = true;
                            self.set_status("🔴 Recording macro... press Stop when done", true);
                        }

                        let stop_btn =
                            egui::ImageButton::new(egui::Image::new(stop_img).max_height(28.0));
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
                                egui::RichText::new("● REC ACTIVE")
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .strong(),
                            );
                        }
                    });

                    ui.add_space(8.0);
                    section_header(
                        ui,
                        "EVENT SEQUENCE",
                        Some("Step-by-step key actions and delay timings"),
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
                                                    .color(egui::Color32::from_rgb(180, 190, 210)),
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
                                                    .small_button("✕")
                                                    .on_hover_text("Delete event")
                                                    .clicked()
                                                {
                                                    event_to_remove = Some(e_idx);
                                                }

                                                let down_img = egui::include_image!(
                                                    "../assets/Macro/Down.png"
                                                );
                                                if ui
                                                    .add(egui::ImageButton::new(
                                                        egui::Image::new(down_img).max_height(10.0),
                                                    ))
                                                    .on_hover_text("Move down")
                                                    .clicked()
                                                    && e_idx + 1 < num_events
                                                {
                                                    move_down = Some(e_idx);
                                                }

                                                let up_img =
                                                    egui::include_image!("../assets/Macro/Up.png");
                                                if ui
                                                    .add(egui::ImageButton::new(
                                                        egui::Image::new(up_img).max_height(10.0),
                                                    ))
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

                        ui.horizontal(|ui| {
                            ui.label("Insert Delay:");
                            ui.add(egui::DragValue::new(&mut self.new_delay_ms).suffix(" ms"));
                            if ui.button("➕ Add Delay").clicked() {
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

    /// Render Tab 5: Settings, 2.4G Dongle Pairing Tool, Profile Import/Export, and Linux Udev Rules
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Pairing & Profiles
            ui.vertical(|ui| {
                ui.set_width(380.0);

                // 2.4G Dongle Pairing Card
                modern_card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/FormSetting/Mouse.png"
                            ))
                            .max_height(22.0),
                        );
                        ui.label(
                            egui::RichText::new("2.4G WIRELESS PAIRING")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 220, 255)),
                        );
                    });
                    ui.label(
                        egui::RichText::new("Pair mouse with 2.4G/4G wireless USB receiver")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(150, 160, 185)),
                    );
                    ui.add_space(8.0);

                    // Pairing status graphic
                    match self.pairing_state {
                        PairingState::Idle => {
                            let pair_img = egui::include_image!("../assets/FormSetting/Pair.png");
                            ui.add(egui::Image::new(pair_img).max_height(28.0));
                        }
                        PairingState::Pairing => {
                            let pari_btn =
                                egui::include_image!("../assets/FormSetting/PariButton.png");
                            ui.add(egui::Image::new(pari_btn).max_height(28.0));
                            ui.label("Pairing in progress... Keep mouse near dongle");
                        }
                        PairingState::Success => {
                            let succ_img =
                                egui::include_image!("../assets/FormSetting/PairSuccess.png");
                            ui.add(egui::Image::new(succ_img).max_height(28.0));
                            ui.label(
                                egui::RichText::new("✓ Dongle Paired Successfully!")
                                    .color(egui::Color32::from_rgb(46, 204, 113))
                                    .strong(),
                            );
                        }
                        PairingState::Failed => {
                            let fail_img =
                                egui::include_image!("../assets/FormSetting/PairFail.png");
                            ui.add(egui::Image::new(fail_img).max_height(28.0));
                            ui.label(
                                egui::RichText::new("Pairing Failed. Try again.")
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .strong(),
                            );
                        }
                    }

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("🔗 Start 2.4G Pairing").clicked() {
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
                modern_card(ui, |ui| {
                    section_header(
                        ui,
                        "PROFILE BACKUP (TOML)",
                        Some("Save and restore mouse configurations to disk"),
                    );

                    ui.horizontal(|ui| {
                        if ui.button("💾 Export Profile (TOML)").clicked() {
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
                                Err(e) => self.set_status(
                                    format!("Failed to serialize profile: {}", e),
                                    false,
                                ),
                            }
                        }

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
                });

                ui.add_space(10.0);

                // Linux USB Permissions Guide
                modern_card(ui, |ui| {
                    section_header(
                        ui,
                        "LINUX USB ACCESS (UDEV)",
                        Some("Allow non-root HID access without sudo"),
                    );

                    ui.code("sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/");
                    ui.code("sudo udevadm control --reload-rules && sudo udevadm trigger");

                    ui.add_space(6.0);
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

            ui.add_space(16.0);

            // Right Column: Hardware & Firmware Specs
            ui.vertical(|ui| {
                modern_card(ui, |ui| {
                    ui.set_width(410.0);
                    section_header(
                        ui,
                        "HARDWARE ARCHITECTURE",
                        Some("Reverse-engineered controller & sensor specifications"),
                    );

                    let specs = [
                        ("Device", "ICARUS V2 MINI Wireless Gaming Mouse"),
                        (
                            "Optical Sensor",
                            "PixArt PAW3395 (26,000 DPI, 650 IPS, 50G)",
                        ),
                        ("Main MCU", "Nordic nRF52840 (ARM Cortex-M4F @ 64MHz)"),
                        ("1K Wireless Dongle", "CX52650N (2.4GHz Wireless)"),
                        ("4K High-Speed Dongle", "WCH CH32V305 (RISC-V High Speed)"),
                        ("USB Vendor ID (VID)", "0x3554"),
                        ("Wired PID", "0xF511 (USB Direct)"),
                        ("2.4G Wireless PID", "0xF512 (Standard Dongle)"),
                        ("4G Wireless PID", "0xF510 (High-Speed Dongle)"),
                        ("HID Endpoint", "Interface 1 (Vendor Specific)"),
                        ("Protocol Format", "17-byte Reports with XOR Checksum"),
                    ];

                    for (k, v) in specs {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", k))
                                    .size(11.5)
                                    .color(egui::Color32::from_rgb(140, 150, 175)),
                            );
                            ui.label(
                                egui::RichText::new(v)
                                    .size(11.5)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                        });
                        ui.add_space(2.0);
                    }

                    ui.separator();

                    ui.label(format!("Firmware Version: {}", self.firmware_version));
                    ui.label(format!("Connection:       {}", self.connection_status));
                    if let Some(bat) = self.battery_level {
                        ui.label(format!("Battery Status:   {}%", bat));
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
            .with_inner_size([980.0, 680.0])
            .with_min_inner_size([920.0, 620.0])
            .with_title("ICARUS V2 MINI — Mouse Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "ICARUS V2 MINI",
        native_options,
        Box::new(|cc| Ok(Box::new(IcarusApp::new(cc)))),
    )
}
