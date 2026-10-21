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
