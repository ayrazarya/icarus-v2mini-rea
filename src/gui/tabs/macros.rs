use std::time::Instant;

use eframe::egui;

use crate::config::{AppLanguage, MacroEvent, MacroEventType, MacroItem, MacroRepeatMode};
use crate::gui::state::IcarusApp;

impl IcarusApp {
    /// Render Tab 4: Macro Manager with visual flowchart sequence cards & delay presets
    pub(crate) fn render_macro_tab(&mut self, ui: &mut egui::Ui) {
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
                            .map(|m| m.name.as_str())
                            .unwrap_or("");

                        let item_btn = egui::Button::new(
                            egui::RichText::new(macro_name)
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

                    // New Macro creation (guarded by 16-slot hardware EEPROM capacity)
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_macro_name);
                        if ui.button("Add Macro").clicked() {
                            let name_trimmed = self.new_macro_name.trim();
                            if name_trimmed.is_empty() {
                                self.set_status("Enter a macro name first", false);
                            } else if self.config.macros.len() >= crate::protocol::MAX_MACRO_SLOTS {
                                self.set_status(
                                    format!(
                                        "Maximum of {} hardware macro slots reached",
                                        crate::protocol::MAX_MACRO_SLOTS
                                    ),
                                    false,
                                );
                            } else {
                                self.config.macros.push(MacroItem {
                                    name: name_trimmed.to_string(),
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
                        }
                    });

                    // Delete Macro (adjusting any button Macro(slot) bindings so indices never shift or go out of bounds)
                    if self.config.macros.len() > 1 && ui.button("Delete Selected").clicked() {
                        let del_idx = self.selected_macro_idx;
                        if del_idx < self.config.macros.len() {
                            self.config.macros.remove(del_idx);
                            let del_slot = del_idx as u8;
                            for btn in &mut self.config.buttons {
                                if let crate::config::ButtonAction::Macro(slot) = btn {
                                    if *slot == del_slot {
                                        *slot = 0;
                                    } else if *slot > del_slot {
                                        *slot -= 1;
                                    }
                                }
                            }
                            for prof in &mut self.profiles {
                                for btn in &mut prof.buttons {
                                    if let crate::config::ButtonAction::Macro(slot) = btn {
                                        if *slot == del_slot {
                                            *slot = 0;
                                        } else if *slot > del_slot {
                                            *slot -= 1;
                                        }
                                    }
                                }
                            }
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
                            .on_hover_text("Start recording live keystrokes")
                            .clicked()
                        {
                            self.is_recording_macro = true;
                            self.last_macro_record_time = Some(Instant::now());
                            self.set_status(self.config.language.status_recording_live(), true);
                        }

                        let (stop_label, stop_hover) = match self.config.language {
                            AppLanguage::English => ("Stop", "Stop recording keystrokes"),
                            AppLanguage::Indonesian => ("Berhenti", "Hentikan perekaman tombol"),
                        };
                        let stop_btn = egui::Button::new(
                            egui::RichText::new(stop_label)
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
                            .on_hover_text(stop_hover)
                            .clicked()
                        {
                            self.is_recording_macro = false;
                            self.last_macro_record_time = None;
                            // Pop any trailing Mouse Left event captured during clicking the Stop button
                            if let Some(mac) = self.config.macros.get_mut(self.selected_macro_idx) {
                                while let Some(last) = mac.events.last() {
                                    if last.key == "Mouse Left" || (last.event_type == crate::config::MacroEventType::Delay && last.delay_ms <= 100) {
                                        mac.events.pop();
                                    } else {
                                        break;
                                    }
                                }
                            }
                            let stop_msg = match self.config.language {
                                AppLanguage::English => "Macro recording stopped.",
                                AppLanguage::Indonesian => "Perekaman makro dihentikan.",
                            };
                            self.set_status(stop_msg, true);
                        }

                        if self.is_recording_macro {
                            ui.label(
                                egui::RichText::new("[REC LIVE]")
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .strong(),
                            );
                        }

                        let hw_buf_label = match self.config.language {
                            AppLanguage::English => "Hardware Buffer",
                            AppLanguage::Indonesian => "Buffer Hardware",
                        };
                        let hw_buf_hover = match self.config.language {
                            AppLanguage::English => "Convert and verify on-board binary buffer structure (MacroKeyToBuffer)",
                            AppLanguage::Indonesian => "Konversi dan verifikasi struktur buffer biner on-board (MacroKeyToBuffer)",
                        };
                        let hw_buf_btn = egui::Button::new(
                            egui::RichText::new(hw_buf_label)
                                .color(egui::Color32::from_rgb(2, 132, 199))
                                .size(11.0)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(240, 249, 255))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(186, 230, 253)))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(100.0, 28.0));

                        if let Some(mac) = self.config.macros.get(self.selected_macro_idx) {
                            let clicked = ui
                                .add(hw_buf_btn)
                                .on_hover_text(hw_buf_hover)
                                .clicked();
                            if clicked {
                                let buf = crate::protocol::macro_to_hardware_buffer(&mac.name, &mac.events);
                                self.set_status(
                                    self.config.language.status_macro_buffer_ok(&mac.name, buf.len()),
                                    true,
                                );
                            }
                        }

                        // Flash Macro to Onboard EEPROM
                        let flash_btn = egui::Button::new(
                            egui::RichText::new(self.config.language.flash_macro_slot_btn())
                                .color(egui::Color32::WHITE)
                                .size(11.0)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(16, 185, 129))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(110.0, 28.0));

                        let mac_opt = self.config.macros.get(self.selected_macro_idx);
                        let flash_hover = match self.config.language {
                            AppLanguage::English => "Write macro bytecode directly to mouse on-board EEPROM slot",
                            AppLanguage::Indonesian => "Tulis bytecode makro langsung ke slot EEPROM on-board mouse",
                        };
                        if ui
                            .add_enabled(mac_opt.is_some(), flash_btn)
                            .on_hover_text(flash_hover)
                            .clicked()
                            && let Some(mac) = mac_opt
                        {
                            if let Some(ref dev) = self.device {
                                let slot = self.selected_macro_idx as u8;
                                match dev.flash_macro_slot(slot, &mac.name, &mac.events) {
                                    Ok(()) => self.set_status(
                                        self.config.language.status_macro_flashed_ok(&mac.name, slot + 1),
                                        true,
                                    ),
                                    Err(e) => self.set_status(self.config.language.status_macro_flash_err(&e.to_string()), false),
                                }
                            } else {
                                self.set_status(self.config.language.status_mouse_not_connected(), false);
                            }
                        }
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let flash_all_btn = egui::Button::new(
                            egui::RichText::new(self.config.language.flash_all_macros_btn())
                                .size(11.0)
                                .color(egui::Color32::from_rgb(15, 23, 42)),
                        )
                        .fill(egui::Color32::from_rgb(241, 245, 249))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225)))
                        .rounding(egui::Rounding::same(4.0));

                        let flash_all_hover = match self.config.language {
                            AppLanguage::English => "Automatically flash all button-assigned macros to mouse EEPROM",
                            AppLanguage::Indonesian => "Flash semua makro yang terpasang pada tombol ke mouse secara otomatis",
                        };
                        if ui.add(flash_all_btn).on_hover_text(flash_all_hover).clicked() {
                            if let Some(ref dev) = self.device {
                                match dev.flash_all_assigned_macros(&self.config) {
                                    Ok(()) => self.set_status(self.config.language.status_macro_all_flashed_ok(), true),
                                    Err(e) => self.set_status(self.config.language.status_macro_all_flash_err(&e.to_string()), false),
                                }
                            } else {
                                self.set_status(self.config.language.status_mouse_not_connected(), false);
                            }
                        }
                    });

                    ui.add_space(6.0);
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
                                                            "../../../assets/Macro/MacroDown.png"
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
                                                            "../../../assets/Macro/MacroUp.png"
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
                                                            "../../../assets/Macro/MacroTime.png"
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
                                                            .range(1..=65_535)
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

                        // Manual event insertion tools (guarded by 64-event hardware slot capacity)
                        let mut capacity_exceeded = false;
                        let mut empty_key_err = false;
                        let mut invalid_key_err: Option<String> = None;
                        ui.horizontal(|ui| {
                            ui.label("Insert Key:");
                            ui.text_edit_singleline(&mut self.new_key_input);
                            if ui.button("Down").clicked() {
                                let key_trimmed = self.new_key_input.trim();
                                if key_trimmed.is_empty() {
                                    empty_key_err = true;
                                } else if crate::protocol::key_to_hid_usage(key_trimmed) == 0 {
                                    invalid_key_err = Some(key_trimmed.to_string());
                                } else if mac.events.len() >= 64 {
                                    capacity_exceeded = true;
                                } else {
                                    mac.events.push(MacroEvent {
                                        event_type: MacroEventType::KeyDown,
                                        key: key_trimmed.to_string(),
                                        delay_ms: 0,
                                    });
                                }
                            }
                            if ui.button("Up").clicked() {
                                let key_trimmed = self.new_key_input.trim();
                                if key_trimmed.is_empty() {
                                    empty_key_err = true;
                                } else if crate::protocol::key_to_hid_usage(key_trimmed) == 0 {
                                    invalid_key_err = Some(key_trimmed.to_string());
                                } else if mac.events.len() >= 64 {
                                    capacity_exceeded = true;
                                } else {
                                    mac.events.push(MacroEvent {
                                        event_type: MacroEventType::KeyUp,
                                        key: key_trimmed.to_string(),
                                        delay_ms: 0,
                                    });
                                }
                            }
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Insert Mouse Click:");
                            let mouse_buttons = [
                                ("+ Left", "Mouse Left"),
                                ("+ Right", "Mouse Right"),
                                ("+ Middle", "Mouse Middle"),
                                ("+ Back", "Mouse Back"),
                                ("+ Forward", "Mouse Forward"),
                            ];
                            for (label, key_name) in mouse_buttons {
                                if ui.button(label).clicked() {
                                    let slots_needed = if mac.auto_insert_delay { 3 } else { 2 };
                                    if mac.events.len() + slots_needed > 64 {
                                        capacity_exceeded = true;
                                    } else {
                                        mac.events.push(MacroEvent {
                                            event_type: MacroEventType::KeyDown,
                                            key: key_name.to_string(),
                                            delay_ms: 0,
                                        });
                                        if mac.auto_insert_delay {
                                            mac.events.push(MacroEvent {
                                                event_type: MacroEventType::Delay,
                                                key: String::new(),
                                                delay_ms: mac.default_delay_ms.clamp(1, 65_535),
                                            });
                                        }
                                        mac.events.push(MacroEvent {
                                            event_type: MacroEventType::KeyUp,
                                            key: key_name.to_string(),
                                            delay_ms: 0,
                                        });
                                    }
                                }
                            }
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Quick Delay:");
                            let delay_presets = [10, 25, 50, 100];
                            for d in delay_presets {
                                if ui.button(format!("+{}ms", d)).clicked() {
                                    if mac.events.len() >= 64 {
                                        capacity_exceeded = true;
                                    } else {
                                        mac.events.push(MacroEvent {
                                            event_type: MacroEventType::Delay,
                                            key: String::new(),
                                            delay_ms: d,
                                        });
                                    }
                                }
                            }
                        });

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.label("Custom Delay:");
                            ui.add(
                                egui::DragValue::new(&mut self.new_delay_ms)
                                    .range(1..=65_535)
                                    .suffix(" ms"),
                            );
                            if ui.button("Add Delay").clicked() {
                                if mac.events.len() >= 64 {
                                    capacity_exceeded = true;
                                } else {
                                    mac.events.push(MacroEvent {
                                        event_type: MacroEventType::Delay,
                                        key: String::new(),
                                        delay_ms: self.new_delay_ms.clamp(1, 65_535),
                                    });
                                }
                            }
                        });

                        if capacity_exceeded {
                            self.set_status(
                                "Macro slot full: hardware supports at most 64 events per macro",
                                false,
                            );
                        } else if empty_key_err {
                            self.set_status("Key name cannot be empty", false);
                        } else if let Some(bad_key) = invalid_key_err {
                            self.set_status(
                                format!("Unrecognized HID key name: '{}'", bad_key),
                                false,
                            );
                        }
                    }
                });
            });
        });
    }
}
