//! HID protocol implementation for ICARUS V2 MINI.
//!
//! Implements the 17-byte command/response protocol used to communicate
//! with the mouse over the vendor-specific HID interface (Interface 1).

/// Report size excluding the report ID byte.
pub const REPORT_SIZE: usize = 17;

/// Full packet size including report ID.
pub const PACKET_SIZE: usize = REPORT_SIZE + 1;

/// Command opcodes
pub const CMD_WRITE: u8 = 0x07;
pub const CMD_READ: u8 = 0x08;
pub const CMD_RESPONSE: u8 = 0x09;

#[allow(dead_code)]
pub mod addr {
    /// Sensor mode (0x00 = Performance, 0x01 = Office)
    pub const SENSOR_MODE: (u8, u8) = (0x01, 0x00);
    /// Polling rate register
    pub const REPORT_RATE: (u8, u8) = (0x01, 0x01);
    /// Current DPI stage index
    pub const CURRENT_DPI: (u8, u8) = (0x01, 0x02);
    /// DPI LED settings
    pub const DPI_LED: (u8, u8) = (0x01, 0x03);
    /// LED bar colors
    pub const LED_BAR: (u8, u8) = (0x01, 0x04);
    /// Battery level
    pub const BATTERY: (u8, u8) = (0x01, 0x05);
    /// Firmware version
    pub const VERSION: (u8, u8) = (0x01, 0x06);
    /// Debounce time filter
    pub const DEBOUNCE: (u8, u8) = (0x01, 0x07);
    /// Lift-off distance
    pub const LOD: (u8, u8) = (0x01, 0x08);
    /// Angle snapping (straight line correction)
    pub const ANGLE_SNAP: (u8, u8) = (0x01, 0x09);
    /// Ripple control (high DPI smoothing)
    pub const RIPPLE: (u8, u8) = (0x01, 0x0A);
    /// Motion sync
    pub const MOTION_SYNC: (u8, u8) = (0x01, 0x0B);
    /// Sensor idle sleep timer
    pub const SENSOR_SLEEP: (u8, u8) = (0x01, 0x0D);
    /// Full flash data dump
    pub const FLASH_ALL: (u8, u8) = (0x02, 0x00);
}

/// A 17-byte HID command packet.
#[derive(Debug, Clone)]
pub struct Command {
    /// Raw 17-byte payload (excluding report ID)
    pub data: [u8; REPORT_SIZE],
}

impl Command {
    /// Build a read command for a specific register.
    pub fn read(addr_hi: u8, addr_lo: u8, length: u8) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = CMD_READ;
        data[1] = addr_hi;
        data[2] = addr_lo;
        data[3] = length;
        // Bytes 4-15 are zero (padding)
        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Build a write command with payload data.
    pub fn write(addr_hi: u8, addr_lo: u8, payload: &[u8]) -> Self {
        let mut data = [0u8; REPORT_SIZE];
        data[0] = CMD_WRITE;
        data[1] = addr_hi;
        data[2] = addr_lo;
        let copy_len = payload.len().min(12);
        data[3] = copy_len as u8;

        data[4..4 + copy_len].copy_from_slice(&payload[..copy_len]);

        data[16] = Self::checksum(&data[..16]);
        Self { data }
    }

    /// Compute XOR checksum of the first 16 bytes.
    fn checksum(bytes: &[u8]) -> u8 {
        bytes.iter().fold(0u8, |acc, &b| acc ^ b)
    }

    /// Get the full packet with report ID prepended (18 bytes).
    pub fn to_packet(&self) -> [u8; PACKET_SIZE] {
        let mut pkt = [0u8; PACKET_SIZE];
        pkt[0] = 0x07; // Report ID
        pkt[1..].copy_from_slice(&self.data);
        pkt
    }
}

/// A parsed response from the mouse.
#[derive(Debug, Clone)]
pub struct Response {
    /// The raw 17-byte response
    pub data: [u8; REPORT_SIZE],
}

impl Response {
    /// Parse a response from raw bytes (17 bytes, without report ID).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < REPORT_SIZE {
            return Err(ProtocolError::ShortResponse(bytes.len()));
        }

        let mut data = [0u8; REPORT_SIZE];
        data.copy_from_slice(&bytes[..REPORT_SIZE]);

        // Verify checksum
        let expected = data[..16].iter().fold(0u8, |acc, &b| acc ^ b);
        if expected != data[16] {
            return Err(ProtocolError::BadChecksum {
                expected,
                actual: data[16],
            });
        }

        Ok(Self { data })
    }

    /// Parse from a packet (18 bytes with report ID 0x07, or raw 17-byte report).
    pub fn from_packet(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() >= PACKET_SIZE && bytes[0] == 0x07 {
            Self::from_bytes(&bytes[1..])
        } else if bytes.len() >= REPORT_SIZE && bytes[0] == CMD_RESPONSE {
            Self::from_bytes(&bytes[..REPORT_SIZE])
        } else if bytes.len() >= PACKET_SIZE {
            Self::from_bytes(&bytes[1..])
        } else if bytes.len() >= REPORT_SIZE {
            Self::from_bytes(&bytes[..REPORT_SIZE])
        } else {
            Err(ProtocolError::ShortResponse(bytes.len()))
        }
    }

    /// Check if this is a valid response to a read command.
    pub fn is_read_response(&self) -> bool {
        self.data[0] == CMD_RESPONSE
    }

    /// Get the address this response is for.
    #[allow(dead_code)]
    pub fn addr(&self) -> (u8, u8) {
        (self.data[1], self.data[2])
    }

    /// Get the data length field.
    #[allow(dead_code)]
    pub fn data_length(&self) -> u8 {
        self.data[3]
    }

    /// Get the payload bytes (up to 12 bytes at offset 4).
    pub fn payload(&self) -> &[u8] {
        let len = (self.data[3] as usize).min(12);
        &self.data[4..4 + len]
    }
}

/// Protocol errors.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Response too short: {0} bytes")]
    ShortResponse(usize),
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
        Self::read(addr::REPORT_RATE.0, addr::REPORT_RATE.1, 0x01)
    }

    /// Read current DPI stage index.
    pub fn read_current_dpi() -> Self {
        Self::read(addr::CURRENT_DPI.0, addr::CURRENT_DPI.1, 0x01)
    }

    /// Read DPI LED settings.
    pub fn read_dpi_led() -> Self {
        Self::read(addr::DPI_LED.0, addr::DPI_LED.1, 0x06)
    }

    /// Read LED bar colors.
    #[allow(dead_code)]
    pub fn read_led_bar() -> Self {
        Self::read(addr::LED_BAR.0, addr::LED_BAR.1, 0x0C)
    }

    /// Read battery level.
    pub fn read_battery() -> Self {
        Self::read(addr::BATTERY.0, addr::BATTERY.1, 0x01)
    }

    /// Read firmware version.
    pub fn read_version() -> Self {
        Self::read(addr::VERSION.0, addr::VERSION.1, 0x08)
    }

    /// Write polling rate.
    pub fn write_report_rate(code: u8) -> Self {
        Self::write(addr::REPORT_RATE.0, addr::REPORT_RATE.1, &[code])
    }

    /// Write current DPI stage.
    pub fn write_current_dpi(stage: u8) -> Self {
        Self::write(addr::CURRENT_DPI.0, addr::CURRENT_DPI.1, &[stage])
    }

    /// Write DPI LED settings.
    pub fn write_dpi_led(mode: u8, brightness: u8, breath_speed: u8, r: u8, g: u8, b: u8) -> Self {
        Self::write(
            addr::DPI_LED.0,
            addr::DPI_LED.1,
            &[mode, brightness, breath_speed, r, g, b],
        )
    }

    /// Read lift-off distance (LOD).
    pub fn read_lod() -> Self {
        Self::read(addr::LOD.0, addr::LOD.1, 0x01)
    }

    /// Write lift-off distance (LOD).
    pub fn write_lod(code: u8) -> Self {
        Self::write(addr::LOD.0, addr::LOD.1, &[code])
    }

    /// Read debounce time.
    pub fn read_debounce() -> Self {
        Self::read(addr::DEBOUNCE.0, addr::DEBOUNCE.1, 0x01)
    }

    /// Write debounce time filter.
    pub fn write_debounce(ms: u8) -> Self {
        Self::write(addr::DEBOUNCE.0, addr::DEBOUNCE.1, &[ms])
    }

    /// Read angle snapping.
    pub fn read_angle_snap() -> Self {
        Self::read(addr::ANGLE_SNAP.0, addr::ANGLE_SNAP.1, 0x01)
    }

    /// Write angle snapping.
    pub fn write_angle_snap(enabled: bool) -> Self {
        Self::write(
            addr::ANGLE_SNAP.0,
            addr::ANGLE_SNAP.1,
            &[if enabled { 0x01 } else { 0x00 }],
        )
    }

    /// Read motion sync.
    pub fn read_motion_sync() -> Self {
        Self::read(addr::MOTION_SYNC.0, addr::MOTION_SYNC.1, 0x01)
    }

    /// Write motion sync.
    pub fn write_motion_sync(enabled: bool) -> Self {
        Self::write(
            addr::MOTION_SYNC.0,
            addr::MOTION_SYNC.1,
            &[if enabled { 0x01 } else { 0x00 }],
        )
    }

    /// Read ripple control.
    pub fn read_ripple() -> Self {
        Self::read(addr::RIPPLE.0, addr::RIPPLE.1, 0x01)
    }

    /// Write ripple control.
    pub fn write_ripple(enabled: bool) -> Self {
        Self::write(
            addr::RIPPLE.0,
            addr::RIPPLE.1,
            &[if enabled { 0x01 } else { 0x00 }],
        )
    }

    /// Read sensor mode (performance vs office).
    pub fn read_sensor_mode() -> Self {
        Self::read(addr::SENSOR_MODE.0, addr::SENSOR_MODE.1, 0x01)
    }

    /// Write sensor mode (0x00 = Performance, 0x01 = Office).
    pub fn write_sensor_mode(performance: bool) -> Self {
        Self::write(
            addr::SENSOR_MODE.0,
            addr::SENSOR_MODE.1,
            &[if performance { 0x00 } else { 0x01 }],
        )
    }

    /// Read sensor sleep timer.
    pub fn read_sensor_sleep() -> Self {
        Self::read(addr::SENSOR_SLEEP.0, addr::SENSOR_SLEEP.1, 0x01)
    }

    /// Write sensor sleep timer.
    pub fn write_sensor_sleep(seconds: u32) -> Self {
        let code = if seconds == 0 {
            0x00
        } else {
            (seconds / 10).clamp(1, 60) as u8
        };
        Self::write(addr::SENSOR_SLEEP.0, addr::SENSOR_SLEEP.1, &[code])
    }

    /// Write 3-byte button mapping for a specific button index (0..5).
    pub fn write_button(idx: u8, action_bytes: [u8; 3]) -> Self {
        Self::write(0x01, 0x30 + idx * 3, &action_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_command_checksum() {
        let cmd = Command::read_report_rate();
        let expected_xor = cmd.data[..16].iter().fold(0u8, |a, &b| a ^ b);
        assert_eq!(cmd.data[16], expected_xor);
    }

    #[test]
    fn test_read_command_opcode() {
        let cmd = Command::read_report_rate();
        assert_eq!(cmd.data[0], CMD_READ);
        assert_eq!(cmd.data[1], 0x01);
        assert_eq!(cmd.data[2], 0x01);
    }

    #[test]
    fn test_write_command_opcode() {
        let cmd = Command::write_report_rate(0x01);
        assert_eq!(cmd.data[0], CMD_WRITE);
        assert_eq!(cmd.data[4], 0x01); // payload
    }

    #[test]
    fn test_response_parse() {
        let mut raw = [0u8; 17];
        raw[0] = CMD_RESPONSE;
        raw[1] = 0x01;
        raw[2] = 0x01;
        raw[3] = 0x01;
        raw[4] = 0x04; // 250Hz
        raw[16] = raw[..16].iter().fold(0u8, |a, &b| a ^ b);

        let resp = Response::from_bytes(&raw).unwrap();
        assert!(resp.is_read_response());
        assert_eq!(resp.addr(), (0x01, 0x01));
        assert_eq!(resp.payload(), &[0x04]);
    }

    #[test]
    fn test_response_from_packet_with_and_without_prefix() {
        let mut raw17 = [0u8; 17];
        raw17[0] = CMD_RESPONSE;
        raw17[1] = 0x01;
        raw17[2] = 0x08; // LOD
        raw17[3] = 0x01;
        raw17[4] = 0x01; // 2mm
        raw17[16] = raw17[..16].iter().fold(0u8, |a, &b| a ^ b);

        // 17-byte raw packet
        let resp17 = Response::from_packet(&raw17).unwrap();
        assert!(resp17.is_read_response());
        assert_eq!(resp17.payload(), &[0x01]);

        // 18-byte prefixed packet (report ID 0x07)
        let mut raw18 = [0u8; 18];
        raw18[0] = 0x07;
        raw18[1..].copy_from_slice(&raw17);
        let resp18 = Response::from_packet(&raw18).unwrap();
        assert!(resp18.is_read_response());
        assert_eq!(resp18.payload(), &[0x01]);
    }

    #[test]
    fn test_extended_hardware_commands() {
        let lod = Command::write_lod(0x01);
        assert_eq!(lod.data[0], CMD_WRITE);
        assert_eq!((lod.data[1], lod.data[2]), addr::LOD);
        assert_eq!(lod.data[4], 0x01);

        let deb = Command::write_debounce(8);
        assert_eq!((deb.data[1], deb.data[2]), addr::DEBOUNCE);
        assert_eq!(deb.data[4], 8);

        let snap = Command::write_angle_snap(true);
        assert_eq!((snap.data[1], snap.data[2]), addr::ANGLE_SNAP);
        assert_eq!(snap.data[4], 0x01);

        let rip = Command::write_ripple(false);
        assert_eq!((rip.data[1], rip.data[2]), addr::RIPPLE);
        assert_eq!(rip.data[4], 0x00);

        let mode = Command::write_sensor_mode(true);
        assert_eq!((mode.data[1], mode.data[2]), addr::SENSOR_MODE);
        assert_eq!(mode.data[4], 0x00); // 0x00 = Performance

        let btn = Command::write_button(2, [0x01, 0x04, 0x00]);
        assert_eq!(btn.data[1], 0x01);
        assert_eq!(btn.data[2], 0x36); // 0x30 + 2*3 = 0x36
        assert_eq!(&btn.data[4..7], &[0x01, 0x04, 0x00]);
    }
}
