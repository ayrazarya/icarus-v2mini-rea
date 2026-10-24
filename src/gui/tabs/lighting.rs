use std::time::Duration;

use eframe::egui;

use crate::config::{Color, LedMode};
use crate::gui::state::IcarusApp;
use crate::gui::theme::{hsv_to_rgb, toggle_switch};

impl IcarusApp {
    /// Render Tab 3: Lighting, Live Animated LED Illumination Preview, 6 Esports Presets & 14 Color Chips
    pub(crate) fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        // Request repaint after 16ms (~60 FPS) if animated mode is running, preventing 100% CPU spinning
        if self.config.dpi_led.mode == LedMode::Breathing
            || self.config.dpi_led.mode == LedMode::Neon
        {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }

        // Banner explaining that mouse lighting indicates active DPI stage
        let banner_frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(240, 249, 255))
            .stroke(egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgb(186, 230, 253),
            ))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 7.0));

        banner_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("DPI Indicator Lamp:")
                        .strong()
                        .color(egui::Color32::from_rgb(2, 132, 199)),
                );
                ui.label(
                    egui::RichText::new(
                        "On ICARUS V2 MINI, the mouse LED reflects the active DPI stage. Choosing a color assigns it to the active stage.",
                    )
                    .size(11.5)
                    .color(egui::Color32::from_rgb(51, 65, 85)),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Target DPI Stage:").size(11.5).color(egui::Color32::from_rgb(100, 116, 139)));
                let num_stages = self.config.dpi_stages.len();
                for s_idx in 0..num_stages {
                    let is_active = self.config.active_dpi_stage == s_idx as u8;
                    let (scol, dpi, col) = {
                        let s = &self.config.dpi_stages[s_idx];
                        (
                            egui::Color32::from_rgb(s.color.r, s.color.g, s.color.b),
                            s.dpi,
                            s.color,
                        )
                    };
                    let btn = egui::Button::new(
                        egui::RichText::new(format!("Stage {} ({} DPI)", s_idx + 1, dpi))
                            .size(11.0)
                            .strong()
                            .color(if is_active {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(30, 41, 59)
                            }),
                    )
                    .fill(if is_active {
                        egui::Color32::from_rgb(2, 132, 199)
                    } else {
                        egui::Color32::from_rgb(255, 255, 255)
                    })
                    .stroke(egui::Stroke::new(1.5_f32, scol))
                    .rounding(egui::Rounding::same(4.0));
                    if ui.add(btn).clicked() {
                        self.config.active_dpi_stage = s_idx as u8;
                        self.config.dpi_led.color = col;
                        if self.auto_apply_lighting {
                            self.apply_lighting_to_device();
                        }
                    }
                }
            });
        });

        ui.add_space(10.0);

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

                // Live Auto-Apply Toggle Switch
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let mut auto_apply = self.auto_apply_lighting;
                    if toggle_switch(ui, &mut auto_apply).changed() {
                        self.auto_apply_lighting = auto_apply;
                        if auto_apply {
                            self.apply_lighting_to_device();
                            self.set_status("Live Auto-Apply on Mouse enabled", true);
                        }
                    }
                    ui.label(
                        egui::RichText::new("Live Auto-Apply on Mouse")
                            .size(12.0)
                            .strong()
                            .color(if self.auto_apply_lighting {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(71, 85, 105)
                            }),
                    ).on_hover_text("When enabled, lighting changes (modes, colors, brightness, speed) are sent directly to your mouse hardware in real time without clicking Apply Changes.");
                });

                ui.add_space(8.0);

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
                            if self.auto_apply_lighting {
                                self.apply_lighting_to_device();
                            }
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
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.dpi_led.brightness,
                            0..=10,
                        ))
                        .changed()
                        && self.auto_apply_lighting
                    {
                        self.queue_lighting_sync();
                    }

                    ui.add_space(6.0);

                    ui.label(format!(
                        "Breathing Speed: {}",
                        self.config.dpi_led.breath_speed
                    ));
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.dpi_led.breath_speed,
                            1..=10,
                        ))
                        .changed()
                        && self.auto_apply_lighting
                    {
                        self.queue_lighting_sync();
                    }

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
                                if ui
                                    .selectable_value(&mut self.config.led_sleep_seconds, val, label)
                                    .clicked()
                                    && self.auto_apply_lighting
                                {
                                    self.apply_lighting_to_device();
                                }
                            }
                        });
                });

                ui.add_space(10.0);

                // 4K Wireless Receiver Dongle RGB & Indicator Bar Card
                ui.group(|ui| {
                    ui.set_width(360.0);
                    ui.heading(
                        egui::RichText::new("4K Wireless Dongle RGB")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Illumination on 4K high-speed wireless receiver:");
                    ui.add_space(4.0);

                    // Mode selection
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Mode:").size(11.5).strong());
                        let dongle_modes = [
                            (LedMode::Off, "Off"),
                            (LedMode::Solid, "Static / Solid"),
                            (LedMode::Breathing, "Breathing"),
                            (LedMode::Neon, "Neon Rainbow"),
                        ];
                        let cur_dongle_mode_label = dongle_modes
                            .iter()
                            .find(|(m, _)| *m == self.config.dongle_rgb.mode)
                            .map(|(_, l)| *l)
                            .unwrap_or("Custom");

                        egui::ComboBox::from_id_salt("dongle_rgb_mode_combo")
                            .selected_text(cur_dongle_mode_label)
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                for (val, label) in dongle_modes {
                                    if ui
                                        .selectable_value(&mut self.config.dongle_rgb.mode, val, label)
                                        .clicked()
                                    {
                                        self.config.dongle_rgb.enabled = val != LedMode::Off;
                                        if self.auto_apply_lighting {
                                            self.apply_dongle_to_device();
                                        }
                                    }
                                }
                            });
                    });

                    // Color picker
                    if self.config.dongle_rgb.mode == LedMode::Solid
                        || self.config.dongle_rgb.mode == LedMode::Breathing
                    {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Color:").size(11.5).strong());
                            let mut rgb = [
                                self.config.dongle_rgb.color.r,
                                self.config.dongle_rgb.color.g,
                                self.config.dongle_rgb.color.b,
                            ];
                            if ui
                                .color_edit_button_srgb(&mut rgb)
                                .on_hover_text("Choose color for 4K wireless dongle")
                                .changed()
                            {
                                self.config.dongle_rgb.color = Color::new(rgb[0], rgb[1], rgb[2]);
                                if self.auto_apply_lighting {
                                    self.queue_dongle_sync();
                                }
                            }
                        });
                    }

                    // Brightness slider
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Brightness:").size(11.5));
                        if ui
                            .add(
                                egui::Slider::new(&mut self.config.dongle_rgb.brightness, 0..=10)
                                    .suffix("/10"),
                            )
                            .changed()
                            && self.auto_apply_lighting
                        {
                            self.queue_dongle_sync();
                        }
                    });

                    // Speed slider
                    if self.config.dongle_rgb.mode == LedMode::Breathing
                        || self.config.dongle_rgb.mode == LedMode::Neon
                    {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Effect Speed:").size(11.5));
                            if ui
                                .add(egui::Slider::new(&mut self.config.dongle_rgb.speed, 1..=10))
                                .changed()
                                && self.auto_apply_lighting
                            {
                                self.queue_dongle_sync();
                            }
                        });
                    }

                    // Dongle indicator bar mode
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Bar Mode:").size(11.5).strong());
                        let bar_modes = [
                            (0u8, "Sync with RGB Effect"),
                            (1u8, "Real-Time Battery Status"),
                            (2u8, "Fixed Color"),
                        ];
                        let cur_bar_label = bar_modes
                            .iter()
                            .find(|(m, _)| *m == self.config.dongle_rgb.bar_mode)
                            .map(|(_, l)| *l)
                            .unwrap_or("Sync");

                        egui::ComboBox::from_id_salt("dongle_bar_mode_combo")
                            .selected_text(cur_bar_label)
                            .width(170.0)
                            .show_ui(ui, |ui| {
                                for (val, label) in bar_modes {
                                    if ui
                                        .selectable_value(&mut self.config.dongle_rgb.bar_mode, val, label)
                                        .clicked()
                                        && self.auto_apply_lighting
                                    {
                                        self.apply_dongle_to_device();
                                    }
                                }
                            });
                    });

                    ui.add_space(6.0);
                    let apply_dongle_btn = egui::Button::new(
                        egui::RichText::new("Apply Dongle RGB")
                            .size(11.0)
                            .strong()
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    )
                    .fill(egui::Color32::from_rgb(240, 249, 255))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(186, 230, 253)))
                    .rounding(egui::Rounding::same(4.0));

                    if ui.add(apply_dongle_btn).clicked() {
                        if let Some(ref dev) = self.device {
                            match dev.apply_dongle_rgb(&self.config.dongle_rgb) {
                                Ok(()) => self.set_status("4K Dongle RGB applied!", true),
                                Err(e) => self.set_status(format!("Dongle RGB error: {}", e), false),
                            }
                        } else {
                            self.set_status("Dongle not connected", false);
                        }
                    }
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
                                    self.set_active_lighting_color(*p_col);
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
                        egui::Image::new(egui::include_image!("../../../assets/Light/ColorBoard.png"))
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
                                egui::include_image!("../../../assets/Light/ColorA.png"),
                                Color::new(255, 0, 0),
                                "Color A: Red",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorB.png"),
                                Color::new(0, 255, 255),
                                "Color B: Cyan",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorC.png"),
                                Color::new(0, 255, 0),
                                "Color C: Green",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorD.png"),
                                Color::new(255, 0, 255),
                                "Color D: Magenta",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorE.png"),
                                Color::new(255, 255, 0),
                                "Color E: Yellow",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorF.png"),
                                Color::new(255, 80, 0),
                                "Color F: Orange",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorG.png"),
                                Color::new(0, 0, 255),
                                "Color G: Blue",
                            ),
                        ];

                        for (img, col, name) in swatches_row1 {
                            let btn = egui::ImageButton::new(
                                egui::Image::new(img).max_width(26.0).max_height(26.0),
                            );
                            if ui.add(btn).on_hover_text(name).clicked() {
                                self.set_active_lighting_color(col);
                            }
                        }
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        let swatches_row2 = [
                            (
                                egui::include_image!("../../../assets/Light/ColorH.png"),
                                Color::new(227, 0, 127),
                                "Color H: Pink",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorI.png"),
                                Color::new(46, 166, 223),
                                "Color I: Sky",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorJ.png"),
                                Color::new(96, 25, 133),
                                "Color J: Purple",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorK.png"),
                                Color::new(255, 255, 255),
                                "Color K: White",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorL.png"),
                                Color::new(139, 178, 36),
                                "Color L: Lime",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorM.png"),
                                Color::new(255, 155, 51),
                                "Color M: Amber",
                            ),
                            (
                                egui::include_image!("../../../assets/Light/ColorN.png"),
                                Color::new(208, 113, 173),
                                "Color N: Orchid",
                            ),
                        ];

                        for (img, col, name) in swatches_row2 {
                            let btn = egui::ImageButton::new(
                                egui::Image::new(img).max_width(26.0).max_height(26.0),
                            );
                            if ui.add(btn).on_hover_text(name).clicked() {
                                self.set_active_lighting_color(col);
                            }
                        }
                    });

                    ui.add_space(12.0);
                    ui.label("Fine Tuning (Custom RGB Sliders):");

                    let mut color_changed = false;

                    // Red slider with Add/Reduce buttons
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("-5").strong()).clicked() {
                            self.config.dpi_led.color.r =
                                self.config.dpi_led.color.r.saturating_sub(5);
                            color_changed = true;
                        }

                        ui.label("Red:  ");
                        if ui.add(egui::Slider::new(&mut self.config.dpi_led.color.r, 0..=255)).changed() {
                            color_changed = true;
                        }

                        if ui.button(egui::RichText::new("+5").strong()).clicked() {
                            self.config.dpi_led.color.r =
                                self.config.dpi_led.color.r.saturating_add(5);
                            color_changed = true;
                        }
                    });

                    // Green slider with Add/Reduce buttons
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("-5").strong()).clicked() {
                            self.config.dpi_led.color.g =
                                self.config.dpi_led.color.g.saturating_sub(5);
                            color_changed = true;
                        }

                        ui.label("Green:");
                        if ui.add(egui::Slider::new(&mut self.config.dpi_led.color.g, 0..=255)).changed() {
                            color_changed = true;
                        }

                        if ui.button(egui::RichText::new("+5").strong()).clicked() {
                            self.config.dpi_led.color.g =
                                self.config.dpi_led.color.g.saturating_add(5);
                            color_changed = true;
                        }
                    });

                    // Blue slider with Add/Reduce buttons
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("-5").strong()).clicked() {
                            self.config.dpi_led.color.b =
                                self.config.dpi_led.color.b.saturating_sub(5);
                            color_changed = true;
                        }

                        ui.label("Blue: ");
                        if ui.add(egui::Slider::new(&mut self.config.dpi_led.color.b, 0..=255)).changed() {
                            color_changed = true;
                        }

                        if ui.button(egui::RichText::new("+5").strong()).clicked() {
                            self.config.dpi_led.color.b =
                                self.config.dpi_led.color.b.saturating_add(5);
                            color_changed = true;
                        }
                    });

                    // Sync fine-tuned slider RGB to the active DPI stage
                    if color_changed {
                        let col = self.config.dpi_led.color;
                        let active_idx = (self.config.active_dpi_stage as usize)
                            .min(self.config.dpi_stages.len().saturating_sub(1));
                        if active_idx < self.config.dpi_stages.len() {
                            self.config.dpi_stages[active_idx].color = col;
                        }
                        if self.auto_apply_lighting {
                            self.queue_lighting_sync();
                        }
                    }

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

    pub(crate) fn set_active_lighting_color(&mut self, col: Color) {
        self.config.dpi_led.color = col;
        let active_idx = (self.config.active_dpi_stage as usize)
            .min(self.config.dpi_stages.len().saturating_sub(1));
        if active_idx < self.config.dpi_stages.len() {
            self.config.dpi_stages[active_idx].color = col;
        }
        if self.auto_apply_lighting {
            self.apply_lighting_to_device();
        }
    }
}
