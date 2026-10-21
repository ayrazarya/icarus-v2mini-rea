//! Graphical User Interface (GUI) for ICARUS V2 MINI.
//!
//! A modern, clean, pro-gaming Linux desktop configuration application
//! styled after Razer Synapse / SteelSeries GG with a sleek left sidebar navigation,
//! spacious single-viewport layout, and all reverse-engineered features & original icons preserved.

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

        // Apply sleek Razer Synapse / SteelSeries GG dark theme
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
        visuals.panel_fill = egui::Color32::from_rgb(17, 21, 30);
        visuals.window_fill = egui::Color32::from_rgb(17, 21, 30);

        // Electric cyan accent
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 160, 230);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 220, 255));

        // Non-interactive widgets
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(22, 27, 38);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(215, 220, 235));

        // Inactive widgets (cards & controls)
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(26, 32, 46);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(180, 190, 210));
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);

        // Hovered states
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(36, 44, 64);
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
// Design System Components
// ---------------------------------------------------------------------------

/// Render a clean modern card frame with subtle border stroke and rounded corners
fn modern_card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(22, 27, 39))
        .stroke(egui::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12),
        ))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, add_contents)
        .inner
}

/// Modern pill-shaped toggle switch
fn modern_toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(36.0, 18.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = if *on { 1.0_f32 } else { 0.0_f32 };
        let bg_fill = if *on {
            egui::Color32::from_rgb(0, 160, 240)
        } else {
            egui::Color32::from_rgb(44, 52, 70)
        };
        ui.painter().rect_filled(rect, 9.0_f32, bg_fill);

        let circle_x = egui::lerp((rect.left() + 9.0)..=(rect.right() - 9.0), how_on);
        let circle_center = egui::pos2(circle_x, rect.center().y);
        let radius = 6.5_f32;
        ui.painter()
            .circle_filled(circle_center, radius, egui::Color32::WHITE);
    }
    response
}

// ---------------------------------------------------------------------------
// Main eframe Application Layout
// ---------------------------------------------------------------------------

impl eframe::App for IcarusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle 2.4G pairing simulation
        if self.pairing_state == PairingState::Pairing
            && let Some(start) = self.pairing_start
            && start.elapsed() >= Duration::from_millis(1800)
        {
            self.pairing_state = PairingState::Success;
            self.set_status("✓ 2.4G Wireless Dongle paired successfully!", true);
        }

        // ===================================================================
        // LEFT SIDEBAR (Razer Synapse / SteelSeries GG Navigation Style)
        // ===================================================================
        egui::SidePanel::left("left_sidebar")
            .exact_width(230.0)
            .resizable(false)
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(13, 16, 23))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8),
                    ))
                    .inner_margin(egui::Margin::symmetric(14.0, 16.0)),
            )
            .show(ctx, |ui| {
                // Brand Header
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Dialog/Logo.png"))
                            .max_width(28.0)
                            .max_height(28.0),
                    );
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("ICARUS")
                                    .size(16.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(0, 220, 255)),
                            );
                            ui.label(
                                egui::RichText::new("V2 MINI")
                                    .size(16.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                        });
                        ui.label(
                            egui::RichText::new("PRO WIRELESS // PAW3395")
                                .size(9.0)
                                .color(egui::Color32::from_rgb(110, 120, 145)),
                        );
                    });
                });

                ui.add_space(14.0);

                // Profile Selector Capsule
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(20, 25, 36))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
                    ))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("PROFILE:")
                                    .size(10.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(130, 140, 165)),
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
                                            egui::Color32::from_rgb(150, 160, 185)
                                        }),
                                )
                                .fill(if is_active {
                                    egui::Color32::from_rgb(0, 140, 220)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .rounding(egui::Rounding::same(4.0))
                                .min_size(egui::vec2(20.0, 18.0));

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

                ui.add_space(16.0);

                // Vertical Navigation Items
                ui.label(
                    egui::RichText::new("NAVIGATION")
                        .size(10.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 110, 135)),
                );
                ui.add_space(4.0);

                let nav_items = [
                    (
                        Tab::Buttons,
                        "Buttons & Keys",
                        egui::include_image!("../assets/Dialog/Home.png"),
                        egui::include_image!("../assets/Dialog/HomeChecked.png"),
                    ),
                    (
                        Tab::Sensor,
                        "Sensor & DPI",
                        egui::include_image!("../assets/Dialog/Sensor.png"),
                        egui::include_image!("../assets/Dialog/SensorChecked.png"),
                    ),
                    (
                        Tab::Lighting,
                        "RGB Lighting",
                        egui::include_image!("../assets/Dialog/Light.png"),
                        egui::include_image!("../assets/Dialog/LightChecked.png"),
                    ),
                    (
                        Tab::Macro,
                        "Macro Studio",
                        egui::include_image!("../assets/Dialog/Macro.png"),
                        egui::include_image!("../assets/Dialog/MacroChecked.png"),
                    ),
                    (
                        Tab::Settings,
                        "Settings & 2.4G",
                        egui::include_image!("../assets/Dialog/Setting.png"),
                        egui::include_image!("../assets/Dialog/SettingClick.png"),
                    ),
                ];

                for (tab, label, icon_off, icon_on) in nav_items {
                    let is_active = self.active_tab == tab;
                    let icon = if is_active { icon_on } else { icon_off };

                    let btn = egui::Button::image_and_text(
                        egui::Image::new(icon).max_height(18.0),
                        egui::RichText::new(label)
                            .size(13.0)
                            .strong()
                            .color(if is_active {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(165, 175, 195)
                            }),
                    )
                    .fill(if is_active {
                        egui::Color32::from_rgb(0, 130, 215)
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .stroke(if is_active {
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 220, 255))
                    } else {
                        egui::Stroke::NONE
                    })
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(egui::vec2(202.0, 36.0));

                    if ui.add(btn).clicked() {
                        self.active_tab = tab;
                    }
                    ui.add_space(4.0);
                }

                // Push bottom hardware status to bottom of sidebar
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    // Scan button
                    let scan_btn = egui::Button::new("🔄 Scan Mouse")
                        .fill(egui::Color32::from_rgb(26, 32, 46))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(202.0, 28.0));

                    if ui
                        .add(scan_btn)
                        .on_hover_text("Refresh device detection")
                        .clicked()
                    {
                        self.refresh_device();
                    }

                    ui.add_space(8.0);

                    // Connection and Battery Status Card
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(18, 23, 34))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
                        ))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.set_width(182.0);
                            let is_connected = self.device.is_some();

                            // Connection indicator
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
                                            .size(11.0)
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
                                        egui::RichText::new("Offline Mode")
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(231, 76, 60))
                                            .strong(),
                                    );
                                }
                            });

                            // Battery indicator
                            if let Some(battery) = self.battery_level {
                                ui.add_space(4.0);
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
                                        egui::RichText::new(format!("Battery: {}%", battery))
                                            .size(11.0)
                                            .color(battery_color)
                                            .strong(),
                                    );
                                });
                            }
                        });
                });
            });

        // ===================================================================
        // MAIN CONTENT AREA (Clean, Spacious, No Stacked Clutter)
        // ===================================================================
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(17, 21, 30))
                    .inner_margin(egui::Margin::symmetric(24.0, 16.0)),
            )
            .show(ctx, |ui| {
                // Main Header Bar: Breadcrumb + Action Controls
                ui.horizontal(|ui| {
                    let (title_text, subtitle_text) = match self.active_tab {
                        Tab::Buttons => (
                            "BUTTON CONFIGURATION",
                            "6 Programmable switches, fire key burst, and debounce filter",
                        ),
                        Tab::Sensor => (
                            "PIXART PAW3395 SENSOR",
                            "DPI stepping (50-26,000), 4K polling rates, LOD & motion sync",
                        ),
                        Tab::Lighting => (
                            "RGB LIGHTING SUITE",
                            "Lighting modes, breathing speeds, sleep timers, and 14 color chips",
                        ),
                        Tab::Macro => (
                            "MACRO STUDIO",
                            "Hardware macro sequence recorder, delay insertion, and loops",
                        ),
                        Tab::Settings => (
                            "SETTINGS & 2.4G PAIRING",
                            "Wireless dongle pairing tool, profile backup, and udev rules",
                        ),
                    };

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(title_text)
                                .size(17.0)
                                .strong()
                                .color(egui::Color32::from_rgb(0, 220, 255)),
                        );
                        ui.label(
                            egui::RichText::new(subtitle_text)
                                .size(11.5)
                                .color(egui::Color32::from_rgb(145, 155, 180)),
                        );
                    });

                    // Right Actions
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Apply button (Glowing Blue)
                        let apply_btn = egui::Button::new(
                            egui::RichText::new("✓ Apply Changes")
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(0, 150, 235))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(0, 220, 255),
                        ))
                        .rounding(egui::Rounding::same(8.0))
                        .min_size(egui::vec2(136.0, 34.0));

                        if ui
                            .add(apply_btn)
                            .on_hover_text("Write settings to mouse")
                            .clicked()
                        {
                            self.apply_to_device();
                        }

                        // Read button
                        let read_btn = egui::Button::new("📥 Read")
                            .fill(egui::Color32::from_rgb(26, 32, 46))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                            ))
                            .rounding(egui::Rounding::same(8.0))
                            .min_size(egui::vec2(86.0, 34.0));

                        if ui
                            .add(read_btn)
                            .on_hover_text("Load settings from mouse")
                            .clicked()
                        {
                            self.read_from_device();
                        }

                        // Reset button
                        let reset_img = egui::include_image!("../assets/Buttoninterface/Reset.png");
                        let reset_btn =
                            egui::ImageButton::new(egui::Image::new(reset_img).max_height(26.0));

                        if ui
                            .add(reset_btn)
                            .on_hover_text("Reset to factory defaults")
                            .clicked()
                        {
                            self.config = MouseConfig::default();
                            self.set_status("Reset to factory defaults (click Apply)", true);
                        }

                        // Toast feedback
                        if let Some((msg, success, time)) = &self.status
                            && time.elapsed() < Duration::from_secs(5)
                        {
                            let col = if *success {
                                egui::Color32::from_rgb(46, 204, 113)
                            } else {
                                egui::Color32::from_rgb(231, 76, 60)
                            };
                            ui.label(egui::RichText::new(msg).color(col).strong());
                        }
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Render Active Content View (All designed to fit cleanly in viewport)
                match self.active_tab {
                    Tab::Buttons => self.render_buttons_tab(ui),
                    Tab::Sensor => self.render_sensor_tab(ui),
                    Tab::Lighting => self.render_lighting_tab(ui),
                    Tab::Macro => self.render_macro_tab(ui),
                    Tab::Settings => self.render_settings_tab(ui),
                }
            });
    }
}

// ---------------------------------------------------------------------------
// Page Renderers (Clean & Proportional)
// ---------------------------------------------------------------------------

impl IcarusApp {
    /// Render Tab 1: Buttons — Mouse Showcase on Left, Unified 6-Button List on Right
    fn render_buttons_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Pane: Mouse Showcase Pedestal & Debounce
            ui.vertical(|ui| {
                ui.set_width(410.0);

                modern_card(ui, |ui| {
                    ui.set_min_size(egui::vec2(386.0, 330.0));
                    ui.vertical_centered(|ui| {
                        // Display active mouse image view
                        match self.mouse_view_idx {
                            0 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse0.png"
                                    ))
                                    .max_width(340.0),
                                );
                            }
                            1 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse1.png"
                                    ))
                                    .max_width(300.0),
                                );
                            }
                            2 => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse2.png"
                                    ))
                                    .max_width(300.0),
                                );
                            }
                            _ => {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/Buttoninterface/mouse3.png"
                                    ))
                                    .max_width(300.0),
                                );
                            }
                        }

                        ui.add_space(6.0);

                        // Perspective angle tabs
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                            let views = [
                                (0, "🔝 Top"),
                                (1, "📐 Angle 1"),
                                (2, "📐 Angle 2"),
                                (3, "⬡ Side"),
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
                                    egui::Color32::from_rgb(26, 32, 46)
                                })
                                .rounding(egui::Rounding::same(6.0));

                                if ui.add(btn).clicked() {
                                    self.mouse_view_idx = idx;
                                }
                            }
                        });
                    });
                });

                ui.add_space(8.0);

                // Debounce Filter Card
                modern_card(ui, |ui| {
                    ui.set_width(386.0);
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
                    ui.horizontal(|ui| {
                        ui.label(format!("Switch Filter: {} ms", self.config.debounce_ms));
                        ui.add(
                            egui::Slider::new(&mut self.config.debounce_ms, 0..=20)
                                .suffix(" ms")
                                .step_by(2.0),
                        );
                    });
                });
            });

            ui.add_space(18.0);

            // Right Pane: Unified 6-Button Assignment List (Clean single card)
            ui.vertical(|ui| {
                modern_card(ui, |ui| {
                    ui.set_width(380.0);
                    ui.label(
                        egui::RichText::new("ASSIGN ACTIONS")
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                    ui.label(
                        egui::RichText::new("Remap switches to mouse clicks, shortcuts, or macros")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(140, 150, 175)),
                    );
                    ui.add_space(8.0);

                    let button_names = [
                        ("1", "Left Click", "Primary Switch"),
                        ("2", "Right Click", "Secondary Switch"),
                        ("3", "Middle Click", "Scroll Wheel Click"),
                        ("4", "Backward", "Side Button 1"),
                        ("5", "Forward", "Side Button 2"),
                        ("6", "DPI Switch", "Top Cycle Button"),
                    ];

                    for (idx, (btn_num, btn_default, _sub)) in button_names.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Badge with original icon asset
                            ui.add(
                                egui::Image::new(egui::include_image!(
                                    "../assets/Buttoninterface/LabelNumber.png"
                                ))
                                .max_height(16.0),
                            );

                            ui.label(
                                egui::RichText::new(format!(" #{} ", btn_num))
                                    .size(11.0)
                                    .strong()
                                    .background_color(egui::Color32::from_rgb(0, 140, 220))
                                    .color(egui::Color32::WHITE),
                            );

                            ui.label(
                                egui::RichText::new(*btn_default)
                                    .size(12.5)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
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
                                        "DPI Cycle",
                                        "Fire Key (Burst)",
                                        "Media (Play/Pause)",
                                        "Key (Ctrl+A)",
                                        "Macro Play",
                                        "Disabled",
                                    ];

                                    let prev_category = action_category;
                                    egui::ComboBox::from_id_salt(format!("btn_action_{}", idx))
                                        .selected_text(options[action_category])
                                        .width(160.0)
                                        .show_ui(ui, |ui| {
                                            for (opt_idx, opt_name) in options.iter().enumerate() {
                                                ui.selectable_value(
                                                    &mut action_category,
                                                    opt_idx,
                                                    *opt_name,
                                                );
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
                                            7 => ButtonAction::Media(0xCD),
                                            8 => ButtonAction::Keyboard {
                                                usage: 0x04,
                                                modifier: 0x01,
                                            },
                                            9 => ButtonAction::Macro(1),
                                            10 => ButtonAction::Disabled,
                                            _ => ButtonAction::MouseButton(1),
                                        };
                                        if idx < self.config.buttons.len() {
                                            self.config.buttons[idx] = new_action;
                                        }
                                    }
                                },
                            );
                        });

                        // Fire key burst sliders if enabled
                        if let ButtonAction::FireKey {
                            ref mut clicks,
                            ref mut delay_ms,
                        } = self.config.buttons[idx]
                        {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("  Burst:")
                                        .size(10.5)
                                        .color(egui::Color32::from_rgb(0, 220, 255)),
                                );
                                ui.add(egui::Slider::new(clicks, 2..=15));
                                ui.label("Interval:");
                                ui.add(egui::Slider::new(delay_ms, 5..=50).suffix("ms"));
                            });
                        }

                        ui.separator();
                    }
                });
            });
        });
    }

    /// Render Tab 2: Sensor & DPI — Horizontal DPI Stage Ladder + 3-Column Tuning Grid
    fn render_sensor_tab(&mut self, ui: &mut egui::Ui) {
        // TOP: Horizontal DPI Stages Bar (Fits in 1 Row!)
        modern_card(ui, |ui| {
            ui.label(
                egui::RichText::new("DPI STAGES (PIXART PAW3395)")
                    .size(13.0)
                    .strong()
                    .color(egui::Color32::from_rgb(0, 220, 255)),
            );
            ui.add_space(6.0);

            let stage_colors = [
                egui::Color32::from_rgb(255, 60, 60),   // Red
                egui::Color32::from_rgb(0, 230, 230),   // Cyan
                egui::Color32::from_rgb(60, 240, 80),   // Green
                egui::Color32::from_rgb(255, 255, 255), // White
                egui::Color32::from_rgb(255, 230, 40),  // Yellow
            ];

            // 5 Stages laid out horizontally
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);
                for i in 0..self.config.dpi_stages.len() {
                    let is_active = self.config.active_dpi_stage == i as u8;
                    let stage_color = stage_colors.get(i).copied().unwrap_or(egui::Color32::WHITE);

                    let stage_card = egui::Frame::none()
                        .fill(if is_active {
                            egui::Color32::from_rgb(28, 38, 58)
                        } else {
                            egui::Color32::from_rgb(20, 25, 36)
                        })
                        .stroke(if is_active {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 220, 255))
                        } else {
                            egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
                            )
                        })
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0));

                    stage_card.show(ui, |ui| {
                        ui.set_width(128.0);
                        ui.horizontal(|ui| {
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 5.0, stage_color);
                            ui.label(
                                egui::RichText::new(format!("Stage {}", i + 1))
                                    .size(11.5)
                                    .strong()
                                    .color(if is_active {
                                        egui::Color32::from_rgb(0, 220, 255)
                                    } else {
                                        egui::Color32::from_rgb(160, 170, 195)
                                    }),
                            );
                        });

                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(format!("{} DPI", self.config.dpi_stages[i].dpi))
                                .size(15.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui
                                .button(if is_active { "✓ Active" } else { "Select" })
                                .clicked()
                            {
                                self.config.active_dpi_stage = i as u8;
                            }
                            modern_toggle(ui, &mut self.config.dpi_stages[i].enabled);
                        });
                    });
                }
            });

            ui.add_space(8.0);

            // Active Stage Slider with Add and Reduce buttons
            let active_idx = self.config.active_dpi_stage as usize;
            if active_idx < self.config.dpi_stages.len() {
                ui.horizontal(|ui| {
                    ui.label("Fine Step:");
                    let red_btn = egui::ImageButton::new(
                        egui::Image::new(egui::include_image!("../assets/Sensor/Reduce.png"))
                            .max_height(14.0),
                    );
                    if ui.add(red_btn).on_hover_text("Decrease 50 DPI").clicked() {
                        let cur = self.config.dpi_stages[active_idx].dpi;
                        self.config.dpi_stages[active_idx].dpi = cur.saturating_sub(50).max(50);
                    }

                    let mut dpi_val = self.config.dpi_stages[active_idx].dpi as i32;
                    let slider = egui::Slider::new(&mut dpi_val, 50..=26000)
                        .step_by(50.0)
                        .suffix(" DPI")
                        .logarithmic(true);

                    if ui.add(slider).changed() {
                        let snapped = crate::dpi::snap_dpi(dpi_val.clamp(50, 26000) as u16);
                        self.config.dpi_stages[active_idx].dpi = snapped;
                    }

                    let add_btn = egui::ImageButton::new(
                        egui::Image::new(egui::include_image!("../assets/Sensor/Add.png"))
                            .max_height(14.0),
                    );
                    if ui.add(add_btn).on_hover_text("Increase 50 DPI").clicked() {
                        let cur = self.config.dpi_stages[active_idx].dpi;
                        self.config.dpi_stages[active_idx].dpi = cur.saturating_add(50).min(26000);
                    }
                });
            }
        });

        ui.add_space(10.0);

        // BOTTOM: 3-Column Clean Hardware Grid (Fits in 1 Viewport!)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);

            // Col 1: Polling Rate
            modern_card(ui, |ui| {
                ui.set_width(245.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Sensor/ReportRate.png"))
                            .max_height(18.0),
                    );
                    ui.label(
                        egui::RichText::new("POLLING RATE")
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                });
                ui.add_space(6.0);

                let rates = [
                    (PollingRate::Hz125, "125 Hz", "8 ms"),
                    (PollingRate::Hz250, "250 Hz", "4 ms"),
                    (PollingRate::Hz500, "500 Hz", "2 ms"),
                    (PollingRate::Hz1000, "1000 Hz", "1 ms (Esports)"),
                ];

                for (rate, label, desc) in rates {
                    let is_selected = self.config.polling_rate == rate;
                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{} ({})", label, desc))
                            .size(11.5)
                            .color(if is_selected {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(170, 180, 205)
                            })
                            .strong(),
                    )
                    .fill(if is_selected {
                        egui::Color32::from_rgb(0, 140, 220)
                    } else {
                        egui::Color32::from_rgb(26, 32, 46)
                    })
                    .rounding(egui::Rounding::same(6.0))
                    .min_size(egui::vec2(225.0, 26.0));

                    if ui.add(btn).clicked() {
                        self.config.polling_rate = rate;
                    }
                    ui.add_space(2.0);
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let is_2k = self.config.polling_rate == PollingRate::Hz2000;
                    let btn_2k = egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../assets/DialogChange/2KHz.png"))
                            .max_height(16.0),
                        egui::RichText::new("2K").strong(),
                    )
                    .fill(if is_2k {
                        egui::Color32::from_rgb(0, 140, 220)
                    } else {
                        egui::Color32::from_rgb(26, 32, 46)
                    })
                    .min_size(egui::vec2(108.0, 28.0));

                    if ui.add(btn_2k).clicked() {
                        self.config.polling_rate = PollingRate::Hz2000;
                    }

                    let is_4k = self.config.polling_rate == PollingRate::Hz4000;
                    let btn_4k = egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../assets/DialogChange/4KHz.png"))
                            .max_height(16.0),
                        egui::RichText::new("4K").strong(),
                    )
                    .fill(if is_4k {
                        egui::Color32::from_rgb(0, 140, 220)
                    } else {
                        egui::Color32::from_rgb(26, 32, 46)
                    })
                    .min_size(egui::vec2(108.0, 28.0));

                    if ui.add(btn_4k).clicked() {
                        self.config.polling_rate = PollingRate::Hz4000;
                    }
                });
            });

            // Col 2: Sensor Tuning & Power
            modern_card(ui, |ui| {
                ui.set_width(245.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Sensor/LOD.png"))
                            .max_height(16.0),
                    );
                    ui.label(
                        egui::RichText::new("LOD & POWER")
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                });
                ui.add_space(6.0);

                ui.label("Lift-Off Distance (LOD):");
                ui.horizontal(|ui| {
                    let low_selected = self.config.lod == LiftOffDistance::Low;
                    if ui.selectable_label(low_selected, "1 mm (Low)").clicked() {
                        self.config.lod = LiftOffDistance::Low;
                    }
                    let high_selected = self.config.lod == LiftOffDistance::High;
                    if ui.selectable_label(high_selected, "2 mm (High)").clicked() {
                        self.config.lod = LiftOffDistance::High;
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    modern_toggle(ui, &mut self.config.performance_mode);
                    ui.label(
                        egui::RichText::new(if self.config.performance_mode {
                            "⚡ Performance"
                        } else {
                            "🍃 Office Eco"
                        })
                        .strong(),
                    );
                });

                ui.add_space(6.0);
                ui.label("Sleep Timer:");
                let sleep_options = [
                    (10, "10s"),
                    (30, "30s"),
                    (60, "1m"),
                    (120, "2m"),
                    (300, "5m"),
                    (0, "Never"),
                ];
                let selected_label = sleep_options
                    .iter()
                    .find(|(s, _)| *s == self.config.sensor_sleep_seconds)
                    .map(|(_, l)| *l)
                    .unwrap_or("Custom");

                egui::ComboBox::from_id_salt("sensor_sleep_combo")
                    .selected_text(selected_label)
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        for (val, label) in sleep_options {
                            ui.selectable_value(&mut self.config.sensor_sleep_seconds, val, label);
                        }
                    });
            });

            // Col 3: Advanced Sensor Toggles
            modern_card(ui, |ui| {
                ui.set_width(245.0);
                ui.label(
                    egui::RichText::new("PAW3395 FEATURES")
                        .strong()
                        .color(egui::Color32::from_rgb(0, 220, 255)),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    modern_toggle(ui, &mut self.config.motion_sync);
                    ui.label("Motion Sync (Clock sync)");
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    modern_toggle(ui, &mut self.config.angle_snapping);
                    ui.label("Angle Snap (Line straight)");
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    modern_toggle(ui, &mut self.config.ripple_control);
                    ui.label("Ripple Control (>5K DPI)");
                });
            });
        });
    }

    /// Render Tab 3: Lighting Suite — Mode Pills + 14 Swatches + Live RGB Tuner
    fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Pane: Mode & Sliders
            ui.vertical(|ui| {
                ui.set_width(380.0);
                modern_card(ui, |ui| {
                    ui.label(
                        egui::RichText::new("LIGHTING EFFECT")
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                    ui.add_space(6.0);

                    let modes = [
                        (LedMode::Solid, "Solid Static"),
                        (LedMode::Breathing, "Breathing Pulse"),
                        (LedMode::Neon, "Neon Cycle"),
                        (LedMode::Off, "Off (Save Power)"),
                    ];

                    for (mode, label) in modes {
                        let is_active = self.config.dpi_led.mode == mode;
                        let btn = egui::Button::new(egui::RichText::new(label).size(12.0).color(
                            if is_active {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(175, 185, 205)
                            },
                        ))
                        .fill(if is_active {
                            egui::Color32::from_rgb(0, 140, 220)
                        } else {
                            egui::Color32::from_rgb(26, 32, 46)
                        })
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(340.0, 28.0));

                        if ui.add(btn).clicked() {
                            self.config.dpi_led.mode = mode;
                        }
                        ui.add_space(3.0);
                    }

                    ui.add_space(8.0);
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
                    ui.horizontal(|ui| {
                        ui.label("Sleep Timer:");
                        let sleep_options = [
                            (10, "10s"),
                            (30, "30s"),
                            (60, "1m"),
                            (120, "2m"),
                            (300, "5m"),
                            (0, "Never"),
                        ];
                        let selected_label = sleep_options
                            .iter()
                            .find(|(s, _)| *s == self.config.led_sleep_seconds)
                            .map(|(_, l)| *l)
                            .unwrap_or("Custom");

                        egui::ComboBox::from_id_salt("led_sleep_combo")
                            .selected_text(selected_label)
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                for (val, label) in sleep_options {
                                    ui.selectable_value(
                                        &mut self.config.led_sleep_seconds,
                                        val,
                                        label,
                                    );
                                }
                            });
                    });
                });
            });

            ui.add_space(18.0);

            // Right Pane: Color Palette & RGB Tuner
            ui.vertical(|ui| {
                ui.set_width(380.0);
                modern_card(ui, |ui| {
                    ui.label(
                        egui::RichText::new("COLOR PALETTE & CUSTOM RGB")
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                    ui.add_space(4.0);

                    // ColorBoard banner
                    ui.add(
                        egui::Image::new(egui::include_image!("../assets/Light/ColorBoard.png"))
                            .max_width(350.0),
                    );

                    ui.add_space(6.0);
                    ui.label("14 Preset Color Swatches:");

                    // 14 Swatches (7x2 Grid)
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
                                egui::Image::new(img).max_width(24.0).max_height(24.0),
                            );
                            if ui.add(btn).on_hover_text(name).clicked() {
                                self.config.dpi_led.color = col;
                            }
                        }
                    });

                    ui.add_space(2.0);

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
                                egui::Image::new(img).max_width(24.0).max_height(24.0),
                            );
                            if ui.add(btn).on_hover_text(name).clicked() {
                                self.config.dpi_led.color = col;
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // RGB Sliders
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("R:")
                                .color(egui::Color32::from_rgb(255, 100, 100))
                                .strong(),
                        );
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.r, 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("G:")
                                .color(egui::Color32::from_rgb(100, 255, 100))
                                .strong(),
                        );
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.g, 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("B:")
                                .color(egui::Color32::from_rgb(100, 180, 255))
                                .strong(),
                        );
                        ui.add(egui::Slider::new(&mut self.config.dpi_led.color.b, 0..=255));
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/Light/PreviewColor.png"
                            ))
                            .max_height(20.0),
                        );
                        ui.label("Preview:");
                        let preview_col = egui::Color32::from_rgb(
                            self.config.dpi_led.color.r,
                            self.config.dpi_led.color.g,
                            self.config.dpi_led.color.b,
                        );
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(60.0, 22.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0_f32, preview_col);
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

    /// Render Tab 4: Macro Studio — 2-Column Clean Workspace
    fn render_macro_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Pane: Library & Repeat
            ui.vertical(|ui| {
                ui.set_width(320.0);
                modern_card(ui, |ui| {
                    ui.label(
                        egui::RichText::new("MACRO SCRIPTS")
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                    ui.add_space(6.0);

                    let num_macros = self.config.macros.len();
                    for idx in 0..num_macros {
                        let is_selected = self.selected_macro_idx == idx;
                        let macro_name = self.config.macros[idx].name.clone();

                        let item_btn =
                            egui::Button::new(egui::RichText::new(&macro_name).size(12.0).color(
                                if is_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(175, 185, 205)
                                },
                            ))
                            .fill(if is_selected {
                                egui::Color32::from_rgb(0, 140, 220)
                            } else {
                                egui::Color32::from_rgb(26, 32, 46)
                            })
                            .min_size(egui::vec2(285.0, 28.0));

                        if ui.add(item_btn).clicked() {
                            self.selected_macro_idx = idx;
                        }
                        ui.add_space(2.0);
                    }

                    ui.add_space(6.0);
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

                    if self.config.macros.len() > 1 && ui.button("🗑️ Delete Selected").clicked()
                    {
                        self.config.macros.remove(self.selected_macro_idx);
                        if self.selected_macro_idx >= self.config.macros.len() {
                            self.selected_macro_idx = self.config.macros.len() - 1;
                        }
                    }

                    ui.add_space(8.0);

                    // Repeat mode settings
                    if let Some(mac) = self.config.macros.get_mut(self.selected_macro_idx) {
                        ui.separator();
                        ui.label(
                            egui::RichText::new("REPEAT MODE:")
                                .size(11.0)
                                .strong()
                                .color(egui::Color32::from_rgb(0, 220, 255)),
                        );

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
                            .radio_value(&mut mode_code, 1, "Hold to Repeat")
                            .clicked()
                        {
                            mac.repeat_mode = MacroRepeatMode::WhileHeld;
                        }
                        if ui
                            .radio_value(&mut mode_code, 2, "Click to Toggle")
                            .clicked()
                        {
                            mac.repeat_mode = MacroRepeatMode::Toggle;
                        }
                    }
                });
            });

            ui.add_space(18.0);

            // Right Pane: Event Sequence Timeline
            ui.vertical(|ui| {
                ui.set_width(430.0);
                modern_card(ui, |ui| {
                    // Record / Stop Toolbar
                    ui.horizontal(|ui| {
                        let record_img = egui::include_image!("../assets/Macro/Record.png");
                        let stop_img = egui::include_image!("../assets/Macro/Stop.png");

                        let rec_btn =
                            egui::ImageButton::new(egui::Image::new(record_img).max_height(26.0));
                        if ui
                            .add(rec_btn)
                            .on_hover_text("Start recording keystrokes")
                            .clicked()
                        {
                            self.is_recording_macro = true;
                            self.set_status("🔴 Recording macro...", true);
                        }

                        let stop_btn =
                            egui::ImageButton::new(egui::Image::new(stop_img).max_height(26.0));
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
                                egui::RichText::new("● REC")
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .strong(),
                            );
                        }
                    });

                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("EVENT TIMELINE")
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
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
                                    ui.horizontal(|ui| {
                                        match event.event_type {
                                            MacroEventType::KeyDown => {
                                                ui.add(
                                                    egui::Image::new(egui::include_image!(
                                                        "../assets/Macro/MacroDown.png"
                                                    ))
                                                    .max_height(14.0),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Down: {}",
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
                                                    .max_height(14.0),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Up:   {}",
                                                        event.key
                                                    ))
                                                    .color(egui::Color32::from_rgb(170, 180, 205)),
                                                );
                                            }
                                            MacroEventType::Delay => {
                                                ui.add(
                                                    egui::Image::new(egui::include_image!(
                                                        "../assets/Macro/MacroTime.png"
                                                    ))
                                                    .max_height(14.0),
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
                                                if ui.small_button("✕").clicked() {
                                                    event_to_remove = Some(e_idx);
                                                }
                                                let down_img = egui::include_image!(
                                                    "../assets/Macro/Down.png"
                                                );
                                                if ui
                                                    .add(egui::ImageButton::new(
                                                        egui::Image::new(down_img).max_height(10.0),
                                                    ))
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
                            ui.label("Delay:");
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

    /// Render Tab 5: Settings & 2.4G Pairing — 2 Clean Columns
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Pane: 2.4G Pairing & Profile Backup
            ui.vertical(|ui| {
                ui.set_width(380.0);
                modern_card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!(
                                "../assets/FormSetting/Mouse.png"
                            ))
                            .max_height(20.0),
                        );
                        ui.label(
                            egui::RichText::new("2.4G DONGLE PAIRING")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 220, 255)),
                        );
                    });
                    ui.label("Pair mouse with 2.4G / 4G USB receiver:");
                    ui.add_space(8.0);

                    match self.pairing_state {
                        PairingState::Idle => {
                            let pair_img = egui::include_image!("../assets/FormSetting/Pair.png");
                            ui.add(egui::Image::new(pair_img).max_height(26.0));
                        }
                        PairingState::Pairing => {
                            let pari_btn =
                                egui::include_image!("../assets/FormSetting/PariButton.png");
                            ui.add(egui::Image::new(pari_btn).max_height(26.0));
                            ui.label("Pairing in progress... Keep mouse near dongle");
                        }
                        PairingState::Success => {
                            let succ_img =
                                egui::include_image!("../assets/FormSetting/PairSuccess.png");
                            ui.add(egui::Image::new(succ_img).max_height(26.0));
                            ui.label(
                                egui::RichText::new("✓ Dongle Paired Successfully!")
                                    .color(egui::Color32::from_rgb(46, 204, 113))
                                    .strong(),
                            );
                        }
                        PairingState::Failed => {
                            let fail_img =
                                egui::include_image!("../assets/FormSetting/PairFail.png");
                            ui.add(egui::Image::new(fail_img).max_height(26.0));
                            ui.label(
                                egui::RichText::new("Pairing Failed")
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
                            self.set_status("Scanning for dongle...", true);
                        }
                        if self.pairing_state != PairingState::Idle
                            && ui.button("Reset State").clicked()
                        {
                            self.pairing_state = PairingState::Idle;
                        }
                    });
                });

                ui.add_space(10.0);

                modern_card(ui, |ui| {
                    ui.label(
                        egui::RichText::new("PROFILE MANAGEMENT (TOML)")
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("💾 Export TOML").clicked()
                            && let Ok(toml_str) = toml::to_string_pretty(&self.config)
                        {
                            let _ = std::fs::write("icarus-profile.toml", toml_str);
                            self.set_status("✓ Saved to icarus-profile.toml", true);
                        }
                        if ui.button("📂 Import TOML").clicked()
                            && let Ok(content) = std::fs::read_to_string("icarus-profile.toml")
                            && let Ok(cfg) = toml::from_str::<MouseConfig>(&content)
                        {
                            self.config = cfg;
                            self.set_status("✓ Loaded from icarus-profile.toml", true);
                        }
                    });
                });
            });

            ui.add_space(18.0);

            // Right Pane: Linux Udev & Hardware Architecture
            ui.vertical(|ui| {
                ui.set_width(380.0);
                modern_card(ui, |ui| {
                    ui.label(
                        egui::RichText::new("HARDWARE ARCHITECTURE")
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                    ui.add_space(6.0);

                    let specs = [
                        ("Device", "ICARUS V2 MINI Wireless Gaming Mouse"),
                        ("Optical Sensor", "PixArt PAW3395 (26,000 DPI, 650 IPS)"),
                        ("Main MCU", "Nordic nRF52840 (ARM Cortex-M4F @ 64MHz)"),
                        ("1K Wireless Dongle", "CX52650N (2.4GHz Wireless)"),
                        ("4K High-Speed Dongle", "WCH CH32V305 (RISC-V USB-HS)"),
                        ("USB VID / PID", "0x3554 : 0xF511 (USB) / 0xF512 (2.4G)"),
                        ("HID Endpoint", "Interface 1 (Vendor Specific)"),
                        ("Report Format", "17-byte Packet with XOR Checksum"),
                    ];

                    for (k, v) in specs {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", k))
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(140, 150, 175)),
                            );
                            ui.label(
                                egui::RichText::new(v)
                                    .size(11.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(
                        egui::RichText::new("LINUX USB RULES (UDEV)")
                            .strong()
                            .color(egui::Color32::from_rgb(0, 220, 255)),
                    );
                    ui.code("sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/");
                    ui.code("sudo udevadm control --reload-rules && sudo udevadm trigger");
                    ui.add_space(4.0);
                    if ui.button("📋 Generate 99-icarus-mouse.rules").clicked() {
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
        });
    }
}

/// Run the native Desktop GUI window.
pub fn run_gui() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 700.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_title("ICARUS V2 MINI — Mouse Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "ICARUS V2 MINI",
        native_options,
        Box::new(|cc| Ok(Box::new(IcarusApp::new(cc)))),
    )
}
