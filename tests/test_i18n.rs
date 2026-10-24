//! Integration tests for internationalization (i18n) and bilingual language settings.

use icarus_v2mini::config::{ButtonAction, MouseConfig};
use icarus_v2mini::i18n::AppLanguage;

#[test]
fn test_app_language_variants_and_metadata() {
    let languages = AppLanguage::all();
    assert_eq!(languages.len(), 2);
    assert_eq!(languages[0], AppLanguage::English);
    assert_eq!(languages[1], AppLanguage::Indonesian);

    assert_eq!(AppLanguage::English.code(), "EN");
    assert_eq!(AppLanguage::Indonesian.code(), "ID");

    assert!(AppLanguage::English.display_name().contains("English"));
    assert!(AppLanguage::Indonesian.display_name().contains("Indonesia"));

    // Default must be English
    assert_eq!(AppLanguage::default(), AppLanguage::English);
}

#[test]
fn test_all_translations_non_empty() {
    for lang in AppLanguage::all() {
        assert!(!lang.tab_buttons().is_empty());
        assert!(!lang.tab_sensor().is_empty());
        assert!(!lang.tab_lighting().is_empty());
        assert!(!lang.tab_macro().is_empty());
        assert!(!lang.tab_settings().is_empty());

        assert!(!lang.disconnected().is_empty());
        assert!(!lang.connected_usb().is_empty());
        assert!(!lang.connected_24g().is_empty());
        assert!(!lang.connected_4k().is_empty());
        assert!(!lang.dongle_offline_hover().is_empty());

        assert!(!lang.button_remapping_title().is_empty());
        assert!(!lang.debounce_tester_title().is_empty());
        assert!(!lang.capture_key_btn().is_empty());
        assert!(!lang.listening_key_btn().is_empty());
        assert!(!lang.left_click_safety_lock().is_empty());
        assert!(!lang.status_left_click_lock_warn().is_empty());
        assert!(!lang.status_listen_hint().is_empty());

        assert_eq!(lang.button_definitions().len(), 6);
        for (name, note, idx) in lang.button_definitions() {
            assert!(!name.is_empty());
            assert!(!note.is_empty());
            assert!(idx < 6);
        }

        assert_eq!(lang.button_options().len(), 11);
        for opt in lang.button_options() {
            assert!(!opt.is_empty());
        }

        assert!(!lang.status_recording_live().is_empty());
        assert!(!lang.status_macro_all_flashed_ok().is_empty());
        assert!(!lang.status_mouse_not_connected().is_empty());
        assert!(!lang.status_select_firmware_first().is_empty());
        assert!(!lang.status_firmware_need_usb().is_empty());
        assert!(!lang.firmware_warning().is_empty());

        // Parameterized status messages
        let buf_msg = lang.status_macro_buffer_ok("TestMacro", 64);
        assert!(buf_msg.contains("TestMacro") && buf_msg.contains("64"));

        let flash_msg = lang.status_macro_flashed_ok("TestMacro", 2);
        assert!(flash_msg.contains("TestMacro") && flash_msg.contains("#2"));

        let btn_msg = lang.status_button_bound(1, "Left Click");
        assert!(btn_msg.contains("1") && btn_msg.contains("Left Click"));

        let bat_crit = lang.battery_critical_tag(8);
        assert!(bat_crit.contains("8%"));

        let bat_low = lang.battery_low_tag(15);
        assert!(bat_low.contains("15%"));
    }
}

#[test]
fn test_button_action_display_localized() {
    let action_left = ButtonAction::MouseButton(1);
    assert_eq!(
        action_left.display_localized(AppLanguage::English),
        "Left Click"
    );
    assert_eq!(
        action_left.display_localized(AppLanguage::Indonesian),
        "Klik Kiri"
    );

    let action_right = ButtonAction::MouseButton(2);
    assert_eq!(
        action_right.display_localized(AppLanguage::English),
        "Right Click"
    );
    assert_eq!(
        action_right.display_localized(AppLanguage::Indonesian),
        "Klik Kanan"
    );

    let action_back = ButtonAction::MouseButton(8);
    assert_eq!(action_back.display_localized(AppLanguage::English), "Back");
    assert_eq!(
        action_back.display_localized(AppLanguage::Indonesian),
        "Mundur (Back)"
    );

    let action_fwd = ButtonAction::MouseButton(16);
    assert_eq!(
        action_fwd.display_localized(AppLanguage::English),
        "Forward"
    );
    assert_eq!(
        action_fwd.display_localized(AppLanguage::Indonesian),
        "Maju (Forward)"
    );

    let action_dpi = ButtonAction::DpiSwitch;
    assert_eq!(
        action_dpi.display_localized(AppLanguage::English),
        "DPI Cycle"
    );
    assert_eq!(
        action_dpi.display_localized(AppLanguage::Indonesian),
        "Siklus DPI"
    );

    let action_disabled = ButtonAction::Disabled;
    assert_eq!(
        action_disabled.display_localized(AppLanguage::English),
        "Disabled"
    );
    assert_eq!(
        action_disabled.display_localized(AppLanguage::Indonesian),
        "Nonaktif"
    );

    let action_media = ButtonAction::Media(0xCD);
    assert_eq!(
        action_media.display_localized(AppLanguage::English),
        "Play / Pause"
    );
    assert_eq!(
        action_media.display_localized(AppLanguage::Indonesian),
        "Putar / Jeda"
    );
}

#[test]
fn test_safety_lock_localized_error() {
    let mut cfg = MouseConfig::default();
    // Intentionally remove Left Click from all buttons
    cfg.buttons.fill(ButtonAction::Disabled);

    cfg.language = AppLanguage::English;
    let err_en = cfg.validate_buttons().unwrap_err();
    assert!(err_en.contains("Safety Lock") && err_en.contains("Left Click"));

    cfg.language = AppLanguage::Indonesian;
    let err_id = cfg.validate_buttons().unwrap_err();
    assert!(err_id.contains("Kunci Pengaman") && err_id.contains("Klik Kiri"));
}

#[test]
fn test_mouse_config_language_toml_serde() {
    let mut cfg = MouseConfig::default();
    assert_eq!(cfg.language, AppLanguage::English);

    // Switch to Indonesian and verify TOML serialization contains language field
    cfg.language = AppLanguage::Indonesian;
    let toml_str = toml::to_string_pretty(&cfg).expect("serialize failed");
    assert!(toml_str.contains("language = \"indonesian\""));

    // Deserialize back and verify
    let restored: MouseConfig = toml::from_str(&toml_str).expect("deserialize failed");
    assert_eq!(restored.language, AppLanguage::Indonesian);

    // Legacy profile TOML without language tag must default cleanly to English
    let default_cfg = MouseConfig::default();
    let default_toml = toml::to_string_pretty(&default_cfg).expect("serialize failed");
    let lines: Vec<&str> = default_toml
        .lines()
        .filter(|l| !l.starts_with("language"))
        .collect();
    let legacy_toml = lines.join("\n");
    let legacy_restored: MouseConfig = toml::from_str(&legacy_toml).expect("legacy parse failed");
    assert_eq!(legacy_restored.language, AppLanguage::English);
}
