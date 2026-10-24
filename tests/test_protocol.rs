//! Tests for USB/HID protocol packet structures, checksums, commands, and responses.

use icarus_v2mini::protocol::{
    CMD_QUERY_BATTERY, CMD_QUERY_ONLINE, CMD_READ, CMD_RESPONSE, CMD_WRITE, Command, REPORT_ID,
    Response, addr,
};

#[test]
fn test_read_command_checksum() {
    let cmd = Command::read_report_rate();
    let expected = Command::checksum(&cmd.data[..16]);
    assert_eq!(cmd.data[16], expected);
}

#[test]
fn test_read_command_opcode() {
    let cmd = Command::read_report_rate();
    assert_eq!(cmd.data[0], REPORT_ID);
    assert_eq!(cmd.data[1], CMD_READ);
    assert_eq!(cmd.data[2], 0x00);
    assert_eq!(cmd.data[3], addr::READ_REPORT_RATE.0);
    assert_eq!(cmd.data[4], addr::READ_REPORT_RATE.1);
}

#[test]
fn test_write_command_opcode() {
    let cmd = Command::write_report_rate(0x01);
    assert_eq!(cmd.data[0], REPORT_ID);
    assert_eq!(cmd.data[1], CMD_WRITE);
    assert_eq!(cmd.data[2], 0x00);
    assert_eq!(cmd.data[3], addr::REPORT_RATE.0);
    assert_eq!(cmd.data[4], addr::REPORT_RATE.1);
    assert_eq!(cmd.data[6], 0x01); // payload
    assert_eq!(cmd.data[7], Command::checksum(&[0x01])); // payload checksum
}

#[test]
fn test_response_parse() {
    let mut raw = [0u8; 17];
    raw[0] = REPORT_ID;
    raw[1] = CMD_RESPONSE;
    raw[2] = 0x01;
    raw[3] = 0x01;
    raw[4] = 0x01;
    raw[5] = 0x01;
    raw[6] = 0x04; // 250Hz

    let resp = Response::from_bytes(&raw).unwrap();
    assert!(resp.is_read_response());
    assert_eq!(resp.addr(), (0x01, 0x01));
    assert_eq!(resp.payload(), &[0x04]);
}

#[test]
fn test_response_from_packet_with_and_without_prefix() {
    let mut raw17 = [0u8; 17];
    raw17[0] = REPORT_ID;
    raw17[1] = CMD_READ;
    raw17[2] = 0x01;
    raw17[3] = 0x01;
    raw17[4] = 0x08; // LOD
    raw17[5] = 0x01;
    raw17[6] = 0x01; // 2mm

    // 17-byte raw packet
    let resp17 = Response::from_packet(&raw17).unwrap();
    assert!(resp17.is_read_response());
    assert_eq!(resp17.payload(), &[0x01]);
}

#[test]
fn test_battery_and_online_queries() {
    let bat_cmd = Command::query_battery();
    assert_eq!(bat_cmd.data[0], REPORT_ID);
    assert_eq!(bat_cmd.data[1], CMD_QUERY_BATTERY);

    let mut bat_resp_bytes = [0u8; 17];
    bat_resp_bytes[0] = REPORT_ID;
    bat_resp_bytes[1] = CMD_QUERY_BATTERY;
    bat_resp_bytes[2] = 0x01;
    bat_resp_bytes[6] = 85; // 85%
    bat_resp_bytes[7] = 1; // charging
    bat_resp_bytes[8] = 0x0F;
    bat_resp_bytes[9] = 0x50; // 3920 mV
    let bat_resp = Response::from_bytes(&bat_resp_bytes).unwrap();
    assert!(bat_resp.is_battery_response());
    assert_eq!(bat_resp.battery_level(), 85);
    assert!(bat_resp.is_charging());
    assert_eq!(bat_resp.battery_voltage_mv(), 3920);

    let online_cmd = Command::query_online();
    assert_eq!(online_cmd.data[0], REPORT_ID);
    assert_eq!(online_cmd.data[1], CMD_QUERY_ONLINE);

    let mut online_resp_bytes = [0u8; 17];
    online_resp_bytes[0] = REPORT_ID;
    online_resp_bytes[1] = CMD_QUERY_ONLINE;
    online_resp_bytes[2] = 0x01;
    online_resp_bytes[6] = 1;
    let online_resp = Response::from_bytes(&online_resp_bytes).unwrap();
    assert!(online_resp.is_online_response());
    assert!(online_resp.is_device_online());
}

#[test]
fn test_extended_hardware_commands() {
    let lod = Command::write_lod(0x01);
    assert_eq!(lod.data[0], REPORT_ID);
    assert_eq!(lod.data[1], CMD_WRITE);
    assert_eq!((lod.data[3], lod.data[4]), addr::LOD);
    assert_eq!(lod.data[6], 0x01);

    let deb = Command::write_debounce(8);
    assert_eq!((deb.data[3], deb.data[4]), addr::DEBOUNCE);
    assert_eq!(deb.data[6], 8);

    let snap = Command::write_angle_snap(true);
    assert_eq!((snap.data[3], snap.data[4]), addr::ANGLE_SNAP);
    assert_eq!(snap.data[6], 0x01);

    let rip = Command::write_ripple(false);
    assert_eq!((rip.data[3], rip.data[4]), addr::RIPPLE);
    assert_eq!(rip.data[6], 0x00);

    let mode = Command::write_sensor_mode(true);
    assert_eq!((mode.data[3], mode.data[4]), addr::SENSOR_MODE);
    assert_eq!(mode.data[6], 0x00); // 0x00 = Performance

    let btn = Command::write_button(2, [0x01, 0x04, 0x00]);
    assert_eq!(btn.data[3], 0x00);
    assert_eq!(btn.data[4], 0x60 + 2 * 4); // 0x68
    assert_eq!(&btn.data[6..9], &[0x01, 0x04, 0x00]);
    assert_eq!(btn.data[9], Command::checksum(&[0x01, 0x04, 0x00]));
}

#[test]
fn test_write_dpi_stage_ext_10bit() {
    // Stage 1 at 26,000 DPI (low=0x07, flag=0x80)
    let cmd = Command::write_dpi_stage_ext(1, 0x07, 0x07, 0x80);
    assert_eq!(cmd.data[0], REPORT_ID);
    assert_eq!(cmd.data[1], CMD_WRITE);
    assert_eq!(cmd.data[4], addr::DPI_STAGE_BASE + 4);
    assert_eq!(cmd.data[6], 0x07);
    assert_eq!(cmd.data[7], 0x07);
    assert_eq!(cmd.data[8], 0x80);
    assert_eq!(cmd.data[9], Command::checksum(&[0x07, 0x07, 0x80]));
}

#[test]
fn test_extended_dongle_and_power_commands() {
    use icarus_v2mini::protocol::{
        CMD_DONGLE_PAIR_CID_MID, CMD_ENTER_DONGLE_PAIR, CMD_ENTER_UPDATE_MODE,
        CMD_GET_LONGRANGE_MODE, CMD_READ_DONGLE_PAIR_STATUS, CMD_SET_LONGRANGE_MODE,
    };

    // 1. Dongle Pairing
    let pair_simple = Command::enter_dongle_pair();
    assert_eq!(pair_simple.data[0], REPORT_ID);
    assert_eq!(pair_simple.data[1], CMD_ENTER_DONGLE_PAIR);
    assert_eq!(
        pair_simple.data[16],
        Command::checksum(&pair_simple.data[..16])
    );

    let pair_cid_mid = Command::enter_dongle_pair_with_cid_mid(0x51, 0x01);
    assert_eq!(pair_cid_mid.data[0], REPORT_ID);
    assert_eq!(pair_cid_mid.data[1], CMD_DONGLE_PAIR_CID_MID);
    assert_eq!(pair_cid_mid.data[5], 0x02);
    assert_eq!(pair_cid_mid.data[6], 0x51);
    assert_eq!(pair_cid_mid.data[7], 0x01);
    assert_eq!(
        pair_cid_mid.data[16],
        Command::checksum(&pair_cid_mid.data[..16])
    );

    let pair_status = Command::read_dongle_pair_status();
    assert_eq!(pair_status.data[0], REPORT_ID);
    assert_eq!(pair_status.data[1], CMD_READ_DONGLE_PAIR_STATUS);

    // 2. Long Range Mode (RF power boost)
    let lr_on = Command::set_long_range_mode(true);
    assert_eq!(lr_on.data[0], REPORT_ID);
    assert_eq!(lr_on.data[1], CMD_SET_LONGRANGE_MODE);
    assert_eq!(lr_on.data[5], 0x01);
    assert_eq!(lr_on.data[6], 0x01);

    let lr_off = Command::set_long_range_mode(false);
    assert_eq!(lr_off.data[6], 0x00);

    let lr_read = Command::read_long_range_mode();
    assert_eq!(lr_read.data[1], CMD_GET_LONGRANGE_MODE);

    // 3. Firmware Update ISP Mode
    let dfu = Command::enter_update_mode();
    assert_eq!(dfu.data[0], REPORT_ID);
    assert_eq!(dfu.data[1], CMD_ENTER_UPDATE_MODE);
    assert_eq!(dfu.data[16], Command::checksum(&dfu.data[..16]));
}

#[test]
fn test_hardware_macro_buffer_serialization_roundtrip() {
    use icarus_v2mini::config::{MacroEvent, MacroEventType};
    use icarus_v2mini::protocol::{hardware_buffer_to_macro, macro_to_hardware_buffer};

    let events = vec![
        MacroEvent {
            event_type: MacroEventType::KeyDown,
            key: "Key_0x04".into(),
            delay_ms: 0,
        },
        MacroEvent {
            event_type: MacroEventType::Delay,
            key: String::new(),
            delay_ms: 50,
        },
        MacroEvent {
            event_type: MacroEventType::KeyUp,
            key: "Key_0x04".into(),
            delay_ms: 0,
        },
    ];

    let buf = macro_to_hardware_buffer("Burst_Fire", &events);
    assert!(buf.len() >= 32);
    assert_eq!(buf[0], 10); // length of "Burst_Fire"
    assert_eq!(&buf[1..11], b"Burst_Fire");
    assert_eq!(buf[31], 3); // 3 events

    let parsed = hardware_buffer_to_macro(&buf).expect("Parse hardware macro buffer");
    assert_eq!(parsed.name, "Burst_Fire");
    assert_eq!(parsed.events.len(), 3);
    assert_eq!(parsed.events[0].event_type, MacroEventType::KeyDown);
    assert_eq!(parsed.events[1].event_type, MacroEventType::Delay);
    assert_eq!(parsed.events[1].delay_ms, 50);
    assert_eq!(parsed.events[2].event_type, MacroEventType::KeyUp);
}

#[test]
fn test_hid_usage_key_mapping() {
    use icarus_v2mini::protocol::{hid_usage_to_key_name, key_to_hid_usage};

    // Standard alphanumeric
    assert_eq!(key_to_hid_usage("A"), 0x04);
    assert_eq!(key_to_hid_usage("Z"), 0x1D);
    assert_eq!(key_to_hid_usage("1"), 0x1E);
    assert_eq!(key_to_hid_usage("0"), 0x27);

    // Controls and navigation
    assert_eq!(key_to_hid_usage("Space"), 0x2C);
    assert_eq!(key_to_hid_usage("Enter"), 0x28);
    assert_eq!(key_to_hid_usage("Escape"), 0x29);
    assert_eq!(key_to_hid_usage("Tab"), 0x2B);
    assert_eq!(key_to_hid_usage("F1"), 0x3A);
    assert_eq!(key_to_hid_usage("F12"), 0x45);

    // Mouse buttons inside macro sequences
    assert_eq!(key_to_hid_usage("Mouse Left"), 0xF0);
    assert_eq!(key_to_hid_usage("Mouse Right"), 0xF1);
    assert_eq!(key_to_hid_usage("Mouse Middle"), 0xF2);
    assert_eq!(key_to_hid_usage("Mouse Forward"), 0xF3);
    assert_eq!(key_to_hid_usage("Mouse Back"), 0xF4);

    // Reverse lookup
    assert_eq!(hid_usage_to_key_name(0x04), "A");
    assert_eq!(hid_usage_to_key_name(0x2C), "Space");
    assert_eq!(hid_usage_to_key_name(0xF0), "Mouse Left");
    assert_eq!(hid_usage_to_key_name(0xF4), "Mouse Back");
}

#[test]
fn test_macro_hardware_buffer_with_mouse_clicks() {
    use icarus_v2mini::config::{MacroEvent, MacroEventType};
    use icarus_v2mini::protocol::{hardware_buffer_to_macro, macro_to_hardware_buffer};

    let events = vec![
        MacroEvent {
            event_type: MacroEventType::KeyDown,
            key: "Mouse Left".into(),
            delay_ms: 0,
        },
        MacroEvent {
            event_type: MacroEventType::Delay,
            key: String::new(),
            delay_ms: 12,
        },
        MacroEvent {
            event_type: MacroEventType::KeyUp,
            key: "Mouse Left".into(),
            delay_ms: 0,
        },
    ];

    let buf = macro_to_hardware_buffer("Clicker", &events);
    assert_eq!(buf[0], 7);
    assert_eq!(&buf[1..8], b"Clicker");
    assert_eq!(buf[31], 3);

    let parsed = hardware_buffer_to_macro(&buf).expect("Parse hardware buffer with mouse clicks");
    assert_eq!(parsed.name, "Clicker");
    assert_eq!(parsed.events.len(), 3);
    assert_eq!(parsed.events[0].key, "Mouse Left");
    assert_eq!(parsed.events[2].key, "Mouse Left");
}

#[test]
fn test_macro_hardware_buffer_utf8_name_truncation() {
    use icarus_v2mini::protocol::{hardware_buffer_to_macro, macro_to_hardware_buffer};

    // A string with multi-byte unicode characters that exceeds 30 bytes
    // "Mäcrö-Gämïng-Prö-Tëstïng-12345" has many 2-byte characters
    let long_utf8_name = "Mäcrö-Gämïng-Prö-Tëstïng-12345";
    assert!(long_utf8_name.len() > 30);

    let buf = macro_to_hardware_buffer(long_utf8_name, &[]);
    assert!(buf[0] <= 30);
    // Ensure the slice is valid UTF-8
    let name_str = std::str::from_utf8(&buf[1..1 + (buf[0] as usize)])
        .expect("Buffer name must be valid UTF-8");
    assert!(!name_str.is_empty());

    let parsed = hardware_buffer_to_macro(&buf).expect("Parsed successfully");
    assert_eq!(parsed.name, name_str);
}

#[test]
fn test_response_checksum_and_opcode_verification() {
    use icarus_v2mini::protocol::ProtocolError;

    let cmd = Command::read_report_rate();
    let resp = Response::from_bytes_verified(&cmd.data).expect("Valid checksum packet");
    assert!(resp.verify_checksum().is_ok());
    assert!(resp.verify_opcode(CMD_READ).is_ok());
    assert_eq!(
        resp.verify_opcode(CMD_WRITE),
        Err(ProtocolError::UnexpectedOpcode(CMD_READ))
    );

    let mut corrupted = cmd.data;
    corrupted[16] ^= 0xFF;
    assert!(matches!(
        Response::from_bytes_verified(&corrupted),
        Err(ProtocolError::BadChecksum { .. })
    ));
}

#[test]
fn test_hid_usage_case_insensitive_and_full_coverage() {
    use icarus_v2mini::protocol::{hid_usage_to_key_name, key_to_hid_usage};

    // Lowercase and mixed-case keys must map accurately, not fall back to 'A' (0x04)
    assert_eq!(key_to_hid_usage("a"), 0x04);
    assert_eq!(key_to_hid_usage("e"), 0x08);
    assert_eq!(key_to_hid_usage("z"), 0x1D);
    assert_eq!(key_to_hid_usage("space"), 0x2C);
    assert_eq!(key_to_hid_usage("enter"), 0x28);
    assert_eq!(key_to_hid_usage("esc"), 0x29);
    assert_eq!(key_to_hid_usage("ctrl"), 0xE0);
    assert_eq!(key_to_hid_usage("f12"), 0x45);
    assert_eq!(key_to_hid_usage("mouse right"), 0xF1);
    assert_eq!(key_to_hid_usage("key_0x15"), 0x15);

    // egui::Key Debug format aliases
    assert_eq!(key_to_hid_usage("Backtick"), 0x35);
    assert_eq!(key_to_hid_usage("Plus"), 0x2E);
    assert_eq!(key_to_hid_usage("Pipe"), 0x31);
    assert_eq!(key_to_hid_usage("Colon"), 0x33);
    assert_eq!(key_to_hid_usage("Questionmark"), 0x38);

    // Invalid / unmapped key names must return 0x00, not silently map to 'A' (0x04)
    assert_eq!(key_to_hid_usage("invalid_nonexistent_key"), 0x00);
    assert_eq!(key_to_hid_usage(""), 0x00);

    // Reverse lookup for extended navigation and modifier keys
    assert_eq!(hid_usage_to_key_name(0x39), "CapsLock");
    assert_eq!(hid_usage_to_key_name(0x46), "PrintScreen");
    assert_eq!(hid_usage_to_key_name(0x4C), "Delete");
    assert_eq!(hid_usage_to_key_name(0xE4), "RightCtrl");
    assert_eq!(hid_usage_to_key_name(0xE6), "RightAlt");
}

#[test]
fn test_macro_buffer_corruption_and_bounds_rejection() {
    use icarus_v2mini::config::{MacroEvent, MacroEventType};
    use icarus_v2mini::protocol::{hardware_buffer_to_macro, macro_to_hardware_buffer};

    let events = vec![MacroEvent {
        event_type: MacroEventType::KeyDown,
        key: "space".into(),
        delay_ms: 10,
    }];
    let mut buf = macro_to_hardware_buffer("Jump", &events);
    assert!(hardware_buffer_to_macro(&buf).is_some());

    // Corrupt checksum byte
    let last_idx = buf.len() - 1;
    buf[last_idx] ^= 0xFF;
    assert!(hardware_buffer_to_macro(&buf).is_none());

    // Fix checksum, then set event_count > 64
    buf[last_idx] ^= 0xFF;
    buf[31] = 65;
    assert!(hardware_buffer_to_macro(&buf).is_none());
}
