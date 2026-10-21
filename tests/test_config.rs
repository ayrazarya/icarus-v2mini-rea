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
