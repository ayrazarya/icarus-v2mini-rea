use eframe::egui;

use crate::config::{Color, LiftOffDistance, PollingRate};
use crate::gui::state::IcarusApp;
use crate::gui::theme::toggle_switch;

/// Precomputed `ln(50.0)` and `ln(26000.0)` for logarithmic DPI spectrum mapping.
const MIN_DPI_LN: f32 = 3.912_023;
const MAX_DPI_LN: f32 = 10.165_852;
const DPI_LN_SPAN: f32 = MAX_DPI_LN - MIN_DPI_LN;

/// Zero-allocation parser for extracting an integer from ASCII digits in a string slice.
#[inline]
fn parse_ascii_digits_u32(s: &str) -> Option<u32> {
    let mut acc = 0u32;
    let mut has_digit = false;
    for b in s.bytes().filter(|b| b.is_ascii_digit()) {
        has_digit = true;
        acc = acc.saturating_mul(10).saturating_add((b - b'0') as u32);
    }
    has_digit.then_some(acc)
}

impl IcarusApp {
    /// Render Tab 2: Sensor, Polling Rate, 2K/4K Modes, Continuous DPI Visualizer and Stages
    pub(crate) fn render_sensor_tab(&mut self, ui: &mut egui::Ui) {
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

                    // Long Range Wireless Mode toggle (RF transmission power boost)
                    ui.horizontal(|ui| {
                        toggle_switch(ui, &mut self.config.long_range_mode);
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Long Range Wireless Mode (High RF Power)")
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

                // Helper to map DPI to x position logarithmically using precomputed log bounds
                let dpi_to_x = |dpi: u16| -> f32 {
                    let cur_ln = (dpi.clamp(50, 26000) as f32).ln();
                    let t = (cur_ln - MIN_DPI_LN) / DPI_LN_SPAN;
                    rail_left + rail_width * t.clamp(0.0, 1.0)
                };

                // Interactive Ticks for key milestones
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
                    let tick_rect = egui::Rect::from_center_size(
                        egui::pos2(x, rail_y + 11.0),
                        egui::vec2(28.0, 18.0),
                    );
                    let tick_resp = ui.interact(
                        tick_rect,
                        ui.id().with(("dpi_milestone", d)),
                        egui::Sense::click(),
                    );
                    let is_hovered = tick_resp.hovered();

                    ui.painter().line_segment(
                        [egui::pos2(x, rail_y - 3.0), egui::pos2(x, rail_y + 3.0)],
                        egui::Stroke::new(
                            if is_hovered { 1.5_f32 } else { 1.0_f32 },
                            if is_hovered {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(148, 163, 184)
                            },
                        ),
                    );

                    if is_hovered {
                        ui.painter().rect_filled(
                            tick_rect,
                            3.0,
                            egui::Color32::from_rgb(224, 242, 254),
                        );
                    }

                    ui.painter().text(
                        egui::pos2(x, rail_y + 11.0),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(9.5),
                        if is_hovered {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(100, 116, 139)
                        },
                    );

                    let act_idx = (self.config.active_dpi_stage as usize)
                        .min(self.config.dpi_stages.len().saturating_sub(1));
                    let tick_resp = tick_resp.on_hover_text(format!(
                        "Click to set active stage (Stage {}) to {} DPI",
                        act_idx + 1,
                        d
                    ));

                    if tick_resp.clicked() && act_idx < self.config.dpi_stages.len() {
                        self.config.dpi_stages[act_idx].dpi = d;
                        if self.dpi_text_inputs.len() > act_idx {
                            self.dpi_text_inputs[act_idx] = d.to_string();
                        }
                        if self.config.separate_xy_dpi {
                            self.config.dpi_stages[act_idx].dpi_y = Some(d);
                            if self.dpi_y_text_inputs.len() > act_idx {
                                self.dpi_y_text_inputs[act_idx] = d.to_string();
                            }
                        }
                        self.set_status(format!("Stage {} set to {} DPI", act_idx + 1, d), true);
                    }
                }

                // Paint Stage markers
                for (s_idx, s) in self.config.dpi_stages.iter().enumerate() {
                    if !s.enabled {
                        continue;
                    }
                    let sx = dpi_to_x(s.dpi);
                    let pt = egui::pos2(sx, rail_y);
                    let scol = egui::Color32::from_rgb(s.color.r, s.color.g, s.color.b);
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

                ui.add_space(8.0);

                // Independent X/Y Sensitivity Toggle
                ui.horizontal(|ui| {
                    let mut sep = self.config.separate_xy_dpi;
                    if ui
                        .checkbox(&mut sep, "Independent X / Y Axis Sensitivity (Separate X/Y DPI)")
                        .on_hover_text("Enable separate horizontal and vertical DPI calibration (e.g. for widescreen, ultrawide, or vertical recoil control)")
                        .changed()
                    {
                        self.config.separate_xy_dpi = sep;
                        if !sep {
                            for s in &mut self.config.dpi_stages {
                                s.dpi_y = None;
                            }
                        } else {
                            for s in &mut self.config.dpi_stages {
                                if s.dpi_y.is_none() {
                                    s.dpi_y = Some(s.dpi);
                                }
                            }
                        }
                        self.sync_dpi_text_inputs();
                    }
                });
                ui.add_space(6.0);

                // Quick Presets and Single DPI Lock Action Bar
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    let enabled_count = self.config.dpi_stages.iter().filter(|s| s.enabled).count();
                    let is_single_locked = enabled_count == 1;

                    let lock_btn = egui::Button::new(
                        egui::RichText::new(if is_single_locked {
                            "Unlock Stages"
                        } else {
                            "Lock Single DPI (FPS)"
                        })
                        .size(11.0)
                        .strong()
                        .color(if is_single_locked {
                            egui::Color32::from_rgb(22, 163, 74)
                        } else {
                            egui::Color32::from_rgb(2, 132, 199)
                        }),
                    )
                    .fill(if is_single_locked {
                        egui::Color32::from_rgb(240, 253, 244)
                    } else {
                        egui::Color32::from_rgb(240, 249, 255)
                    })
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        if is_single_locked {
                            egui::Color32::from_rgb(187, 247, 208)
                        } else {
                            egui::Color32::from_rgb(186, 230, 253)
                        },
                    ))
                    .rounding(egui::Rounding::same(4.0));

                    let lock_resp = ui.add(lock_btn);
                    if lock_resp.on_hover_text(if is_single_locked {
                        "Click to unlock and re-enable all DPI stages."
                    } else {
                        "Lock to active stage only (disables other stages to avoid accidental DPI shifts during competitive play)."
                    }).clicked() {
                        let active = (self.config.active_dpi_stage as usize)
                            .min(self.config.dpi_stages.len().saturating_sub(1));
                        if is_single_locked {
                            for s in &mut self.config.dpi_stages {
                                s.enabled = true;
                            }
                            self.set_status("All DPI stages unlocked and enabled", true);
                        } else if active < self.config.dpi_stages.len() {
                            for (idx, s) in self.config.dpi_stages.iter_mut().enumerate() {
                                s.enabled = idx == active;
                            }
                            self.set_status(format!("Locked to Single DPI: Stage {} ({} DPI)", active + 1, self.config.dpi_stages[active].dpi), true);
                        }
                    }

                    // Esports Pro Preset Button
                    let esports_btn = egui::Button::new(
                        egui::RichText::new("Esports Pro (400-3200)")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(51, 65, 85)),
                    )
                    .fill(egui::Color32::from_rgb(248, 250, 252))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240)))
                    .rounding(egui::Rounding::same(4.0));

                    if ui.add(esports_btn).on_hover_text("Preset: 400, 800, 1600, 3200 DPI (Stage 2: 800 active)").clicked() {
                        let preset = [400, 800, 1600, 3200];
                        for (idx, &dpi) in preset.iter().enumerate() {
                            if idx < self.config.dpi_stages.len() {
                                self.config.dpi_stages[idx].dpi = dpi;
                                if self.config.separate_xy_dpi {
                                    self.config.dpi_stages[idx].dpi_y = Some(dpi);
                                }
                                self.config.dpi_stages[idx].enabled = true;
                            }
                        }
                        if self.config.dpi_stages.len() > 4 {
                            self.config.dpi_stages[4].enabled = false;
                        }
                        self.config.active_dpi_stage = 1;
                        if let Some(stage) = self.config.dpi_stages.get(1) {
                            self.config.dpi_led.color = stage.color;
                        }
                        self.sync_dpi_text_inputs();
                        self.set_status("Applied Esports Pro DPI preset (400-3200)", true);
                    }

                    // 4K High-Res Preset Button
                    let hi_btn = egui::Button::new(
                        egui::RichText::new("4K High-Res (800-6400)")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(51, 65, 85)),
                    )
                    .fill(egui::Color32::from_rgb(248, 250, 252))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240)))
                    .rounding(egui::Rounding::same(4.0));

                    if ui.add(hi_btn).on_hover_text("Preset: 800, 1600, 2400, 3200, 6400 DPI").clicked() {
                        let preset = [800, 1600, 2400, 3200, 6400];
                        for (idx, &dpi) in preset.iter().enumerate() {
                            if idx < self.config.dpi_stages.len() {
                                self.config.dpi_stages[idx].dpi = dpi;
                                if self.config.separate_xy_dpi {
                                    self.config.dpi_stages[idx].dpi_y = Some(dpi);
                                }
                                self.config.dpi_stages[idx].enabled = true;
                            }
                        }
                        self.sync_dpi_text_inputs();
                        self.set_status("Applied 4K High-Res DPI preset (800-6400)", true);
                    }
                });

                ui.add_space(8.0);

                // 5 DPI Stage Configuration Cards
                for i in 0..self.config.dpi_stages.len() {
                    let is_active_stage = self.config.active_dpi_stage == i as u8;
                    let is_enabled = self.config.dpi_stages[i].enabled;

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
                                self.config.dpi_led.color = self.config.dpi_stages[i].color;
                                if self.auto_apply_lighting {
                                    self.queue_lighting_sync();
                                }
                            }

                            // Interactive Color Picker for this DPI stage
                            let mut rgb = [
                                self.config.dpi_stages[i].color.r,
                                self.config.dpi_stages[i].color.g,
                                self.config.dpi_stages[i].color.b,
                            ];
                            if ui
                                .color_edit_button_srgb(&mut rgb)
                                .on_hover_text("Click to customize LED color for this DPI stage")
                                .changed()
                            {
                                let new_col = Color::new(rgb[0], rgb[1], rgb[2]);
                                self.config.dpi_stages[i].color = new_col;
                                if is_active_stage {
                                    self.config.dpi_led.color = new_col;
                                    if self.auto_apply_lighting {
                                        self.queue_lighting_sync();
                                    }
                                }
                            }

                            ui.add_space(2.0);

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
                                            self.config.dpi_led.color =
                                                self.config.dpi_stages[first].color;
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(6.0);

                        // Row 2: X-Axis Stepping buttons, Slider, and Value Box
                        ui.horizontal(|ui| {
                            if self.config.separate_xy_dpi {
                                ui.label(
                                    egui::RichText::new("X-DPI:")
                                        .size(11.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(71, 85, 105)),
                                );
                            }

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
                                .on_hover_text("Decrease X-Axis by 50 DPI")
                                .clicked()
                            {
                                let cur = self.config.dpi_stages[i].dpi;
                                let new_val = cur.saturating_sub(50).max(50);
                                self.config.dpi_stages[i].dpi = new_val;
                                self.dpi_text_inputs[i] = new_val.to_string();
                            }

                            let mut dpi_val = self.config.dpi_stages[i].dpi as i32;
                            let slider = egui::Slider::new(&mut dpi_val, 50..=26000)
                                .step_by(50.0)
                                .show_value(false)
                                .trailing_fill(true)
                                .logarithmic(true);

                            let slider_width = if self.config.separate_xy_dpi { 210.0 } else { 250.0 };
                            let slider_resp = ui.add_sized(egui::vec2(slider_width, 24.0), slider);
                            if is_enabled && slider_resp.changed() {
                                let snapped = crate::dpi::snap_dpi(dpi_val.clamp(50, 26000) as u16);
                                self.config.dpi_stages[i].dpi = snapped;
                                self.dpi_text_inputs[i] = snapped.to_string();
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
                                .on_hover_text("Increase X-Axis by 50 DPI")
                                .clicked()
                            {
                                let cur = self.config.dpi_stages[i].dpi;
                                let new_val = cur.saturating_add(50).min(26000);
                                self.config.dpi_stages[i].dpi = new_val;
                                self.dpi_text_inputs[i] = new_val.to_string();
                            }

                            ui.add_space(8.0);

                            // Dedicated Editable DPI Value Box with Free Text Input
                            if self.dpi_text_inputs.len() <= i {
                                self.dpi_text_inputs.resize(i + 1, String::new());
                            }
                            let text_id = egui::Id::new(("dpi_stage_input", i));
                            let is_text_focused = ui.memory(|m| m.has_focus(text_id));

                            // Sync buffer from config when not actively typing (only allocate when out of sync)
                            if !is_text_focused {
                                let cur_dpi = self.config.dpi_stages[i].dpi;
                                if self.dpi_text_inputs[i].parse::<u16>().ok() != Some(cur_dpi) {
                                    self.dpi_text_inputs[i] = cur_dpi.to_string();
                                }
                            }

                            let pill_frame = egui::Frame::none()
                                .fill(if is_text_focused {
                                    egui::Color32::WHITE
                                } else if is_active_stage {
                                    egui::Color32::from_rgb(224, 242, 254)
                                } else {
                                    egui::Color32::from_rgb(248, 250, 252)
                                })
                                .stroke(egui::Stroke::new(
                                    if is_text_focused { 1.5_f32 } else { 1.0_f32 },
                                    if is_text_focused {
                                        egui::Color32::from_rgb(2, 132, 199)
                                    } else if is_active_stage {
                                        egui::Color32::from_rgb(186, 230, 253)
                                    } else {
                                        egui::Color32::from_rgb(226, 232, 240)
                                    },
                                ))
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::symmetric(6.0, 1.0));

                            let mut text_edit_resp = None;
                            pill_frame.show(ui, |ui| {
                                ui.set_min_size(egui::vec2(96.0, 24.0));
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    let text_widget =
                                        egui::TextEdit::singleline(&mut self.dpi_text_inputs[i])
                                            .id(text_id)
                                            .desired_width(52.0)
                                            .frame(false)
                                            .horizontal_align(egui::Align::RIGHT)
                                            .font(egui::FontId::proportional(12.5))
                                            .text_color(if !is_enabled {
                                                egui::Color32::from_rgb(148, 163, 184)
                                            } else {
                                                egui::Color32::from_rgb(15, 23, 42)
                                            });

                                    let resp = ui.add_enabled(is_enabled, text_widget);

                                    let lbl_resp = ui.add_enabled(
                                        is_enabled,
                                        egui::Label::new(
                                            egui::RichText::new(if self.config.separate_xy_dpi { "X-DPI" } else { "DPI" })
                                                .size(11.0)
                                                .color(if is_active_stage {
                                                    egui::Color32::from_rgb(2, 132, 199)
                                                } else if !is_enabled {
                                                    egui::Color32::from_rgb(148, 163, 184)
                                                } else {
                                                    egui::Color32::from_rgb(100, 116, 139)
                                                })
                                                .strong(),
                                        )
                                        .sense(egui::Sense::click()),
                                    );

                                    if is_enabled && lbl_resp.clicked() {
                                        resp.request_focus();
                                    }

                                    text_edit_resp = Some(resp);
                                });
                            });

                            if let Some(mut resp) = text_edit_resp {
                                resp = resp.on_hover_text(
                                    "Click to type custom DPI (50 – 26000, 50 DPI steps). Press Enter to apply.",
                                );

                                let enter_pressed = resp.has_focus()
                                    && ui.input(|inp| inp.key_pressed(egui::Key::Enter));

                                // Live update if user entered a valid number
                                if is_enabled
                                    && resp.changed()
                                    && let Some(val) =
                                        parse_ascii_digits_u32(&self.dpi_text_inputs[i])
                                            .filter(|&v| (50..=26000).contains(&v))
                                {
                                    self.config.dpi_stages[i].dpi = val as u16;
                                }

                                // On blur or Enter: validate, clamp to 50..=26000, snap to 50 DPI step, and format
                                if is_enabled && (resp.lost_focus() || enter_pressed) {
                                    if let Some(val) = parse_ascii_digits_u32(&self.dpi_text_inputs[i]) {
                                        let clamped = val.clamp(50, 26000) as u16;
                                        let snapped = crate::dpi::snap_dpi(clamped);
                                        self.config.dpi_stages[i].dpi = snapped;
                                        self.dpi_text_inputs[i] = snapped.to_string();
                                    } else {
                                        self.dpi_text_inputs[i] =
                                            self.config.dpi_stages[i].dpi.to_string();
                                    }

                                    if enter_pressed {
                                        resp.surrender_focus();
                                    }
                                }
                            }
                        });

                        // Row 3: Y-Axis DPI controls when Independent X/Y Sensitivity is active
                        if self.config.separate_xy_dpi {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Y-DPI:")
                                        .size(11.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(71, 85, 105)),
                                );

                                let minus_y_btn = egui::Button::new(
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
                                    .add_enabled(is_enabled, minus_y_btn)
                                    .on_hover_text("Decrease Y-Axis by 50 DPI")
                                    .clicked()
                                {
                                    let cur = self.config.dpi_stages[i].effective_y();
                                    let new_val = cur.saturating_sub(50).max(50);
                                    self.config.dpi_stages[i].dpi_y = Some(new_val);
                                    if self.dpi_y_text_inputs.len() > i {
                                        self.dpi_y_text_inputs[i] = new_val.to_string();
                                    }
                                }

                                let mut y_val = self.config.dpi_stages[i].effective_y() as i32;
                                let y_slider = egui::Slider::new(&mut y_val, 50..=26000)
                                    .step_by(50.0)
                                    .show_value(false)
                                    .trailing_fill(true)
                                    .logarithmic(true);

                                let y_slider_resp = ui.add_sized(egui::vec2(210.0, 24.0), y_slider);
                                if is_enabled && y_slider_resp.changed() {
                                    let snapped = crate::dpi::snap_dpi(y_val.clamp(50, 26000) as u16);
                                    self.config.dpi_stages[i].dpi_y = Some(snapped);
                                    if self.dpi_y_text_inputs.len() > i {
                                        self.dpi_y_text_inputs[i] = snapped.to_string();
                                    }
                                }

                                let plus_y_btn = egui::Button::new(
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
                                    .add_enabled(is_enabled, plus_y_btn)
                                    .on_hover_text("Increase Y-Axis by 50 DPI")
                                    .clicked()
                                {
                                    let cur = self.config.dpi_stages[i].effective_y();
                                    let new_val = cur.saturating_add(50).min(26000);
                                    self.config.dpi_stages[i].dpi_y = Some(new_val);
                                    if self.dpi_y_text_inputs.len() > i {
                                        self.dpi_y_text_inputs[i] = new_val.to_string();
                                    }
                                }

                                ui.add_space(8.0);

                                if self.dpi_y_text_inputs.len() <= i {
                                    self.dpi_y_text_inputs.resize(i + 1, String::new());
                                }
                                let y_text_id = egui::Id::new(("dpi_y_stage_input", i));
                                let is_y_focused = ui.memory(|m| m.has_focus(y_text_id));
                                if !is_y_focused {
                                    let cur_y = self.config.dpi_stages[i].effective_y();
                                    if self.dpi_y_text_inputs[i].parse::<u16>().ok() != Some(cur_y)
                                    {
                                        self.dpi_y_text_inputs[i] = cur_y.to_string();
                                    }
                                }

                                let y_pill_frame = egui::Frame::none()
                                    .fill(if is_y_focused {
                                        egui::Color32::WHITE
                                    } else if is_active_stage {
                                        egui::Color32::from_rgb(224, 242, 254)
                                    } else {
                                        egui::Color32::from_rgb(248, 250, 252)
                                    })
                                    .stroke(egui::Stroke::new(
                                        if is_y_focused { 1.5_f32 } else { 1.0_f32 },
                                        if is_y_focused {
                                            egui::Color32::from_rgb(2, 132, 199)
                                        } else if is_active_stage {
                                            egui::Color32::from_rgb(186, 230, 253)
                                        } else {
                                            egui::Color32::from_rgb(226, 232, 240)
                                        },
                                    ))
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 1.0));

                                let mut y_text_edit_resp = None;
                                y_pill_frame.show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(96.0, 24.0));
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        let y_widget = egui::TextEdit::singleline(&mut self.dpi_y_text_inputs[i])
                                            .id(y_text_id)
                                            .desired_width(52.0)
                                            .frame(false)
                                            .horizontal_align(egui::Align::RIGHT)
                                            .font(egui::FontId::proportional(12.5))
                                            .text_color(if !is_enabled {
                                                egui::Color32::from_rgb(148, 163, 184)
                                            } else {
                                                egui::Color32::from_rgb(15, 23, 42)
                                            });

                                        let resp = ui.add_enabled(is_enabled, y_widget);

                                        let lbl_resp = ui.add_enabled(
                                            is_enabled,
                                            egui::Label::new(
                                                egui::RichText::new("Y-DPI")
                                                    .size(11.0)
                                                    .color(if is_active_stage {
                                                        egui::Color32::from_rgb(2, 132, 199)
                                                    } else if !is_enabled {
                                                        egui::Color32::from_rgb(148, 163, 184)
                                                    } else {
                                                        egui::Color32::from_rgb(100, 116, 139)
                                                    })
                                                    .strong(),
                                            )
                                            .sense(egui::Sense::click()),
                                        );

                                        if is_enabled && lbl_resp.clicked() {
                                            resp.request_focus();
                                        }

                                        y_text_edit_resp = Some(resp);
                                    });
                                });

                                if let Some(mut resp) = y_text_edit_resp {
                                    resp = resp.on_hover_text(
                                        "Click to type custom Y-Axis DPI (50 – 26000, 50 DPI steps). Press Enter to apply.",
                                    );

                                    let enter_pressed = resp.has_focus()
                                        && ui.input(|inp| inp.key_pressed(egui::Key::Enter));

                                    if is_enabled
                                        && resp.changed()
                                        && let Some(val) =
                                            parse_ascii_digits_u32(&self.dpi_y_text_inputs[i])
                                                .filter(|&v| (50..=26000).contains(&v))
                                    {
                                        self.config.dpi_stages[i].dpi_y = Some(val as u16);
                                    }

                                    if is_enabled && (resp.lost_focus() || enter_pressed) {
                                        if let Some(val) =
                                            parse_ascii_digits_u32(&self.dpi_y_text_inputs[i])
                                        {
                                            let clamped = val.clamp(50, 26000) as u16;
                                            let snapped = crate::dpi::snap_dpi(clamped);
                                            self.config.dpi_stages[i].dpi_y = Some(snapped);
                                            self.dpi_y_text_inputs[i] = snapped.to_string();
                                        } else {
                                            self.dpi_y_text_inputs[i] =
                                                self.config.dpi_stages[i].effective_y().to_string();
                                        }

                                        if enter_pressed {
                                            resp.surrender_focus();
                                        }
                                    }
                                }
                            });
                        }
                    });
                    ui.add_space(6.0);
                }
            });
        });
    }
}
