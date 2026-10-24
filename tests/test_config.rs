//! Tests for mouse configuration parsing, validation, TOML serialization, and enums.

use icarus_v2mini::config::{ButtonAction, LedMode, LiftOffDistance, MouseConfig, PollingRate};

#[test]
fn test_polling_rate_roundtrip() {
    for &rate in PollingRate::all() {
        let code = rate.to_code();
        let parsed = PollingRate::from_code(code).unwrap();
        assert_eq!(rate, parsed);
    }
}

#[test]
fn test_lod_roundtrip() {
    let low = LiftOffDistance::Low;
    let high = LiftOffDistance::High;
    assert_eq!(LiftOffDistance::from_code(low.to_code()), Some(low));
    assert_eq!(LiftOffDistance::from_code(high.to_code()), Some(high));
    assert_eq!(LiftOffDistance::from_code(0xFF), None);
}

#[test]
fn test_button_action_roundtrip() {
    let left = ButtonAction::MouseButton(1);
    let wire = left.to_bytes();
    assert_eq!(ButtonAction::from_bytes(wire), left);

    let kb = ButtonAction::Keyboard {
        usage: 0x04,
        modifier: 0x01,
    };
    let wire_kb = kb.to_bytes();
    assert_eq!(ButtonAction::from_bytes(wire_kb), kb);

    let dpi = ButtonAction::DpiSwitch;
    let wire_dpi = dpi.to_bytes();
    assert_eq!(ButtonAction::from_bytes(wire_dpi), dpi);

    let fire = ButtonAction::FireKey {
        clicks: 3,
        delay_ms: 12,
    };
    let wire_fire = fire.to_bytes();
    assert_eq!(ButtonAction::from_bytes(wire_fire), fire);
}

#[test]
fn test_led_mode_neon() {
    assert_eq!(LedMode::Neon.to_code(), 0x03);
    assert_eq!(LedMode::from_code(0x03), Some(LedMode::Neon));
}

#[test]
fn test_config_toml_roundtrip() {
    let default_cfg = MouseConfig::default();
    let toml_str = toml::to_string_pretty(&default_cfg).expect("Serialize TOML");
    let parsed_cfg: MouseConfig = toml::from_str(&toml_str).expect("Deserialize TOML");
    assert_eq!(default_cfg.polling_rate, parsed_cfg.polling_rate);
    assert_eq!(default_cfg.active_dpi_stage, parsed_cfg.active_dpi_stage);
    assert_eq!(default_cfg.debounce_ms, parsed_cfg.debounce_ms);
    assert_eq!(default_cfg.lod, parsed_cfg.lod);
    assert_eq!(default_cfg.dpi_stages.len(), parsed_cfg.dpi_stages.len());
}

#[test]
fn test_parse_example_profile() {
    let example = include_str!("../examples/profile-default.toml");
    let parsed: MouseConfig = toml::from_str(example).expect("Failed to parse example profile");
    assert_eq!(parsed.polling_rate, PollingRate::Hz1000);
    assert_eq!(parsed.dpi_stages.len(), 5);
    assert_eq!(parsed.buttons.len(), 6);
}

#[test]
fn test_config_validation() {
    let mut cfg = MouseConfig {
        active_profile: 99, // invalid
        debounce_ms: 55,    // invalid
        ..Default::default()
    };
    cfg.buttons.clear(); // empty buttons
    cfg.dpi_stages[cfg.active_dpi_stage as usize].enabled = false; // active stage disabled
    cfg.dpi_led.brightness = 99; // invalid
    cfg.dpi_led.breath_speed = 0; // invalid

    cfg.validate();

    assert_eq!(cfg.active_profile, 1);
    assert_eq!(cfg.debounce_ms, 20);
    assert_eq!(cfg.buttons.len(), 6);
    assert_eq!(cfg.buttons[0], ButtonAction::MouseButton(1)); // Left Click
    assert_eq!(cfg.buttons[1], ButtonAction::MouseButton(2)); // Right Click
    assert_eq!(cfg.buttons[2], ButtonAction::MouseButton(4)); // Middle Click
    assert_eq!(cfg.buttons[3], ButtonAction::MouseButton(16)); // Forward (Button 4)
    assert_eq!(cfg.buttons[4], ButtonAction::MouseButton(8)); // Back (Button 5)
    assert_eq!(cfg.buttons[5], ButtonAction::DpiSwitch); // DPI Cycle (Button 6)
    assert!(cfg.dpi_stages[cfg.active_dpi_stage as usize].enabled);
    assert_eq!(cfg.dpi_led.brightness, 10);
    assert_eq!(cfg.dpi_led.breath_speed, 1);
}

#[test]
fn test_left_click_safety_lock() {
    let mut cfg = MouseConfig::default();
    assert!(cfg.validate_buttons().is_ok());

    // Remap all buttons to Disabled (removing Left Click completely)
    cfg.buttons.fill(ButtonAction::Disabled);
    assert!(cfg.validate_buttons().is_err());

    // Normalize / validate automatically restores Left Click to Button 1
    cfg.validate();
    assert_eq!(cfg.buttons[0], ButtonAction::MouseButton(1));
    assert!(cfg.validate_buttons().is_ok());
}

#[test]
fn test_separate_xy_dpi_and_effective_values() {
    use icarus_v2mini::config::DpiStage;

    let mut stage = DpiStage {
        dpi: 800,
        dpi_y: Some(1600),
        enabled: true,
        color: icarus_v2mini::config::Color::RED,
    };
    assert_eq!(stage.effective_x(), 800);
    assert_eq!(stage.effective_y(), 1600);

    stage.dpi_y = None;
    assert_eq!(stage.effective_x(), 800);
    assert_eq!(stage.effective_y(), 800);

    let mut cfg = MouseConfig {
        separate_xy_dpi: true,
        ..Default::default()
    };
    cfg.dpi_stages[0].dpi = 799; // should snap to 800
    cfg.dpi_stages[0].dpi_y = Some(1603); // should snap to 1600
    cfg.validate();
    assert_eq!(cfg.dpi_stages[0].dpi, 800);
    assert_eq!(cfg.dpi_stages[0].dpi_y, Some(1600));
}

#[test]
fn test_dongle_rgb_config_defaults_and_roundtrip() {
    use icarus_v2mini::config::{Color, DongleRgbConfig};

    let cfg = DongleRgbConfig::default();
    assert!(cfg.enabled);
    assert_eq!(cfg.mode, LedMode::Solid);
    assert_eq!(cfg.brightness, 10);
    assert_eq!(cfg.speed, 5);
    assert_eq!(cfg.color, Color::CYAN);
    assert_eq!(cfg.bar_mode, 0);

    let serialized = toml::to_string_pretty(&cfg).expect("Serialize DongleRgbConfig");
    let deserialized: DongleRgbConfig =
        toml::from_str(&serialized).expect("Deserialize DongleRgbConfig");
    assert_eq!(cfg, deserialized);
}

#[test]
fn test_button_action_display_formatting() {
    let kb = ButtonAction::Keyboard {
        usage: 0x04,           // 'A'
        modifier: 0x01 | 0x02, // Ctrl + Shift
    };
    assert_eq!(format!("{}", kb), "Ctrl+Shift+A");

    let media = ButtonAction::Media(0xCD); // Play/Pause
    assert_eq!(format!("{}", media), "Play / Pause");

    let macro_act = ButtonAction::Macro(2);
    assert_eq!(format!("{}", macro_act), "Macro #2");

    let fire = ButtonAction::FireKey {
        clicks: 3,
        delay_ms: 12,
    };
    assert_eq!(format!("{}", fire), "Fire Key (3x, 12ms)");
}

#[test]
fn test_mouse_config_helpers_and_derived_traits() {
    let cfg = MouseConfig::default();
    let toml_str = cfg.to_toml().expect("to_toml should succeed");
    let parsed = MouseConfig::from_toml(&toml_str).expect("from_toml should succeed");
    assert_eq!(cfg, parsed);

    // Test file I/O helpers via a temporary file
    let tmp_path = std::env::temp_dir().join("test_icarus_config_roundtrip.toml");
    cfg.to_file(&tmp_path).expect("to_file should succeed");
    let loaded = MouseConfig::from_file(&tmp_path).expect("from_file should succeed");
    assert_eq!(cfg, loaded);
    let _ = std::fs::remove_file(tmp_path);

    // Verify DpiLed Default
    let default_led = icarus_v2mini::config::DpiLed::default();
    assert_eq!(default_led.mode, LedMode::Solid);
    assert_eq!(default_led.brightness, 0);
}

#[test]
fn test_default_dpi_stages_and_validation() {
    let stages = icarus_v2mini::config::default_dpi_stages();
    assert_eq!(stages.len(), 5);
    assert_eq!(stages[0].dpi, 400);
    assert_eq!(stages[1].dpi, 800);
    assert_eq!(stages[2].dpi, 1600);
    assert_eq!(stages[3].dpi, 3200);
    assert_eq!(stages[4].dpi, 6400);

    let mut cfg = MouseConfig::default();
    cfg.dpi_stages.clear();
    cfg.validate();
    // Validating an empty dpi_stages should restore the standard default stages without erasing other settings
    assert_eq!(cfg.dpi_stages.len(), 5);
    assert_eq!(cfg.dpi_stages[0].dpi, 400);
}

#[test]
fn test_color_constants_and_dongle_rgb_validation() {
    use icarus_v2mini::config::Color;

    assert_eq!(Color::MAGENTA, Color::new(255, 0, 255));
    assert_eq!(Color::PURPLE, Color::new(128, 0, 255));
    assert_ne!(Color::PURPLE, Color::CYAN);

    let mut cfg = MouseConfig::default();
    cfg.dongle_rgb.brightness = 100;
    cfg.dongle_rgb.speed = 0;
    cfg.dongle_rgb.bar_mode = 9;
    cfg.dongle_rgb.mode = LedMode::Off;
    cfg.dongle_rgb.enabled = true;
    cfg.buttons[3] = ButtonAction::Macro(99);
    cfg.buttons[4] = ButtonAction::FireKey {
        clicks: 1,
        delay_ms: 1,
    };
    cfg.validate();

    assert_eq!(cfg.dongle_rgb.brightness, 10);
    assert_eq!(cfg.dongle_rgb.speed, 1);
    assert_eq!(cfg.dongle_rgb.bar_mode, 0);
    assert!(!cfg.dongle_rgb.enabled);
    assert_eq!(
        cfg.buttons[3],
        ButtonAction::Macro((cfg.macros.len() - 1) as u8)
    );
    assert_eq!(
        cfg.buttons[4],
        ButtonAction::FireKey {
            clicks: 2,
            delay_ms: 5
        }
    );
}

#[test]
fn test_cli_helpers_utf8_safety_and_battery_bar_bounds() {
    use icarus_v2mini::cli::commands::info::battery_bar;
    use icarus_v2mini::cli::helpers::parse_hex_color;
    use icarus_v2mini::config::Color;

    // 6-byte multi-byte UTF-8 string ("€€" = 3 + 3 bytes) must return Err, never panic!
    assert!(parse_hex_color("€€").is_err());
    assert!(parse_hex_color("äöü").is_err());
    assert_eq!(
        parse_hex_color("#00F0FF").unwrap(),
        Color::new(0x00, 0xF0, 0xFF)
    );

    // Battery levels > 100 must clamp safely without arithmetic underflow or panic
    let bar_100 = battery_bar(100);
    let bar_255 = battery_bar(255);
    assert_eq!(bar_100, bar_255);
}

#[test]
fn test_extended_button_display_and_macro_rule_validation() {
    use icarus_v2mini::config::{
        AutoProfileRule, MacroEvent, MacroEventType, MacroItem, MacroRepeatMode,
    };

    // Punctuation & extended function keys use zero-allocation protocol lookup
    let slash_kb = ButtonAction::Keyboard {
        usage: 0x38,
        modifier: 0x01,
    };
    assert_eq!(format!("{}", slash_kb), "Ctrl+/");

    let f16_kb = ButtonAction::Keyboard {
        usage: 0x6B,
        modifier: 0,
    };
    assert_eq!(format!("{}", f16_kb), "F16");

    // Extended media codes
    assert_eq!(format!("{}", ButtonAction::Media(0x83)), "Media Player");
    assert_eq!(format!("{}", ButtonAction::Media(0x24)), "Browser Back");
    assert_eq!(format!("{}", ButtonAction::Media(0x25)), "Browser Forward");

    // Validate macro sanitization, auto-profile clamping, and separate_xy_dpi auto-detection
    let mut cfg = MouseConfig::default();
    cfg.dpi_stages[0].dpi = 800;
    cfg.dpi_stages[0].dpi_y = Some(1600);

    cfg.macros.push(MacroItem {
        name: "   ".to_string(),
        events: (0..80)
            .map(|i| MacroEvent {
                event_type: MacroEventType::Delay,
                key: String::from("ignored"),
                delay_ms: if i == 0 { 0 } else { 100_000 },
            })
            .collect(),
        repeat_mode: MacroRepeatMode::Times(0),
        auto_insert_delay: true,
        default_delay_ms: 0,
    });
    cfg.auto_profile_rules.push(AutoProfileRule {
        app_pattern: "test".to_string(),
        profile_index: 99,
        description: "TestApp".to_string(),
    });

    cfg.validate();

    assert!(cfg.separate_xy_dpi);
    let last_mac = cfg.macros.last().unwrap();
    assert!(!last_mac.name.is_empty());
    assert_eq!(last_mac.events.len(), 64);
    assert_eq!(last_mac.events[0].delay_ms, 1);
    assert!(last_mac.events[0].key.is_empty());
    assert_eq!(last_mac.events[1].delay_ms, 65_535);
    assert_eq!(last_mac.repeat_mode, MacroRepeatMode::Times(1));
    assert_eq!(last_mac.default_delay_ms, 1);
    assert_eq!(cfg.auto_profile_rules.last().unwrap().profile_index, 5);
}
