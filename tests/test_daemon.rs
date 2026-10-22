//! Tests for daemon battery export, auto-profile rules, and serialization.

use icarus_v2mini::config::{AutoProfileRule, MouseConfig};
use icarus_v2mini::daemon::{BatteryExport, DaemonConfig};

#[test]
fn test_auto_profile_default_rules() {
    let cfg = MouseConfig::default();
    assert!(!cfg.auto_profile_enabled);
    assert_eq!(cfg.auto_profile_rules.len(), 3);

    // Rule 1: CS2
    assert_eq!(cfg.auto_profile_rules[0].app_pattern, "cs2");
    assert_eq!(cfg.auto_profile_rules[0].profile_index, 3);

    // Rule 2: Dota 2
    assert_eq!(cfg.auto_profile_rules[1].app_pattern, "dota2");
    assert_eq!(cfg.auto_profile_rules[1].profile_index, 4);

    // Rule 3: Blender
    assert_eq!(cfg.auto_profile_rules[2].app_pattern, "blender");
    assert_eq!(cfg.auto_profile_rules[2].profile_index, 5);
}

#[test]
fn test_auto_profile_rule_serialization_roundtrip() {
    let mut cfg = MouseConfig {
        auto_profile_enabled: true,
        ..Default::default()
    };
    cfg.auto_profile_rules.push(AutoProfileRule {
        app_pattern: "valorant".to_string(),
        profile_index: 2,
        description: "Valorant Comp".to_string(),
    });

    let toml_str = toml::to_string_pretty(&cfg).expect("Serialize to TOML");
    let parsed: MouseConfig = toml::from_str(&toml_str).expect("Deserialize from TOML");

    assert!(parsed.auto_profile_enabled);
    assert_eq!(parsed.auto_profile_rules.len(), 4);
    assert_eq!(parsed.auto_profile_rules[3].app_pattern, "valorant");
    assert_eq!(parsed.auto_profile_rules[3].profile_index, 2);
    assert_eq!(parsed.auto_profile_rules[3].description, "Valorant Comp");
}

#[test]
fn test_battery_export_json_schema() {
    let export = BatteryExport {
        text: "󰂀 75%".to_string(),
        tooltip: "ICARUS V2 MINI: 75% (3950 mV) [Profile 1]".to_string(),
        percentage: 75,
        charging: false,
        voltage_mv: 3950,
        class: "normal".to_string(),
        active_profile: 1,
    };

    let json_str = serde_json::to_string_pretty(&export).expect("Serialize BatteryExport");
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("Parse JSON");

    assert_eq!(val["text"], "󰂀 75%");
    assert_eq!(val["percentage"], 75);
    assert_eq!(val["charging"], false);
    assert_eq!(val["voltage_mv"], 3950);
    assert_eq!(val["class"], "normal");
    assert_eq!(val["active_profile"], 1);
    assert!(val["tooltip"].as_str().unwrap().contains("3950 mV"));
}

#[test]
fn test_battery_export_charging_class() {
    let export = BatteryExport {
        text: "󰂄 100%".to_string(),
        tooltip: "ICARUS V2 MINI: 100% (Charging) [Profile 2]".to_string(),
        percentage: 100,
        charging: true,
        voltage_mv: 4200,
        class: "charging".to_string(),
        active_profile: 2,
    };

    let json_str = serde_json::to_string(&export).expect("Serialize BatteryExport");
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("Parse JSON");

    assert_eq!(val["charging"], true);
    assert_eq!(val["class"], "charging");
}

#[test]
fn test_daemon_config_builder() {
    let default_cfg = DaemonConfig::default();
    assert_eq!(default_cfg.interval_secs, 10);
    assert!(default_cfg.enable_notify);
    assert!(default_cfg.auto_profile);

    let custom = DaemonConfig::new()
        .with_interval(30)
        .with_notifications(false)
        .with_auto_profile(false);

    assert_eq!(custom.interval_secs, 30);
    assert!(!custom.enable_notify);
    assert!(!custom.auto_profile);
}
