use std::time::{Duration, Instant};

use eframe::egui;

use super::state::{IcarusApp, PairingState, Tab};
use crate::config::{AppLanguage, ButtonAction, MouseConfig};

impl eframe::App for IcarusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global Keyboard Shortcuts (Ctrl+S, Ctrl+R, Esc, 1..5 / Ctrl+1..5)
        // Suspended when capturing a button key binding or recording a macro so keys 1..5 / Ctrl+S can be captured
        let is_text_focused = ctx.memory(|m| m.focused().is_some());
        let is_capturing_input = self.binding_button_idx.is_some() || self.is_recording_macro;
        ctx.input(|i| {
            if !is_capturing_input {
                if i.modifiers.command && i.key_pressed(egui::Key::S) {
                    self.apply_to_device();
                }
                if i.modifiers.command && i.key_pressed(egui::Key::R) {
                    self.read_from_device();
                }
                if i.key_pressed(egui::Key::Escape) {
                    ctx.memory_mut(|m| {
                        if let Some(id) = m.focused() {
                            m.surrender_focus(id);
                        }
                    });
                    self.is_renaming_profile = false;
                }
                let ctrl = i.modifiers.command;
                if ctrl || !is_text_focused {
                    if i.key_pressed(egui::Key::Num1) {
                        self.active_tab = Tab::Buttons;
                    } else if i.key_pressed(egui::Key::Num2) {
                        self.active_tab = Tab::Sensor;
                    } else if i.key_pressed(egui::Key::Num3) {
                        self.active_tab = Tab::Lighting;
                    } else if i.key_pressed(egui::Key::Num4) {
                        self.active_tab = Tab::Macro;
                    } else if i.key_pressed(egui::Key::Num5) {
                        self.active_tab = Tab::Settings;
                    }
                }
            }
        });

        // Periodic battery, wireless online status, and USB hot-plug polling every 4 seconds
        if self.last_battery_poll.elapsed() >= Duration::from_secs(4) {
            self.last_battery_poll = Instant::now();
            if self.device.is_none() {
                if crate::device::list_devices().is_ok_and(|list| !list.is_empty()) {
                    self.refresh_device();
                }
            } else if let Some(ref dev) = self.device {
                let mut fatal_hid_err = false;
                if self.is_wireless {
                    let was_online = self.is_device_online;
                    match dev.read_device_online() {
                        Ok(online) => {
                            self.is_device_online = online;
                            if online {
                                self.connection_status = match self.config.language {
                                    AppLanguage::English => {
                                        format!("Connected ({})", dev.connection)
                                    }
                                    AppLanguage::Indonesian => {
                                        format!("Terhubung ({})", dev.connection)
                                    }
                                };
                                if !was_online
                                    && self.firmware_version == "Unknown"
                                    && let Ok(fw) = dev.read_firmware_version()
                                {
                                    self.firmware_version = fw;
                                }
                            } else {
                                self.connection_status =
                                    self.config.language.dongle_ready_mouse_offline().into();
                            }
                        }
                        Err(e) => {
                            if let Some(crate::protocol::ProtocolError::Hid(_)) =
                                e.downcast_ref::<crate::protocol::ProtocolError>()
                            {
                                fatal_hid_err = true;
                            }
                        }
                    }
                }

                if !fatal_hid_err && (!self.is_wireless || self.is_device_online) {
                    match dev.read_battery_status() {
                        Ok(status) => {
                            self.battery_level = Some(status.level);
                            self.is_charging = status.is_charging;
                            if status.voltage_mv > 0 {
                                self.battery_voltage_mv = Some(status.voltage_mv);
                            }
                        }
                        Err(e) => {
                            // If device read fails with HID error (e.g. USB unplugged), invalidate device connection
                            if let Some(crate::protocol::ProtocolError::Hid(_)) =
                                e.downcast_ref::<crate::protocol::ProtocolError>()
                            {
                                fatal_hid_err = true;
                            }
                        }
                    }
                }

                if fatal_hid_err {
                    self.device = None;
                    self.connection_status = self.config.language.disconnected().into();
                    self.battery_level = None;
                }
            }
        }

        // Repaint animation when charging or when battery is critical (for glowing or pulsing warnings)
        if self.is_charging || self.battery_level.is_some_and(|b| b <= 15) {
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        // Flush throttled real-time lighting / dongle RGB updates when slider drag ends or interval elapses
        if self.pending_lighting_sync || self.pending_dongle_sync {
            let pointer_down = ctx.input(|i| i.pointer.any_down());
            self.flush_pending_lighting_sync(pointer_down);
            if self.pending_lighting_sync || self.pending_dongle_sync {
                ctx.request_repaint_after(Duration::from_millis(60));
            }
        }

        // Interactive Button Keyboard Binding Listener
        if let Some(btn_idx) = self.binding_button_idx {
            let mut captured_key = None;
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    captured_key = Some(None);
                } else {
                    for event in &i.events {
                        if let egui::Event::Key {
                            key,
                            pressed: true,
                            repeat: false,
                            modifiers,
                            ..
                        } = event
                        {
                            let mut mod_mask = 0u8;
                            if modifiers.ctrl {
                                mod_mask |= 0x01;
                            }
                            if modifiers.shift {
                                mod_mask |= 0x02;
                            }
                            if modifiers.alt {
                                mod_mask |= 0x04;
                            }
                            if modifiers.command || modifiers.mac_cmd {
                                mod_mask |= 0x08;
                            }
                            let key_name = format!("{:?}", key);
                            let usage = crate::protocol::key_to_hid_usage(&key_name);
                            if usage != 0 || mod_mask != 0 {
                                captured_key = Some(Some((usage, mod_mask)));
                                break;
                            }
                        }
                    }
                }
            });

            if let Some(binding_opt) = captured_key {
                if let Some((usage, modifier)) = binding_opt
                    && btn_idx < self.config.buttons.len()
                {
                    let will_have_left_click =
                        self.config.buttons.iter().enumerate().any(|(idx, act)| {
                            idx != btn_idx && matches!(act, ButtonAction::MouseButton(1))
                        });
                    if !will_have_left_click {
                        self.set_status(self.config.language.status_left_click_lock_warn(), false);
                    } else {
                        self.config.buttons[btn_idx] = ButtonAction::Keyboard { usage, modifier };
                        self.set_status(
                            self.config.language.status_button_bound(
                                btn_idx + 1,
                                &self.config.buttons[btn_idx].to_string(),
                            ),
                            true,
                        );
                    }
                }
                self.binding_button_idx = None;
            }
        }

        // Live Macro Keystroke & Mouse Click Recording when is_recording_macro is active
        if self.is_recording_macro {
            let mut captured_events = Vec::new();
            let now = Instant::now();
            let last_time = self.last_macro_record_time.unwrap_or(now);
            let elapsed_ms = now.saturating_duration_since(last_time).as_millis() as u32;

            ctx.input(|i| {
                for event in &i.events {
                    match event {
                        egui::Event::Key {
                            key,
                            pressed,
                            repeat: false,
                            ..
                        } => {
                            captured_events.push((format!("{:?}", key), *pressed));
                        }
                        egui::Event::PointerButton {
                            button, pressed, ..
                        } => {
                            let btn_str = match button {
                                egui::PointerButton::Primary => Some("Mouse Left"),
                                egui::PointerButton::Secondary => Some("Mouse Right"),
                                egui::PointerButton::Middle => Some("Mouse Middle"),
                                egui::PointerButton::Extra1 => Some("Mouse Back"),
                                egui::PointerButton::Extra2 => Some("Mouse Forward"),
                            };
                            if let Some(name) = btn_str {
                                captured_events.push((name.to_string(), *pressed));
                            }
                        }
                        _ => {}
                    }
                }
            });

            if !captured_events.is_empty() {
                self.last_macro_record_time = Some(now);
                if let Some(mac) = self.config.macros.get_mut(self.selected_macro_idx) {
                    let mut first = true;
                    for (key_name, pressed) in captured_events {
                        let need_delay = first && mac.auto_insert_delay && elapsed_ms >= 5;
                        // Ensure a KeyDown event always reserves room for its matching KeyUp within the 64-event hardware slot
                        let slots_needed = usize::from(need_delay) + if pressed { 2 } else { 1 };
                        if mac.events.len() + slots_needed > 64 {
                            self.is_recording_macro = false;
                            break;
                        }
                        if need_delay {
                            mac.events.push(crate::config::MacroEvent {
                                event_type: crate::config::MacroEventType::Delay,
                                key: String::new(),
                                delay_ms: elapsed_ms.clamp(5, 5000),
                            });
                        }
                        first = false;
                        mac.events.push(crate::config::MacroEvent {
                            event_type: if pressed {
                                crate::config::MacroEventType::KeyDown
                            } else {
                                crate::config::MacroEventType::KeyUp
                            },
                            key: key_name,
                            delay_ms: 0,
                        });
                    }
                }
            }
        }

        // Update 2.4G pairing state (throttled USB poll every 250ms so mouse movement doesn't flood HID)
        if self.pairing_state == PairingState::Pairing
            && let Some(start) = self.pairing_start
        {
            ctx.request_repaint_after(Duration::from_millis(250));
            if start.elapsed() >= Duration::from_secs(10) {
                self.pairing_state = PairingState::Failed;
                let fail_msg = match self.config.language {
                    AppLanguage::English => "Pairing timed out. Mouse was not detected.",
                    AppLanguage::Indonesian => "Pairing habis waktu. Mouse tidak terdeteksi.",
                };
                self.set_status(fail_msg, false);
            } else if self.last_pairing_poll.elapsed() >= Duration::from_millis(250) {
                self.last_pairing_poll = Instant::now();
                if let Some(ref dev) = self.device
                    && let Ok(status) = dev.read_dongle_pair_status()
                    && status == 0x01
                {
                    self.pairing_state = PairingState::Success;
                    let ok_msg = match self.config.language {
                        AppLanguage::English => "2.4G Wireless Dongle paired successfully!",
                        AppLanguage::Indonesian => "Dongle Wireless 2.4G berhasil dipasangkan!",
                    };
                    self.set_status(ok_msg, true);
                }
            }
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
                        egui::Image::new(egui::include_image!("../../assets/Dialog/Logo.png"))
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
                                egui::RichText::new(self.config.language.subtitle())
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(100, 116, 139)),
                            );
                        });
                    });

                    ui.add_space(24.0);

                    // Compact Profile Dropdown Selector with Custom Renaming
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        ui.label(
                            egui::RichText::new(self.config.language.profile())
                                .size(12.5)
                                .color(egui::Color32::from_rgb(71, 85, 105))
                                .strong(),
                        );

                        let cur_profile = self.config.active_profile as usize;
                        let cur_name_str = self
                            .profile_names
                            .get(cur_profile.saturating_sub(1))
                            .map(String::as_str)
                            .unwrap_or("Default");

                        if self.is_renaming_profile {
                            let text_edit = egui::TextEdit::singleline(&mut self.profile_rename_buf)
                                .desired_width(120.0);
                            let r_resp = ui.add(text_edit);
                            r_resp.request_focus();

                            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let save_btn = egui::Button::new(
                                egui::RichText::new(self.config.language.save())
                                    .size(11.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(2, 132, 199))
                            .rounding(egui::Rounding::same(4.0));

                            if ui.add(save_btn).clicked() || enter {
                                let idx = (self.config.active_profile as usize).saturating_sub(1);
                                if idx < self.profile_names.len()
                                    && !self.profile_rename_buf.trim().is_empty()
                                {
                                    self.profile_names[idx] =
                                        self.profile_rename_buf.trim().to_string();
                                    self.set_status(
                                        format!("Profile {} renamed to '{}'", idx + 1, self.profile_names[idx]),
                                        true,
                                    );
                                }
                                self.is_renaming_profile = false;
                            }

                            let cancel_btn = egui::Button::new(
                                egui::RichText::new(self.config.language.cancel())
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(71, 85, 105)),
                            )
                            .fill(egui::Color32::from_rgb(241, 245, 249))
                            .rounding(egui::Rounding::same(4.0));

                            if ui.add(cancel_btn).clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                self.is_renaming_profile = false;
                            }
                        } else {
                            let mut selected_profile = self.config.active_profile;
                            let prev_profile = selected_profile;
                            egui::ComboBox::from_id_salt("header_profile_dropdown")
                                .selected_text(
                                    egui::RichText::new(format!(
                                        "P{}: {}",
                                        selected_profile, cur_name_str
                                    ))
                                    .size(12.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(15, 23, 42)),
                                )
                                .width(135.0)
                                .show_ui(ui, |ui| {
                                    for p in 1..=5u8 {
                                        let name = &self.profile_names[(p - 1) as usize];
                                        let is_active = p == selected_profile;
                                        let label = if is_active {
                                            egui::RichText::new(format!("P{}: {}", p, name))
                                                .size(12.0)
                                                .strong()
                                                .color(egui::Color32::from_rgb(2, 132, 199))
                                        } else {
                                            egui::RichText::new(format!("P{}: {}", p, name))
                                                .size(12.0)
                                                .color(egui::Color32::from_rgb(30, 41, 59))
                                        };
                                        ui.selectable_value(&mut selected_profile, p, label);
                                    }
                                });

                            if selected_profile != prev_profile {
                                let old_idx = (prev_profile as usize).saturating_sub(1);
                                if old_idx < self.profiles.len() {
                                    self.profiles[old_idx] = self.config.clone();
                                }
                                let new_idx = (selected_profile as usize).saturating_sub(1);
                                if new_idx < self.profiles.len() {
                                    let saved_lang = self.config.language;
                                    let saved_macros = std::mem::take(&mut self.config.macros);
                                    let saved_rules =
                                        std::mem::take(&mut self.config.auto_profile_rules);
                                    let saved_auto_profile = self.config.auto_profile_enabled;
                                    let saved_dongle = self.config.dongle_rgb;

                                    self.config = self.profiles[new_idx].clone();
                                    self.config.active_profile = selected_profile;
                                    self.config.language = saved_lang;
                                    self.config.macros = saved_macros;
                                    self.config.auto_profile_rules = saved_rules;
                                    self.config.auto_profile_enabled = saved_auto_profile;
                                    self.config.dongle_rgb = saved_dongle;
                                }
                                self.sync_dpi_text_inputs();
                                if let Some(ref dev) = self.device {
                                    let _ = dev.commit_config(selected_profile);
                                }
                                self.set_status(
                                    format!(
                                        "Switched to Profile {} ({})",
                                        selected_profile,
                                        self.profile_names[(selected_profile as usize).saturating_sub(1)]
                                    ),
                                    true,
                                );
                            }

                            // Clean, readable Rename button
                            let rename_btn = egui::Button::new(
                                egui::RichText::new(self.config.language.rename())
                                    .size(11.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(2, 132, 199)),
                            )
                            .fill(egui::Color32::from_rgb(240, 249, 255))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(186, 230, 253)))
                            .rounding(egui::Rounding::same(4.0));

                            if ui
                                .add(rename_btn)
                                .on_hover_text("Rename active profile")
                                .clicked()
                            {
                                self.is_renaming_profile = true;
                                let idx = (self.config.active_profile as usize).saturating_sub(1);
                                self.profile_rename_buf = self
                                    .profile_names
                                    .get(idx)
                                    .cloned()
                                    .unwrap_or_else(|| format!("Profile {}", idx + 1));
                            }
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Language Quick Toggle Dropdown
                        let lang = self.config.language;
                        let mut selected_lang = lang;
                        let flag_text = match lang {
                            AppLanguage::English => "🌐 EN",
                            AppLanguage::Indonesian => "🌐 ID",
                        };
                        egui::ComboBox::from_id_salt("header_lang_combo")
                            .selected_text(
                                egui::RichText::new(flag_text)
                                    .size(11.5)
                                    .strong()
                                    .color(egui::Color32::from_rgb(2, 132, 199)),
                            )
                            .width(82.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut selected_lang,
                                    AppLanguage::English,
                                    "🇺🇸 English (US)",
                                );
                                ui.selectable_value(
                                    &mut selected_lang,
                                    AppLanguage::Indonesian,
                                    "🇮🇩 Bahasa Indonesia",
                                );
                            });

                        if selected_lang != lang {
                            self.config.language = selected_lang;
                            for p in &mut self.profiles {
                                p.language = selected_lang;
                            }
                            let msg = match selected_lang {
                                AppLanguage::English => "Language set to English (US)",
                                AppLanguage::Indonesian => "Bahasa diatur ke Bahasa Indonesia",
                            };
                            self.set_status(msg, true);
                        }

                        ui.add_space(8.0);

                        // Scan Devices Button
                        let scan_btn = egui::Button::new(
                            egui::RichText::new(lang.scan_devices())
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

                        // Battery Indicator Pill with multi-state icons and colors
                        if let Some(battery) = self.battery_level {
                            let is_charging = self.is_charging;
                            let is_critical = battery <= 10 && !is_charging;
                            let is_low = battery <= 20 && !is_charging && !is_critical;
                            let pulse = (self.start_time.elapsed().as_millis() % 900) < 450;

                            let (bg_color, stroke_color, fg_color, bar_color, label_text, pill_width, tooltip_title, tooltip_desc) = if is_charging {
                                let label = if !self.is_wireless {
                                    format!("{}% USB", battery)
                                } else {
                                    format!("{}% Charging", battery)
                                };
                                (
                                    egui::Color32::from_rgb(240, 249, 255),
                                    egui::Color32::from_rgb(56, 189, 248),
                                    egui::Color32::from_rgb(2, 132, 199),
                                    egui::Color32::from_rgb(2, 132, 199),
                                    label,
                                    124.0,
                                    lang.battery_usb_charging(),
                                    lang.battery_usb_charging_desc(),
                                )
                            } else if is_critical {
                                (
                                    if pulse {
                                        egui::Color32::from_rgb(254, 202, 202)
                                    } else {
                                        egui::Color32::from_rgb(254, 242, 242)
                                    },
                                    egui::Color32::from_rgb(220, 38, 38),
                                    egui::Color32::from_rgb(153, 27, 27),
                                    if pulse {
                                        egui::Color32::from_rgb(220, 38, 38)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    },
                                    lang.battery_critical_tag(battery),
                                    134.0,
                                    lang.battery_critical_title(),
                                    lang.battery_critical_desc(),
                                )
                            } else if is_low {
                                (
                                    egui::Color32::from_rgb(254, 242, 242),
                                    egui::Color32::from_rgb(248, 113, 113),
                                    egui::Color32::from_rgb(185, 28, 28),
                                    egui::Color32::from_rgb(239, 68, 68),
                                    lang.battery_low_tag(battery),
                                    122.0,
                                    lang.battery_low_title(),
                                    lang.battery_low_desc(),
                                )
                            } else if battery <= 50 {
                                (
                                    egui::Color32::from_rgb(254, 243, 199),
                                    egui::Color32::from_rgb(245, 158, 11),
                                    egui::Color32::from_rgb(180, 83, 9),
                                    egui::Color32::from_rgb(245, 158, 11),
                                    format!("{}%", battery),
                                    84.0,
                                    match lang {
                                        AppLanguage::English => "Moderate Battery",
                                        AppLanguage::Indonesian => "Baterai Sedang",
                                    },
                                    match lang {
                                        AppLanguage::English => "Battery level is sufficient for normal wireless usage.",
                                        AppLanguage::Indonesian => "Daya baterai mencukupi untuk penggunaan nirkabel normal.",
                                    },
                                )
                            } else {
                                (
                                    egui::Color32::from_rgb(240, 253, 244),
                                    egui::Color32::from_rgb(34, 197, 94),
                                    egui::Color32::from_rgb(21, 128, 61),
                                    egui::Color32::from_rgb(34, 197, 94),
                                    format!("{}%", battery),
                                    84.0,
                                    match lang {
                                        AppLanguage::English => "Battery Healthy",
                                        AppLanguage::Indonesian => "Baterai Optimal",
                                    },
                                    match lang {
                                        AppLanguage::English => "Battery is in prime condition and safely charged.",
                                        AppLanguage::Indonesian => "Daya baterai dalam kondisi prima dan terisi aman.",
                                    },
                                )
                            };

                            let (pill_rect, pill_resp) = ui
                                .allocate_exact_size(egui::vec2(pill_width, 28.0), egui::Sense::hover());
                            ui.painter().rect_filled(pill_rect, 6.0, bg_color);
                            ui.painter().rect_stroke(
                                pill_rect,
                                6.0,
                                egui::Stroke::new(if is_critical { 1.5_f32 } else { 1.0_f32 }, stroke_color),
                            );

                            // Vector Battery Casing Icon
                            let bat_x = pill_rect.left() + 7.0;
                            let bat_y = pill_rect.center().y - 6.0;
                            let bat_w = 20.0;
                            let bat_h = 12.0;
                            let body_rect = egui::Rect::from_min_size(
                                egui::pos2(bat_x, bat_y),
                                egui::vec2(bat_w, bat_h),
                            );
                            ui.painter().rect_stroke(
                                body_rect,
                                2.0,
                                egui::Stroke::new(1.2_f32, stroke_color),
                            );

                            // Terminal nipple on positive pole
                            let nub_rect = egui::Rect::from_min_size(
                                egui::pos2(bat_x + bat_w + 1.0, bat_y + 3.5),
                                egui::vec2(2.0, 5.0),
                            );
                            ui.painter().rect_filled(nub_rect, 1.0, stroke_color);

                            // Interior content of the battery icon
                            if is_charging {
                                // Interior charging fill
                                let fill_w = (bat_w - 4.0) * (battery.min(100) as f32 / 100.0);
                                if fill_w > 1.0 {
                                    let fill_rect = egui::Rect::from_min_size(
                                        egui::pos2(bat_x + 2.0, bat_y + 2.0),
                                        egui::vec2(fill_w, bat_h - 4.0),
                                    );
                                    ui.painter().rect_filled(
                                        fill_rect,
                                        1.0,
                                        egui::Color32::from_rgba_unmultiplied(2, 132, 199, 75),
                                    );
                                }

                                // Crisp vector Lightning Bolt (⚡)
                                let cx = body_rect.center().x;
                                let cy = body_rect.center().y;
                                let bolt_pts = vec![
                                    egui::pos2(cx + 0.5, cy - 4.5),
                                    egui::pos2(cx - 3.0, cy + 0.5),
                                    egui::pos2(cx - 0.5, cy + 0.5),
                                    egui::pos2(cx - 1.5, cy + 4.5),
                                    egui::pos2(cx + 3.0, cy - 0.5),
                                    egui::pos2(cx + 0.5, cy - 0.5),
                                ];
                                ui.painter().add(egui::Shape::convex_polygon(
                                    bolt_pts,
                                    egui::Color32::from_rgb(2, 132, 199),
                                    egui::Stroke::new(0.5_f32, egui::Color32::WHITE),
                                ));
                            } else {
                                // Progressive battery fill
                                let max_fill = bat_w - 4.0;
                                let fill_w = max_fill * (battery.min(100) as f32 / 100.0);
                                if fill_w >= 1.5 {
                                    let fill_rect = egui::Rect::from_min_size(
                                        egui::pos2(bat_x + 2.0, bat_y + 2.0),
                                        egui::vec2(fill_w, bat_h - 4.0),
                                    );
                                    ui.painter().rect_filled(fill_rect, 1.0, bar_color);
                                }
                            }

                            // Text Label
                            ui.painter().text(
                                egui::pos2(bat_x + bat_w + 8.0, pill_rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                label_text,
                                egui::FontId::proportional(11.5),
                                fg_color,
                            );

                            // Rich Interactive Tooltip on Hover
                            pill_resp.on_hover_ui(|ui| {
                                ui.set_max_width(260.0);
                                ui.label(egui::RichText::new(tooltip_title).strong().color(fg_color));
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new(tooltip_desc)
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(100, 116, 139)),
                                );
                                if let Some(mv) = self.battery_voltage_mv {
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(
                                            lang.voltage_label(mv as f32 / 1000.0, mv),
                                        )
                                        .size(10.5)
                                        .color(egui::Color32::from_rgb(148, 163, 184)),
                                    );
                                }
                            });

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
                            pill_resp.on_hover_text(self.config.language.dongle_offline_hover());
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
                                egui::include_image!("../../assets/Dialog/HomeChecked.png"),
                                self.config.language.tab_buttons(),
                            ),
                            (Tab::Buttons, false) => {
                                (egui::include_image!("../../assets/Dialog/Home.png"), self.config.language.tab_buttons())
                            }
                            (Tab::Sensor, true) => (
                                egui::include_image!("../../assets/Dialog/SensorChecked.png"),
                                self.config.language.tab_sensor(),
                            ),
                            (Tab::Sensor, false) => (
                                egui::include_image!("../../assets/Dialog/Sensor.png"),
                                self.config.language.tab_sensor(),
                            ),
                            (Tab::Lighting, true) => (
                                egui::include_image!("../../assets/Dialog/LightChecked.png"),
                                self.config.language.tab_lighting(),
                            ),
                            (Tab::Lighting, false) => (
                                egui::include_image!("../../assets/Dialog/Light.png"),
                                self.config.language.tab_lighting(),
                            ),
                            (Tab::Macro, true) => (
                                egui::include_image!("../../assets/Dialog/MacroChecked.png"),
                                self.config.language.tab_macro(),
                            ),
                            (Tab::Macro, false) => {
                                (egui::include_image!("../../assets/Dialog/Macro.png"), self.config.language.tab_macro())
                            }
                            (Tab::Settings, true) => (
                                egui::include_image!("../../assets/Dialog/SettingClick.png"),
                                self.config.language.tab_settings(),
                            ),
                            (Tab::Settings, false) => (
                                egui::include_image!("../../assets/Dialog/Setting.png"),
                                self.config.language.tab_settings(),
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

                        let unsaved_label = match self.config.language {
                            AppLanguage::English => "Unsaved Changes",
                            AppLanguage::Indonesian => "Belum Disimpan",
                        };
                        ui.painter().text(
                            egui::pos2(unsaved_pill.left() + 20.0, unsaved_pill.center().y),
                            egui::Align2::LEFT_CENTER,
                            unsaved_label,
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
                            egui::RichText::new(self.config.language.apply_changes_btn())
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
                            egui::RichText::new(self.config.language.read_from_mouse_btn())
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
                        let reset_label = match self.config.language {
                            AppLanguage::English => "Reset Defaults",
                            AppLanguage::Indonesian => "Kembalikan Standar",
                        };
                        let reset_hover = match self.config.language {
                            AppLanguage::English => "Reset configuration to factory defaults",
                            AppLanguage::Indonesian => "Kembalikan pengaturan ke standar pabrik",
                        };
                        let reset_btn = egui::Button::new(
                            egui::RichText::new(reset_label)
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

                        if ui.add(reset_btn).on_hover_text(reset_hover).clicked() {
                            let cur_lang = self.config.language;
                            let cur_profile = self.config.active_profile;
                            self.config = MouseConfig::default();
                            self.config.language = cur_lang;
                            self.config.active_profile = cur_profile;
                            self.selected_macro_idx = 0;
                            self.is_recording_macro = false;
                            self.binding_button_idx = None;
                            self.sync_dpi_text_inputs();
                            self.sync_active_profile_slot();
                            let reset_msg = match cur_lang {
                                AppLanguage::English => {
                                    "Reset to factory defaults (click Apply to save)"
                                }
                                AppLanguage::Indonesian => {
                                    "Kembalikan ke standar pabrik (klik Terapkan untuk simpan)"
                                }
                            };
                            self.set_status(reset_msg, true);
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
