use std::time::{Duration, Instant};

use eframe::egui;

use crate::config::{AppLanguage, ButtonAction};
use crate::gui::state::IcarusApp;
use crate::gui::theme::action_badge;

impl IcarusApp {
    /// Render Tab 1: Button Mapping with interactive 3D mouse visualizer and 6 ordered button cards
    pub(crate) fn render_buttons_tab(&mut self, ui: &mut egui::Ui) {
        let mut next_hovered = None;

        // Button hardware definitions (Name, Physical role note, Hardware button index 0..5)
        let button_definitions = self.config.language.button_definitions();

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
                        "../../../assets/Buttoninterface/mouse0.png"
                    ))
                    .fit_to_exact_size(egui::vec2(340.0, 280.0));

                    let img_resp = ui.centered_and_justified(|ui| ui.add(img)).inner;
                    let rect = img_resp.rect;
                    let w = rect.width();
                    let h = rect.height();

                    // Pin coordinates calibrated to exact physical button locations matching official driver (mouse0.png)
                    let pin_coords = [
                        (
                            0,
                            egui::pos2(rect.left() + w * 0.415, rect.top() + h * 0.435),
                        ), // 1: Left Click (Depan Kanan/Dekat)
                        (
                            1,
                            egui::pos2(rect.left() + w * 0.230, rect.top() + h * 0.405),
                        ), // 2: Right Click (Depan Kiri/Jauh)
                        (
                            2,
                            egui::pos2(rect.left() + w * 0.335, rect.top() + h * 0.345),
                        ), // 3: Middle Wheel (Roda Scroll)
                        (
                            3,
                            egui::pos2(rect.left() + w * 0.610, rect.top() + h * 0.495),
                        ), // 4: Forward Side (Samping Depan)
                        (
                            4,
                            egui::pos2(rect.left() + w * 0.705, rect.top() + h * 0.470),
                        ), // 5: Back Side (Samping Belakang)
                        (
                            5,
                            egui::pos2(rect.left() + w * 0.665, rect.top() + h * 0.660),
                        ), // 6: DPI Switch (Bawah Mouse)
                    ];

                    let pointer_pos = ui.input(|i| i.pointer.hover_pos());

                    // Register hover and click interactions for pins
                    for (btn_idx, pin_pos) in &pin_coords {
                        let pin_rect =
                            egui::Rect::from_center_size(*pin_pos, egui::vec2(24.0, 24.0));
                        let pin_interact = ui.interact(
                            pin_rect,
                            ui.id().with(("mouse_pin", *btn_idx)),
                            egui::Sense::click_and_drag(),
                        );
                        if pin_interact.hovered()
                            || pointer_pos.is_some_and(|p| pin_rect.contains(p))
                        {
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
                            // Clean white circular badge matching official driver
                            ui.painter()
                                .circle_filled(*pin_pos, 9.5, egui::Color32::WHITE);
                            ui.painter().circle_stroke(
                                *pin_pos,
                                9.5,
                                egui::Stroke::new(1.3_f32, egui::Color32::from_rgb(71, 85, 105)),
                            );
                            ui.painter().text(
                                *pin_pos,
                                egui::Align2::CENTER_CENTER,
                                format!("{}", btn_idx + 1),
                                egui::FontId::proportional(10.5),
                                egui::Color32::from_rgb(15, 23, 42),
                            );
                        }
                    }

                    ui.add_space(4.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(
                                "6 Programmable Hardware Buttons (Hover to highlight)",
                            )
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
                        egui::RichText::new("Anti-shake filter prevents double clicks (0 - 20 ms)")
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
                        ui.painter().rect_filled(
                            badge_r,
                            4.0,
                            egui::Color32::from_rgb(224, 242, 254),
                        );
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

                ui.add_space(10.0);

                // Mouse Click & Debounce Tester Card
                let test_frame = egui::Frame::none()
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgb(226, 232, 240),
                    ))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0));

                test_frame.show(ui, |ui| {
                    ui.set_width(382.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Mouse Click & Debounce Tester")
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::from_rgb(2, 132, 199)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Clear").clicked() {
                                self.test_clicks_count = 0;
                                self.test_last_click_time = None;
                                self.test_last_click_delta_ms = None;
                                self.test_double_click_detected = false;
                                self.test_recent_clicks.clear();
                                self.test_last_button_name = "None".into();
                            }
                        });
                    });

                    ui.add_space(4.0);

                    // Clickable Testing Area
                    let (target_rect, target_resp) = ui.allocate_exact_size(
                        egui::vec2(382.0, 48.0),
                        egui::Sense::click_and_drag(),
                    );

                    let is_target_hovered = target_resp.hovered();
                    ui.painter().rect_filled(
                        target_rect,
                        6.0,
                        if self.test_double_click_detected {
                            egui::Color32::from_rgb(254, 242, 242)
                        } else if is_target_hovered {
                            egui::Color32::from_rgb(240, 249, 255)
                        } else {
                            egui::Color32::from_rgb(248, 250, 252)
                        },
                    );
                    ui.painter().rect_stroke(
                        target_rect,
                        6.0,
                        egui::Stroke::new(
                            if is_target_hovered || self.test_double_click_detected {
                                1.5_f32
                            } else {
                                1.0_f32
                            },
                            if self.test_double_click_detected {
                                egui::Color32::from_rgb(239, 68, 68)
                            } else if is_target_hovered {
                                egui::Color32::from_rgb(56, 189, 248)
                            } else {
                                egui::Color32::from_rgb(226, 232, 240)
                            },
                        ),
                    );

                    // Capture any pointer click inside the box
                    let mut clicked_btn_name = None;
                    if target_resp.clicked() {
                        clicked_btn_name = Some("Left Click");
                    } else if target_resp.secondary_clicked() {
                        clicked_btn_name = Some("Right Click");
                    } else if target_resp.middle_clicked() {
                        clicked_btn_name = Some("Middle Click");
                    }

                    if let Some(btn_name) = clicked_btn_name {
                        let now = Instant::now();
                        if let Some(prev) = self.test_last_click_time {
                            let delta = now.saturating_duration_since(prev).as_millis() as u64;
                            self.test_last_click_delta_ms = Some(delta);
                            self.test_double_click_detected =
                                delta <= self.config.debounce_ms as u64 || delta < 20;
                        }
                        self.test_last_click_time = Some(now);
                        self.test_clicks_count += 1;
                        self.test_recent_clicks.push(now);
                        self.test_last_button_name = btn_name.to_string();
                    }

                    // Prune clicks older than 1 second to compute live CPS
                    let now = Instant::now();
                    self.test_recent_clicks
                        .retain(|t| now.saturating_duration_since(*t) <= Duration::from_secs(1));
                    let current_cps = self.test_recent_clicks.len();

                    ui.painter().text(
                        target_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        if self.test_clicks_count == 0 {
                            "Click rapidly inside here with Left / Right / Middle"
                        } else {
                            "Clicking active..."
                        },
                        egui::FontId::proportional(11.5),
                        egui::Color32::from_rgb(100, 116, 139),
                    );

                    ui.add_space(6.0);

                    // Statistics Row: Live CPS, Click Interval & Double-Click warning
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // CPS Badge
                        let (cps_r, _) =
                            ui.allocate_exact_size(egui::vec2(58.0, 20.0), egui::Sense::hover());
                        ui.painter().rect_filled(
                            cps_r,
                            4.0,
                            egui::Color32::from_rgb(224, 242, 254),
                        );
                        ui.painter().text(
                            cps_r.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{} CPS", current_cps),
                            egui::FontId::proportional(11.0),
                            egui::Color32::from_rgb(2, 132, 199),
                        );

                        ui.label(
                            egui::RichText::new(format!("Clicks: {}", self.test_clicks_count))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(71, 85, 105)),
                        );

                        if let Some(delta) = self.test_last_click_delta_ms {
                            let text_col = if self.test_double_click_detected {
                                egui::Color32::from_rgb(220, 38, 38)
                            } else {
                                egui::Color32::from_rgb(22, 163, 74)
                            };
                            let msg = if self.test_double_click_detected {
                                format!("Double Click! ({}ms)", delta)
                            } else {
                                format!("Delta: {}ms", delta)
                            };
                            ui.label(egui::RichText::new(msg).size(11.0).strong().color(text_col));
                        }

                        if self.test_last_button_name != "None" {
                            ui.label(
                                egui::RichText::new(format!("({})", self.test_last_button_name))
                                    .size(10.5)
                                    .color(egui::Color32::from_rgb(148, 163, 184)),
                            );
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
                    egui::RichText::new(
                        "Configure action bindings for all 6 mouse buttons (1 to 6):",
                    )
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

                    let is_listening_this_btn = self.binding_button_idx == Some(btn_idx);
                    let mut toggle_listen_btn = false;

                    let card_resp = card_frame.show(ui, |ui| {
                        ui.set_width(428.0);

                        // Row 1: Number Pill + Title + Subnote + Category Badge
                        ui.horizontal(|ui| {
                            let (p_rect, _) = ui
                                .allocate_exact_size(egui::vec2(22.0, 18.0), egui::Sense::hover());
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

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let (b_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(46.0, 16.0),
                                        egui::Sense::hover(),
                                    );
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
                                    ui.painter().rect_stroke(
                                        b_rect,
                                        3.0,
                                        egui::Stroke::new(1.0_f32, cat_color),
                                    );
                                    ui.painter().text(
                                        b_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        cat_label,
                                        egui::FontId::proportional(9.5),
                                        cat_color,
                                    );
                                },
                            );
                        });

                        ui.add_space(4.0);

                        // Row 2: Action Selection ComboBox
                        let current_display =
                            current_action.display_localized(self.config.language);

                        let mut action_category = match current_action {
                            ButtonAction::MouseButton(1) => 0,
                            ButtonAction::MouseButton(2) => 1,
                            ButtonAction::MouseButton(4) => 2,
                            ButtonAction::MouseButton(16) => 3,
                            ButtonAction::MouseButton(8) => 4,
                            ButtonAction::DpiSwitch => 5,
                            ButtonAction::FireKey { .. } => 6,
                            ButtonAction::Media(_) => 7,
                            ButtonAction::Keyboard { .. } => 8,
                            ButtonAction::Macro(_) => 9,
                            ButtonAction::Disabled => 10,
                            _ => 0,
                        };

                        let options = self.config.language.button_options();

                        let prev_category = action_category;
                        egui::ComboBox::from_id_salt(("btn_combo", btn_idx))
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
                                3 => ButtonAction::MouseButton(16),
                                4 => ButtonAction::MouseButton(8),
                                5 => ButtonAction::DpiSwitch,
                                6 => ButtonAction::FireKey {
                                    clicks: 3,
                                    delay_ms: 12,
                                },
                                7 => ButtonAction::Media(0xCD), // Play/Pause
                                8 => ButtonAction::Keyboard {
                                    usage: 0x04,
                                    modifier: 0x00,
                                }, // Default: A
                                9 => ButtonAction::Macro(0),
                                10 => ButtonAction::Disabled,
                                _ => ButtonAction::MouseButton(1),
                            };

                            // Safety Lock: Prevent user lockout by ensuring at least one button is Left Click (leftbuttononly)
                            let will_have_left_click =
                                self.config.buttons.iter().enumerate().any(|(i, b)| {
                                    if i == btn_idx {
                                        new_action == ButtonAction::MouseButton(1)
                                    } else {
                                        *b == ButtonAction::MouseButton(1)
                                    }
                                });

                            if !will_have_left_click {
                                self.set_status(
                                    self.config.language.status_left_click_lock_warn(),
                                    false,
                                );
                            } else if btn_idx < self.config.buttons.len() {
                                self.config.buttons[btn_idx] = new_action;
                            }
                        }

                        // Sub-selectors based on active action
                        match self.config.buttons.get_mut(btn_idx) {
                            Some(ButtonAction::FireKey { clicks, delay_ms }) => {
                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Clicks:").size(10.5));
                                    ui.add(egui::Slider::new(clicks, 2..=15));
                                    ui.label(egui::RichText::new("Delay:").size(10.5));
                                    ui.add(egui::Slider::new(delay_ms, 5..=50).suffix("ms"));
                                });
                            }
                            Some(ButtonAction::Media(usage)) => {
                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    let (media_label, media_items): (&str, &[(u8, &str)]) =
                                        match self.config.language {
                                            AppLanguage::English => (
                                                "Media:",
                                                &[
                                                    (0xCD, "Play / Pause"),
                                                    (0xB5, "Next Track"),
                                                    (0xB6, "Previous Track"),
                                                    (0xB7, "Stop"),
                                                    (0xE9, "Volume Up"),
                                                    (0xEA, "Volume Down"),
                                                    (0xE2, "Mute"),
                                                    (0x83, "Media Player"),
                                                    (0x8A, "Email Client"),
                                                    (0x92, "Calculator"),
                                                    (0x23, "Browser Home"),
                                                    (0x24, "Browser Back"),
                                                    (0x25, "Browser Forward"),
                                                ],
                                            ),
                                            AppLanguage::Indonesian => (
                                                "Media:",
                                                &[
                                                    (0xCD, "Putar / Jeda (Play/Pause)"),
                                                    (0xB5, "Lagu Selanjutnya"),
                                                    (0xB6, "Lagu Sebelumnya"),
                                                    (0xB7, "Berhenti (Stop)"),
                                                    (0xE9, "Volume Naik"),
                                                    (0xEA, "Volume Turun"),
                                                    (0xE2, "Bisu (Mute)"),
                                                    (0x83, "Pemutar Media"),
                                                    (0x8A, "Aplikasi Email"),
                                                    (0x92, "Kalkulator"),
                                                    (0x23, "Beranda Web"),
                                                    (0x24, "Web Mundur (Back)"),
                                                    (0x25, "Web Maju (Forward)"),
                                                ],
                                            ),
                                        };
                                    ui.label(egui::RichText::new(media_label).size(11.0).strong());
                                    let current_media_label = media_items
                                        .iter()
                                        .find(|(u, _)| *u == *usage)
                                        .map(|(_, l)| *l)
                                        .unwrap_or_else(|| match self.config.language {
                                            AppLanguage::English => "Custom Media Key",
                                            AppLanguage::Indonesian => "Tombol Media Kustom",
                                        });

                                    egui::ComboBox::from_id_salt(("btn_media_combo", btn_idx))
                                        .selected_text(current_media_label)
                                        .width(220.0)
                                        .show_ui(ui, |ui| {
                                            for (u, l) in media_items {
                                                ui.selectable_value(usage, *u, *l);
                                            }
                                        });
                                });
                            }
                            Some(ButtonAction::Keyboard { usage, modifier }) => {
                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    let is_listening = is_listening_this_btn;
                                    let btn_text = if is_listening {
                                        self.config.language.listening_key_btn()
                                    } else {
                                        self.config.language.capture_key_btn()
                                    };
                                    let capture_btn = egui::Button::new(
                                        egui::RichText::new(btn_text).size(11.0).strong().color(
                                            if is_listening {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::from_rgb(2, 132, 199)
                                            },
                                        ),
                                    )
                                    .fill(if is_listening {
                                        egui::Color32::from_rgb(220, 38, 38)
                                    } else {
                                        egui::Color32::from_rgb(240, 249, 255)
                                    })
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(186, 230, 253),
                                    ))
                                    .rounding(egui::Rounding::same(4.0));

                                    if ui.add(capture_btn).clicked() {
                                        toggle_listen_btn = true;
                                    }

                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Key: {}",
                                            crate::protocol::hid_usage_to_key_name(*usage)
                                        ))
                                        .size(11.0)
                                        .strong(),
                                    );

                                    // Modifiers checkboxes
                                    let mut ctrl = (*modifier & 0x01) != 0;
                                    let mut shift = (*modifier & 0x02) != 0;
                                    let mut alt = (*modifier & 0x04) != 0;
                                    let mut win = (*modifier & 0x08) != 0;

                                    if ui.checkbox(&mut ctrl, "Ctrl").changed() {
                                        if ctrl {
                                            *modifier |= 0x01;
                                        } else {
                                            *modifier &= !0x01;
                                        }
                                    }
                                    if ui.checkbox(&mut shift, "Shift").changed() {
                                        if shift {
                                            *modifier |= 0x02;
                                        } else {
                                            *modifier &= !0x02;
                                        }
                                    }
                                    if ui.checkbox(&mut alt, "Alt").changed() {
                                        if alt {
                                            *modifier |= 0x04;
                                        } else {
                                            *modifier &= !0x04;
                                        }
                                    }
                                    if ui.checkbox(&mut win, "Super").changed() {
                                        if win {
                                            *modifier |= 0x08;
                                        } else {
                                            *modifier &= !0x08;
                                        }
                                    }
                                });
                            }
                            Some(ButtonAction::Macro(slot)) => {
                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    let (target_macro_label, no_macros_msg) =
                                        match self.config.language {
                                            AppLanguage::English => (
                                                "Target Macro:",
                                                "(No macros created yet - add one in Macro tab)",
                                            ),
                                            AppLanguage::Indonesian => (
                                                "Makro Target:",
                                                "(Belum ada makro dibuat - tambahkan di tab Makro)",
                                            ),
                                        };
                                    ui.label(
                                        egui::RichText::new(target_macro_label).size(11.0).strong(),
                                    );
                                    if self.config.macros.is_empty() {
                                        ui.label(
                                            egui::RichText::new(no_macros_msg)
                                                .size(11.0)
                                                .color(egui::Color32::from_rgb(220, 38, 38)),
                                        );
                                    } else {
                                        let cur_slot = *slot;
                                        let cur_name = self
                                            .config
                                            .macros
                                            .get(cur_slot as usize)
                                            .map(|m| format!("Slot {}: {}", cur_slot + 1, m.name))
                                            .unwrap_or_else(|| format!("Slot {}", cur_slot + 1));

                                        egui::ComboBox::from_id_salt(("btn_macro_combo", btn_idx))
                                            .selected_text(cur_name)
                                            .width(220.0)
                                            .show_ui(ui, |ui| {
                                                for (m_idx, m) in
                                                    self.config.macros.iter().enumerate()
                                                {
                                                    let s = m_idx as u8;
                                                    ui.selectable_value(
                                                        slot,
                                                        s,
                                                        format!("Slot {}: {}", s + 1, m.name),
                                                    );
                                                }
                                            });
                                    }
                                });
                            }
                            _ => {}
                        }
                    });

                    if toggle_listen_btn {
                        if self.binding_button_idx == Some(btn_idx) {
                            self.binding_button_idx = None;
                        } else {
                            self.binding_button_idx = Some(btn_idx);
                            self.set_status(self.config.language.status_listen_hint(), true);
                        }
                    }

                    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                    let rect = card_resp.response.rect;
                    if card_resp.response.hovered() || pointer_pos.is_some_and(|p| rect.contains(p))
                    {
                        next_hovered = Some(btn_idx);
                    }

                    ui.add_space(6.0);
                }
            });
        });

        self.hovered_button = next_hovered;
    }
}
