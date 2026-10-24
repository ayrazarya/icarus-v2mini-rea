use std::time::Instant;

use eframe::egui;

use crate::config::{AppLanguage, MouseConfig};
use crate::gui::state::{IcarusApp, PairingState};
use crate::gui::theme::toggle_switch;
use crate::gui::widgets::pick_file_zenity;

impl IcarusApp {
    /// Render Tab 5: Settings, 2.4G Pairing Tool, Profile Import/Export, and Linux Udev Rules
    pub(crate) fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left Column: Interface Language, 2.4G Pairing Tool & Profile Management
            ui.vertical(|ui| {
                ui.set_width(380.0);

                // Interface Language Setting Card
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new(self.config.language.language_setting_title())
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label(self.config.language.language_setting_desc());
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        let is_en = self.config.language == AppLanguage::English;
                        let is_id = self.config.language == AppLanguage::Indonesian;

                        let en_btn = egui::Button::new(
                            egui::RichText::new("🇺🇸 English (US)")
                                .size(12.0)
                                .strong()
                                .color(if is_en {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                }),
                        )
                        .fill(if is_en {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            if is_en {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(203, 213, 225)
                            },
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(150.0, 32.0));

                        if ui.add(en_btn).clicked() {
                            self.config.language = AppLanguage::English;
                            for p in &mut self.profiles {
                                p.language = AppLanguage::English;
                            }
                            self.set_status("Display language set to English", true);
                        }

                        let id_btn = egui::Button::new(
                            egui::RichText::new("🇮🇩 Bahasa Indonesia")
                                .size(12.0)
                                .strong()
                                .color(if is_id {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                }),
                        )
                        .fill(if is_id {
                            egui::Color32::from_rgb(2, 132, 199)
                        } else {
                            egui::Color32::from_rgb(241, 245, 249)
                        })
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            if is_id {
                                egui::Color32::from_rgb(2, 132, 199)
                            } else {
                                egui::Color32::from_rgb(203, 213, 225)
                            },
                        ))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(150.0, 32.0));

                        if ui.add(id_btn).clicked() {
                            self.config.language = AppLanguage::Indonesian;
                            for p in &mut self.profiles {
                                p.language = AppLanguage::Indonesian;
                            }
                            self.set_status("Bahasa tampilan diatur ke Bahasa Indonesia", true);
                        }
                    });
                });

                ui.add_space(10.0);

                // 2.4G Wireless Pairing Tool Card
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new(self.config.language.dongle_pairing_title())
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    let pairing_sub = match self.config.language {
                        AppLanguage::English => "Pair your ICARUS V2 MINI mouse to its 2.4G/4G wireless dongle:",
                        AppLanguage::Indonesian => "Pasangkan mouse ICARUS V2 MINI ke dongle nirkabel 2.4G/4K:",
                    };
                    ui.label(pairing_sub);
                    ui.add_space(8.0);

                    // Pairing status indicator pill
                    let (status_badge, status_desc, badge_bg, badge_fg, badge_stroke) = match self.pairing_state {
                        PairingState::Idle => {
                            let (badge, desc) = match self.config.language {
                                AppLanguage::English => ("READY", "Dongle is waiting for pairing command"),
                                AppLanguage::Indonesian => ("SIAP", "Dongle siap menunggu perintah pemasangan"),
                            };
                            (badge, desc, egui::Color32::from_rgb(224, 242, 254), egui::Color32::from_rgb(2, 132, 199), egui::Color32::from_rgb(186, 230, 253))
                        }
                        PairingState::Pairing => {
                            let (badge, desc) = match self.config.language {
                                AppLanguage::English => ("PAIRING...", "Keep mouse close to dongle (< 10cm)"),
                                AppLanguage::Indonesian => ("MEMASANGKAN...", "Dekatkan mouse ke dongle (< 10cm)"),
                            };
                            (badge, desc, egui::Color32::from_rgb(254, 243, 199), egui::Color32::from_rgb(180, 83, 9), egui::Color32::from_rgb(253, 230, 138))
                        }
                        PairingState::Success => {
                            let (badge, desc) = match self.config.language {
                                AppLanguage::English => ("PAIRED", "Dongle Paired Successfully!"),
                                AppLanguage::Indonesian => ("TERPASANG", "Dongle Berhasil Dipasangkan!"),
                            };
                            (badge, desc, egui::Color32::from_rgb(240, 253, 244), egui::Color32::from_rgb(22, 163, 74), egui::Color32::from_rgb(187, 247, 208))
                        }
                        PairingState::Failed => {
                            let (badge, desc) = match self.config.language {
                                AppLanguage::English => ("FAILED", "Pairing Failed. Try again."),
                                AppLanguage::Indonesian => ("GAGAL", "Pemasangan Gagal. Coba lagi."),
                            };
                            (badge, desc, egui::Color32::from_rgb(254, 242, 242), egui::Color32::from_rgb(220, 38, 38), egui::Color32::from_rgb(254, 202, 202))
                        }
                    };

                    ui.horizontal(|ui| {
                        let (badge_r, _) = ui.allocate_exact_size(egui::vec2(86.0, 22.0), egui::Sense::hover());
                        ui.painter().rect_filled(badge_r, 4.0, badge_bg);
                        ui.painter().rect_stroke(badge_r, 4.0, egui::Stroke::new(1.0_f32, badge_stroke));
                        ui.painter().text(
                            badge_r.center(),
                            egui::Align2::CENTER_CENTER,
                            status_badge,
                            egui::FontId::proportional(11.0),
                            badge_fg,
                        );
                        ui.label(status_desc);
                    });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        let (pair_btn_label, reset_state_label) = match self.config.language {
                            AppLanguage::English => ("Start 2.4G Pairing (Match)", "Reset State"),
                            AppLanguage::Indonesian => ("Mulai Pairing 2.4G (Cocokkan)", "Reset Status"),
                        };

                        if ui.button(pair_btn_label).clicked() {
                            self.pairing_state = PairingState::Pairing;
                            let now = Instant::now();
                            self.pairing_start = Some(now);
                            self.last_pairing_poll = now;
                            if let Some(ref dev) = self.device {
                                match dev.pair_dongle(None, None) {
                                    Ok(_) => {
                                        let ok_msg = match self.config.language {
                                            AppLanguage::English => "Hardware pairing command sent! Bring mouse close to dongle (< 10cm).",
                                            AppLanguage::Indonesian => "Perintah pairing hardware dikirim! Dekatkan mouse ke dongle (< 10cm).",
                                        };
                                        self.set_status(ok_msg, true);
                                    }
                                    Err(e) => {
                                        self.set_status(self.config.language.status_pairing_err(&e.to_string()), false);
                                        self.pairing_state = PairingState::Failed;
                                    }
                                }
                            } else {
                                let no_dongle_msg = match self.config.language {
                                    AppLanguage::English => "Connect USB dongle first to start pairing.",
                                    AppLanguage::Indonesian => "Hubungkan USB dongle terlebih dahulu untuk memulai pairing.",
                                };
                                self.set_status(no_dongle_msg, false);
                            }
                        }

                        if self.pairing_state != PairingState::Idle
                            && ui.button(reset_state_label).clicked()
                        {
                            self.pairing_state = PairingState::Idle;
                        }
                    });
                });

                ui.add_space(10.0);

                // Profile Management with Custom Path, Native File Dialog & Quick Presets
                ui.group(|ui| {
                    ui.heading(
                        egui::RichText::new("Profile Import / Export")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Save or load custom mouse configuration files (.toml):");
                    ui.add_space(6.0);

                    // File path input and Browse button
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.profile_custom_path)
                                .desired_width(220.0)
                                .hint_text("icarus-profile.toml"),
                        );
                        if ui
                            .button("Browse...")
                            .on_hover_text("Open native file dialog to choose a profile file")
                            .clicked()
                        {
                            let picked = pick_file_zenity(false, &self.profile_custom_path);
                            if let Some(path) = picked {
                                self.profile_custom_path = path;
                            }
                        }
                    });

                    ui.add_space(6.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        let export_btn = egui::Button::new(
                            egui::RichText::new("Export Profile (TOML)")
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(2, 132, 199))
                        .rounding(egui::Rounding::same(4.0));

                        if ui
                            .add(export_btn)
                            .on_hover_text("Save current configuration to the specified TOML file")
                            .clicked()
                        {
                            let path = self.profile_custom_path.trim();
                            let target_path = if path.is_empty() {
                                "icarus-profile.toml".to_owned()
                            } else {
                                path.to_owned()
                            };
                            match self.config.to_file(&target_path) {
                                Ok(()) => {
                                    self.set_status(
                                        format!("Saved profile to {}", target_path),
                                        true,
                                    );
                                }
                                Err(e) => {
                                    self.set_status(
                                        format!("Failed to save '{}': {}", target_path, e),
                                        false,
                                    );
                                }
                            }
                        }

                        let import_btn = egui::Button::new(
                            egui::RichText::new("Import Profile (TOML)")
                                .color(egui::Color32::from_rgb(15, 23, 42))
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(241, 245, 249))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(203, 213, 225),
                        ))
                        .rounding(egui::Rounding::same(4.0));

                        if ui
                            .add(import_btn)
                            .on_hover_text("Load configuration from the specified TOML file")
                            .clicked()
                        {
                            let path = self.profile_custom_path.trim();
                            let target_path = if path.is_empty() {
                                "icarus-profile.toml".to_owned()
                            } else {
                                path.to_owned()
                            };
                            match MouseConfig::from_file(&target_path) {
                                Ok(cfg) => {
                                    self.config = cfg;
                                    if self.selected_macro_idx >= self.config.macros.len() {
                                        self.selected_macro_idx = 0;
                                    }
                                    self.sync_dpi_text_inputs();
                                    self.sync_active_profile_slot();
                                    self.set_status(
                                        format!("Loaded profile from {}", target_path),
                                        true,
                                    );
                                }
                                Err(e) => {
                                    self.set_status(
                                        format!("Failed to load '{}': {}", target_path, e),
                                        false,
                                    );
                                }
                            }
                        }

                        if ui
                            .button("Save As...")
                            .on_hover_text("Open file save dialog to pick destination and save")
                            .clicked()
                        {
                            let picked = pick_file_zenity(true, &self.profile_custom_path);
                            if let Some(path) = picked {
                                self.profile_custom_path = path.clone();
                                match self.config.to_file(&path) {
                                    Ok(()) => {
                                        self.set_status(format!("Saved profile to {}", path), true);
                                    }
                                    Err(e) => {
                                        self.set_status(
                                            format!("Failed to save '{}': {}", path, e),
                                            false,
                                        );
                                    }
                                }
                            }
                        }
                    });

                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Quick Filename Presets:")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(100, 116, 139)),
                    );
                    ui.horizontal_wrapped(|ui| {
                        let presets = [
                            ("default.toml", "icarus-profile.toml"),
                            ("cs2.toml", "backup-cs2.toml"),
                            ("valorant.toml", "backup-valorant.toml"),
                            ("work.toml", "backup-work.toml"),
                        ];
                        for (label, filename) in presets {
                            let is_curr = self.profile_custom_path == filename;
                            let btn = egui::Button::new(
                                egui::RichText::new(label).size(11.0).color(if is_curr {
                                    egui::Color32::from_rgb(2, 132, 199)
                                } else {
                                    egui::Color32::from_rgb(71, 85, 105)
                                }),
                            )
                            .fill(if is_curr {
                                egui::Color32::from_rgb(224, 242, 254)
                            } else {
                                egui::Color32::from_rgb(241, 245, 249)
                            })
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                if is_curr {
                                    egui::Color32::from_rgb(2, 132, 199)
                                } else {
                                    egui::Color32::from_rgb(226, 232, 240)
                                },
                            ))
                            .rounding(egui::Rounding::same(4.0));
                            if ui.add(btn).clicked() {
                                self.profile_custom_path = filename.to_string();
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
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f511", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f511", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f512", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f512", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f510", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="3554", ATTR{idProduct}=="f510", MODE="0666", TAG+="uaccess"
"#;
                        match std::fs::write("99-icarus-mouse.rules", rule) {
                            Ok(()) => self.set_status(
                                "Generated 99-icarus-mouse.rules in current directory",
                                true,
                            ),
                            Err(e) => self.set_status(
                                format!("Failed to write 99-icarus-mouse.rules: {}", e),
                                false,
                            ),
                        }
                    }
                });

                ui.add_space(10.0);

                // Firmware Update (ISP Flasher) Card
                ui.group(|ui| {
                    ui.set_width(380.0);
                    ui.heading(
                        egui::RichText::new(self.config.language.firmware_flasher_title())
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );

                    let flasher_desc = match self.config.language {
                        AppLanguage::English => "Flash binary (.bin / .hex) firmware image to device:",
                        AppLanguage::Indonesian => "Flash file biner firmware (.bin / .hex) ke perangkat:",
                    };
                    ui.label(flasher_desc);
                    ui.add_space(4.0);

                    // Target radio buttons: Mouse MCU vs 4K Wireless Dongle
                    let (mouse_mcu_lbl, dongle_mcu_lbl) = match self.config.language {
                        AppLanguage::English => ("Mouse MCU (nRF52840)", "4K Dongle (CH32V305)"),
                        AppLanguage::Indonesian => ("MCU Mouse (nRF52840)", "Dongle 4K (CH32V305)"),
                    };
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Target:").strong().size(11.0));
                        ui.radio_value(&mut self.firmware_is_dongle, false, mouse_mcu_lbl);
                        ui.radio_value(&mut self.firmware_is_dongle, true, dongle_mcu_lbl);
                    });

                    ui.add_space(4.0);

                    // File path input and Browse button
                    let fw_browse_hover = match self.config.language {
                        AppLanguage::English => "Select firmware file (.bin or .hex)",
                        AppLanguage::Indonesian => "Pilih file firmware (.bin atau .hex)",
                    };
                    ui.horizontal(|ui| {
                        ui.label("File:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.firmware_update_path)
                                .desired_width(220.0)
                                .hint_text("firmware.bin"),
                        );
                        if ui
                            .button(self.config.language.browse_btn())
                            .on_hover_text(fw_browse_hover)
                            .clicked()
                        {
                            let picked = pick_file_zenity(false, "");
                            if let Some(path) = picked {
                                self.firmware_update_path = path;
                            }
                        }
                    });

                    ui.add_space(6.0);

                    // Safety warning notice
                    let warn_frame = egui::Frame::none()
                        .fill(egui::Color32::from_rgb(254, 242, 242))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(254, 202, 202)))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 5.0));
                    warn_frame.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(self.config.language.firmware_warning())
                                .size(10.5)
                                .color(egui::Color32::from_rgb(185, 28, 28)),
                        );
                    });

                    ui.add_space(6.0);

                    let flash_fw_btn = egui::Button::new(
                        egui::RichText::new(self.config.language.flash_firmware_now_btn())
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(220, 38, 38))
                    .rounding(egui::Rounding::same(4.0))
                    .min_size(egui::vec2(160.0, 26.0));

                    if ui.add(flash_fw_btn).clicked() {
                        let path = self.firmware_update_path.trim().to_string();
                        if path.is_empty() {
                            self.set_status(self.config.language.status_select_firmware_first(), false);
                        } else if let Some(ref dev) = self.device {
                            let is_dongle = self.firmware_is_dongle;
                            let res = dev.flash_firmware_file(std::path::Path::new(&path), is_dongle);
                            match res {
                                Ok(msg) => {
                                    self.set_status(self.config.language.status_firmware_flashed_ok(&msg), true);
                                }
                                Err(e) => {
                                    self.set_status(self.config.language.status_firmware_flash_err(&e.to_string()), false);
                                }
                            }
                        } else {
                            self.set_status(self.config.language.status_firmware_need_usb(), false);
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
                    ui.add_space(8.0);

                    let specs = [
                        ("Device", "ICARUS V2 MINI Wireless Gaming Mouse"),
                        ("Sensor", "PixArt PAW3395 (26,000 DPI, 650 IPS, 50G)"),
                        ("Mouse MCU", "Nordic nRF52840 (ARM Cortex-M4F @ 64MHz)"),
                        ("1K Dongle MCU", "CX52650N (2.4GHz Wireless)"),
                        ("4K Dongle MCU", "WCH CH32V305 (RISC-V High Speed)"),
                        ("USB Vendor ID", "0x3554"),
                        ("Wired PID", "0xF511 (USB Direct)"),
                        ("2.4G PID", "0xF512 (Standard Dongle)"),
                        ("4G PID", "0xF510 (High-Speed Dongle)"),
                        ("HID Endpoint", "Interface 1 (Vendor Specific)"),
                        ("Protocol Packet", "17 bytes (XOR Checksummed)"),
                    ];

                    for (k, v) in specs {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", k))
                                    .strong()
                                    .color(egui::Color32::from_rgb(71, 85, 105)),
                            );
                            ui.label(
                                egui::RichText::new(v).color(egui::Color32::from_rgb(15, 23, 42)),
                            );
                        });
                        ui.add_space(2.0);
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Firmware:")
                                .strong()
                                .color(egui::Color32::from_rgb(71, 85, 105)),
                        );
                        ui.label(&self.firmware_version);
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Connection:")
                                .strong()
                                .color(egui::Color32::from_rgb(71, 85, 105)),
                        );
                        ui.label(&self.connection_status);
                    });

                    if let Some(bat) = self.battery_level {
                        ui.horizontal(|ui| {
                            let (bat_title, charging_tag) = match self.config.language {
                                AppLanguage::English => ("Battery:", "Charging / USB Mode"),
                                AppLanguage::Indonesian => ("Baterai:", "Sedang Mengisi Daya / Mode USB"),
                            };
                            ui.label(
                                egui::RichText::new(bat_title)
                                    .strong()
                                    .color(egui::Color32::from_rgb(71, 85, 105)),
                            );
                            let status_str = if self.is_charging {
                                format!("{}% ({})", bat, charging_tag)
                            } else if bat <= 10 {
                                self.config.language.battery_critical_tag(bat)
                            } else if bat <= 20 {
                                self.config.language.battery_low_tag(bat)
                            } else {
                                format!("{}%", bat)
                            };
                            ui.label(status_str);
                        });
                    }
                });

                ui.add_space(10.0);

                // Native Linux Desktop Integration & Waybar Battery
                ui.group(|ui| {
                    ui.set_width(400.0);
                    ui.heading(
                        egui::RichText::new("Linux Desktop & Battery Integration")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.label("Export battery JSON to Waybar/Polybar and enable low-battery notifications:");
                    ui.add_space(4.0);

                    let cache_path = crate::daemon::get_battery_cache_path();
                    ui.label(
                        egui::RichText::new(format!("Cache: {}", cache_path.display()))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(100, 116, 139)),
                    );

                    ui.add_space(4.0);
                    if ui
                        .button("Install Background Systemd Service")
                        .on_hover_text("Registers user systemd service to run daemon on login")
                        .clicked()
                    {
                        match crate::daemon::install_systemd_user_service() {
                            Ok(_) => self.set_status("Systemd user service 'icarus-v2mini.service' installed and active!", true),
                            Err(e) => self.set_status(format!("Failed to install service: {}", e), false),
                        }
                    }
                });

                ui.add_space(10.0);

                // Auto-Profile Switching (Game Detection)
                ui.group(|ui| {
                    ui.set_width(400.0);
                    ui.heading(
                        egui::RichText::new("Auto-Profile Switching (Per-Game)")
                            .color(egui::Color32::from_rgb(2, 132, 199)),
                    );
                    ui.horizontal(|ui| {
                        toggle_switch(ui, &mut self.config.auto_profile_enabled);
                        ui.label("Enable Automatic Profile Switching");
                    });
                    ui.add_space(4.0);

                    let mut rule_to_remove = None;
                    for (idx, rule) in self.config.auto_profile_rules.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("App:").strong().size(11.0));
                            ui.add(egui::TextEdit::singleline(&mut rule.app_pattern).desired_width(65.0));
                            ui.label(egui::RichText::new("Profile:").strong().size(11.0));
                            ui.add(egui::DragValue::new(&mut rule.profile_index).range(1..=5));
                            ui.add(egui::TextEdit::singleline(&mut rule.description).desired_width(110.0));
                            if ui.small_button("✕").clicked() {
                                rule_to_remove = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = rule_to_remove {
                        self.config.auto_profile_rules.remove(idx);
                    }

                    if ui.button("+ Add Game Rule").clicked() {
                        self.config.auto_profile_rules.push(crate::config::AutoProfileRule {
                            app_pattern: "game".into(),
                            profile_index: 2,
                            description: "New Game Profile".into(),
                        });
                    }
                });
            });
        });
    }
}
