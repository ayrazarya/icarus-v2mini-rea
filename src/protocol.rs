//! HID protocol implementation for ICARUS V2 MINI.
//!
//! Implements the 17-byte command/response protocol used to communicate
//! with the mouse over the vendor-specific HID interface (Interface 1).

/// Report size written to and read from the vendor HID interface (Interface 1).
pub const REPORT_SIZE: usize = 17;

/// Full packet size including Report ID (exact 17-byte report).
pub const PACKET_SIZE: usize = 17;

/// Vendor HID Report ID for Output and Input reports.
pub const REPORT_ID: u8 = 0x08;

/// Command opcodes
pub const CMD_WRITE: u8 = 0x07;
pub const CMD_READ: u8 = 0x08;
pub const CMD_RESPONSE: u8 = 0x09;
pub const CMD_QUERY_ONLINE: u8 = 0x03;
pub const CMD_QUERY_BATTERY: u8 = 0x04;
pub const CMD_PC_DRIVER_STATUS: u8 = 0x02;
pub const CMD_APPLY_CONFIG: u8 = 0x0F;

#[allow(dead_code)]
pub mod addr {
    // ── Flash Data Map Registers (Decompiled from HIDUsb.dll & functionset.obj) ─
    // addr_hi is always 0x00 for all flash map registers
    pub const REPORT_RATE: (u8, u8) = (0x00, 0x00);
    pub const MAX_DPI_STAGES: (u8, u8) = (0x00, 0x02);
    pub const CURRENT_DPI: (u8, u8) = (0x00, 0x04);
    pub const ANGLE_SNAP_X: (u8, u8) = (0x00, 0x06);
    pub const ANGLE_SNAP_Y: (u8, u8) = (0x00, 0x08);
    pub const LOD: (u8, u8) = (0x00, 0x0A);
    pub const DPI_STAGE_BASE: u8 = 0x0C; // 0x0C + stage * 4, length 4: [x_reg, y_reg, 0x00, chk]
    pub const DPI_COLOR_BASE: u8 = 0x2C; // 0x2C + stage * 4, length 4: [r, g, b, chk]
    pub const BUTTON_BASE: u8 = 0x60;    // 0x60 + idx * 4, length 4: [b0, b1, b2, chk]
    pub const DPI_LED_COMPOSITE: (u8, u8) = (0x00, 0xA0); // length 7: [mode, bright, speed, r, g, b, chk]
    pub const DEBOUNCE: (u8, u8) = (0x00, 0xA9);
    pub const MOTION_SYNC: (u8, u8) = (0x00, 0xAB);
    pub const RIPPLE: (u8, u8) = (0x00, 0xB1);
    pub const SENSOR_SLEEP_SWITCH: (u8, u8) = (0x00, 0xB5);
    pub const SENSOR_SLEEP: (u8, u8) = (0x00, 0xB7);
    pub const SENSOR_MODE: (u8, u8) = (0x00, 0xB9);

    // Read address aliases mapped to exact flash registers
    pub const READ_REPORT_RATE: (u8, u8) = REPORT_RATE;
    pub const READ_MAX_DPI_STAGES: (u8, u8) = MAX_DPI_STAGES;
    pub const READ_CURRENT_DPI: (u8, u8) = CURRENT_DPI;
    pub const READ_LOD: (u8, u8) = LOD;
    pub const READ_DEBOUNCE: (u8, u8) = DEBOUNCE;
    pub const READ_MOTION_SYNC: (u8, u8) = MOTION_SYNC;
    pub const READ_RIPPLE: (u8, u8) = RIPPLE;
    pub const READ_ANGLE_SNAP: (u8, u8) = ANGLE_SNAP_X;
    pub const ANGLE_SNAP: (u8, u8) = ANGLE_SNAP_X;
    pub const READ_SENSOR_SLEEP: (u8, u8) = SENSOR_SLEEP;
    pub const READ_SENSOR_MODE: (u8, u8) = SENSOR_MODE;
    pub const READ_VERSION: (u8, u8) = (0x00, 0x06);
    pub const READ_DPI_LED: (u8, u8) = DPI_LED_COMPOSITE;
    pub const READ_BATTERY: (u8, u8) = (0x00, 0x05);

    // Legacy aliases
    pub const DPI_LED: (u8, u8) = READ_DPI_LED;
    pub const BATTERY: (u8, u8) = READ_BATTERY;
    pub const VERSION: (u8, u8) = READ_VERSION;
    pub const FLASH_ALL: (u8, u8) = (0x00, 0x00);
}

/// A 17-byte HID command packet (Report ID 0x08 + 16 payload/checksum bytes).
#[derive(Debug, Clone)]
pub struct Command {
    /// Raw 17-byte payload
    pub data: [u8; REPORT_SIZE],
}

impl Command {
    /// Build a read command for a specific register.
    pub fn read(addr_hi: u8, addr_lo: u8, length: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_READ;  // 0x08
        data[2] = 0x00;      // flags
        data[3] = addr_hi;
        data[4] = addr_lo;
        data[5] = length;
        // Bytes 6-15 are zero (padding)
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Build a write command with payload data.
    #[allow(dead_code)]
    pub fn write(addr_hi: u8, addr_lo: u8, payload: &[u8]) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;  // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00;      // flags
        data[3] = addr_hi;
        data[4] = addr_lo;
        let copy_len = payload.len().min(10);
        data[5] = copy_len as u8;

        data[6..6 + copy_len].copy_from_slice(&payload[..copy_len]);

        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Build a write command for a single 1-byte register.
    pub fn write_byte(addr_hi: u8, addr_lo: u8, val: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00;      // flags
        data[3] = addr_hi;
        data[4] = addr_lo;
        data[5] = 0x02;      // length = 2 (val + payload checksum)
        data[6] = val;
        data[7] = Self::checksum(&[val]); // sub_1000aaf0: 0x55 - val
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Query battery status (opcode 0x04).
    pub fn query_battery() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_QUERY_BATTERY; // 0x04
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Query online status (opcode 0x03).
    pub fn query_online() -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_QUERY_ONLINE; // 0x03
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Build a PC Driver Status command (opcode 0x02) to notify MCU the driver is connected.
    pub fn pc_driver_status(active: bool) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_PC_DRIVER_STATUS; // 0x02
        data[5] = 0x01; // length = 1
        data[6] = if active { 0x01 } else { 0x00 };
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Compute 8-bit checksum: 0x55 - wrapping_sum(bytes).
    /// Decompiled from 0x1000aaf0 in HIDUsb.dll / Compx mouse firmware.
    pub fn checksum(bytes: &[u8]) -> u8 {
        let sum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        0x55u8.wrapping_sub(sum)
    }

    /// Get the full 17-byte packet ready for HID transmission.
    pub fn to_packet(&self) -> [u8; PACKET_SIZE] {
        self.data
    }
}

/// A parsed response from the mouse.
#[derive(Debug, Clone)]
pub struct Response {
    /// The raw 17-byte response
    pub data: [u8; REPORT_SIZE],
}

impl Response {
    /// Parse a response from raw bytes (17 bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < REPORT_SIZE {
            return Err(ProtocolError::ShortResponse(bytes.len()));
        }

        let mut data = [0u8; REPORT_SIZE];
        data.copy_from_slice(&bytes[..REPORT_SIZE]);

        Ok(Self { data })
    }

    /// Parse from a packet (17 bytes or prefixed).
    pub fn from_packet(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() >= REPORT_SIZE {
            Self::from_bytes(&bytes[..REPORT_SIZE])
        } else {
            Err(ProtocolError::ShortResponse(bytes.len()))
        }
    }

    /// Check if this is a valid response to a read command.
    #[allow(dead_code)]
    pub fn is_read_response(&self) -> bool {
        self.data[0] == REPORT_ID && (self.data[1] == CMD_READ || self.data[1] == CMD_RESPONSE)
    }

    /// Check if this is a response to a battery query (opcode 0x04).
    #[allow(dead_code)]
    pub fn is_battery_response(&self) -> bool {
        self.data[0] == REPORT_ID && self.data[1] == CMD_QUERY_BATTERY
    }

    /// Get battery level percentage (0..100).
    pub fn battery_level(&self) -> u8 {
        if self.data[6] > 0 {
            self.data[6]
        } else {
            self.data[2]
        }
    }

    /// Check if the mouse is currently charging.
    #[allow(dead_code)]
    pub fn is_charging(&self) -> bool {
        self.data[7] != 0
    }

    /// Get battery voltage in millivolts.
    #[allow(dead_code)]
    pub fn battery_voltage_mv(&self) -> u16 {
        u16::from_be_bytes([self.data[8], self.data[9]])
    }

    /// Check if this is a response to an online query (opcode 0x03).
    #[allow(dead_code)]
    pub fn is_online_response(&self) -> bool {
        self.data[0] == REPORT_ID && self.data[1] == CMD_QUERY_ONLINE
    }

    /// Check if mouse device is actively online.
    pub fn is_device_online(&self) -> bool {
        // Opcode 0x03 response:
        // byte 2 = 1 (device online/linked on 2.4G dongle)
        // byte 6 != 0 (device online on direct USB)
        self.data[1] == CMD_QUERY_ONLINE && (self.data[2] != 0 || self.data[6] != 0)
    }

    /// Get the address this response is for.
    #[allow(dead_code)]
    pub fn addr(&self) -> (u8, u8) {
        (self.data[3], self.data[4])
    }

    /// Get the data length field.
    #[allow(dead_code)]
    pub fn data_length(&self) -> u8 {
        self.data[5]
    }

    /// Get the payload bytes (up to 10 bytes at offset 6).
    pub fn payload(&self) -> &[u8] {
        let len = (self.data[5] as usize).min(10);
        &self.data[6..6 + len]
    }
}

/// Protocol errors.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Response too short: {0} bytes")]
    ShortResponse(usize),
    #[allow(dead_code)]
    #[error("Checksum mismatch: expected 0x{expected:02X}, got 0x{actual:02X}")]
    BadChecksum { expected: u8, actual: u8 },
    #[allow(dead_code)]
    #[error("Unexpected response opcode: 0x{0:02X}")]
    UnexpectedOpcode(u8),
    #[error("Timeout waiting for response")]
    Timeout,
    #[allow(dead_code)]
    #[error("Device not found")]
    DeviceNotFound,
    #[error("HID error: {0}")]
    Hid(String),
}

// Build read commands for common registers
impl Command {
    /// Read current polling rate.
    pub fn read_report_rate() -> Self {
        Self::read(addr::READ_REPORT_RATE.0, addr::READ_REPORT_RATE.1, 0x02)
    }

    /// Read max DPI stages count.
    #[allow(dead_code)]
    pub fn read_max_dpi_stages() -> Self {
        Self::read(addr::READ_MAX_DPI_STAGES.0, addr::READ_MAX_DPI_STAGES.1, 0x02)
    }

    /// Read current DPI stage index.
    pub fn read_current_dpi() -> Self {
        Self::read(addr::READ_CURRENT_DPI.0, addr::READ_CURRENT_DPI.1, 0x02)
    }

    /// Read DPI LED settings.
    pub fn read_dpi_led() -> Self {
        Self::read(addr::DPI_LED.0, addr::DPI_LED.1, 0x07)
    }

    /// Read LED bar colors.
    #[allow(dead_code)]
    pub fn read_led_bar() -> Self {
        Self::read(0x00, 0xA0, 0x07)
    }

    /// Legacy read battery level.
    #[allow(dead_code)]
    pub fn read_battery() -> Self {
        Self::read(addr::BATTERY.0, addr::BATTERY.1, 0x01)
    }

    /// Read firmware version.
    pub fn read_version() -> Self {
        Self::read(addr::VERSION.0, addr::VERSION.1, 0x08)
    }

    /// Write polling rate.
    pub fn write_report_rate(code: u8) -> Self {
        Self::write_byte(addr::REPORT_RATE.0, addr::REPORT_RATE.1, code)
    }

    /// Write max DPI stages.
    pub fn write_max_dpi_stages(count: u8) -> Self {
        Self::write_byte(addr::MAX_DPI_STAGES.0, addr::MAX_DPI_STAGES.1, count)
    }

    /// Write current DPI stage.
    pub fn write_current_dpi(stage: u8) -> Self {
        Self::write_byte(addr::CURRENT_DPI.0, addr::CURRENT_DPI.1, stage)
    }

    /// Write composite DPI LED settings to hardware register (0x00, 0xA0).
    pub fn write_dpi_led_composite(
        mode: u8,
        brightness: u8,
        breath_speed: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00;
        data[3] = addr::DPI_LED_COMPOSITE.0; // 0x00
        data[4] = addr::DPI_LED_COMPOSITE.1; // 0xA0
        data[5] = 0x07;                      // length = 7
        data[6] = mode;
        data[7] = brightness;
        data[8] = breath_speed;
        data[9] = r;
        data[10] = g;
        data[11] = b;
        data[12] = Self::checksum(&data[6..12]);
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Write DPI LED settings as composite packet.
    pub fn write_dpi_led_commands(
        mode: u8,
        brightness: u8,
        breath_speed: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Vec<Self> {
        vec![
            Self::write_dpi_led_composite(mode, brightness, breath_speed, r, g, b),
        ]
    }

    /// Single write_dpi_led packet using composite hardware register (0x00, 0xA0).
    #[allow(dead_code)]
    pub fn write_dpi_led(mode: u8, brightness: u8, breath_speed: u8, r: u8, g: u8, b: u8) -> Self {
        Self::write_dpi_led_composite(mode, brightness, breath_speed, r, g, b)
    }

    /// Read lift-off distance (LOD).
    pub fn read_lod() -> Self {
        Self::read(addr::READ_LOD.0, addr::READ_LOD.1, 0x02)
    }

    /// Write lift-off distance (LOD).
    pub fn write_lod(code: u8) -> Self {
        Self::write_byte(addr::LOD.0, addr::LOD.1, code)
    }

    /// Read debounce time.
    pub fn read_debounce() -> Self {
        Self::read(addr::READ_DEBOUNCE.0, addr::READ_DEBOUNCE.1, 0x02)
    }

    /// Write debounce time filter.
    pub fn write_debounce(ms: u8) -> Self {
        Self::write_byte(addr::DEBOUNCE.0, addr::DEBOUNCE.1, ms)
    }

    /// Read angle snapping.
    pub fn read_angle_snap() -> Self {
        Self::read(addr::READ_ANGLE_SNAP.0, addr::READ_ANGLE_SNAP.1, 0x02)
    }

    /// Write angle snapping.
    pub fn write_angle_snap(enabled: bool) -> Self {
        Self::write_byte(
            addr::ANGLE_SNAP.0,
            addr::ANGLE_SNAP.1,
            if enabled { 0x01 } else { 0x00 },
        )
    }

    /// Read motion sync.
    pub fn read_motion_sync() -> Self {
        Self::read(addr::READ_MOTION_SYNC.0, addr::READ_MOTION_SYNC.1, 0x02)
    }

    /// Write motion sync.
    pub fn write_motion_sync(enabled: bool) -> Self {
        Self::write_byte(
            addr::MOTION_SYNC.0,
            addr::MOTION_SYNC.1,
            if enabled { 0x01 } else { 0x00 },
        )
    }

    /// Read ripple control.
    pub fn read_ripple() -> Self {
        Self::read(addr::READ_RIPPLE.0, addr::READ_RIPPLE.1, 0x02)
    }

    /// Write ripple control.
    pub fn write_ripple(enabled: bool) -> Self {
        Self::write_byte(
            addr::RIPPLE.0,
            addr::RIPPLE.1,
            if enabled { 0x01 } else { 0x00 },
        )
    }

    /// Read sensor mode (performance vs office).
    pub fn read_sensor_mode() -> Self {
        Self::read(addr::READ_SENSOR_MODE.0, addr::READ_SENSOR_MODE.1, 0x02)
    }

    /// Write sensor mode (0x00 = Performance, 0x01 = Office).
    pub fn write_sensor_mode(performance: bool) -> Self {
        Self::write_byte(
            addr::SENSOR_MODE.0,
            addr::SENSOR_MODE.1,
            if performance { 0x00 } else { 0x01 },
        )
    }

    /// Read sensor sleep timer.
    pub fn read_sensor_sleep() -> Self {
        Self::read(addr::READ_SENSOR_SLEEP.0, addr::READ_SENSOR_SLEEP.1, 0x02)
    }

    /// Write sensor sleep timer.
    pub fn write_sensor_sleep(seconds: u32) -> Self {
        let code = if seconds == 0 {
            0x00
        } else {
            (seconds / 10).clamp(1, 60) as u8
        };
        Self::write_byte(addr::SENSOR_SLEEP.0, addr::SENSOR_SLEEP.1, code)
    }

    /// Write 3-byte button mapping for a specific button index (0..5).
    pub fn write_button(idx: u8, action_bytes: [u8; 3]) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_WRITE;
        data[2] = 0x00;
        data[3] = 0x00; // addr_hi = 0x00
        data[4] = addr::BUTTON_BASE + idx * 4;
        data[5] = 0x04;
        data[6] = action_bytes[0];
        data[7] = action_bytes[1];
        data[8] = action_bytes[2];
        data[9] = Self::checksum(&action_bytes);
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Write DPI stage value to hardware register (0x00, 0x0C + stage * 4).
    pub fn write_dpi_stage(stage: u8, x_reg: u8, y_reg: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID; // 0x08
        data[1] = CMD_WRITE; // 0x07
        data[2] = 0x00;
        data[3] = 0x00;      // addr_hi = 0x00
        data[4] = addr::DPI_STAGE_BASE + stage * 4;
        data[5] = 0x04;      // length = 4
        data[6] = x_reg;
        data[7] = y_reg;
        data[8] = 0x00;
        data[9] = Self::checksum(&[x_reg, y_reg, 0x00]);
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Write DPI stage LED color to hardware register (0x00, 0x2C + stage * 4).
    pub fn write_dpi_stage_color(stage: u8, r: u8, g: u8, b: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_WRITE;
        data[2] = 0x00;
        data[3] = 0x00; // addr_hi = 0x00
        data[4] = addr::DPI_COLOR_BASE + stage * 4;
        data[5] = 0x04;
        data[6] = r;
        data[7] = g;
        data[8] = b;
        data[9] = Self::checksum(&[r, g, b]);
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Apply and commit configuration profile (opcode 0x0F, profile index e.g. 1).
    pub fn apply_config(profile: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = REPORT_ID;
        data[1] = CMD_APPLY_CONFIG;
        data[5] = 0x01;
        data[6] = profile;
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        bat_resp_bytes[7] = 1;  // charging
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
}
