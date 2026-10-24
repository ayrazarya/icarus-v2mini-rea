//! Inbound HID packet response parser and telemetry inspector.

use super::{
    CMD_QUERY_BATTERY, CMD_QUERY_ONLINE, CMD_READ, CMD_RESPONSE, Command, ProtocolError, REPORT_ID,
    REPORT_SIZE,
};

/// A parsed response from the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Response {
    /// The raw 17-byte response
    pub data: [u8; REPORT_SIZE],
}

impl AsRef<[u8]> for Response {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
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

    /// Parse a response from raw bytes (17 bytes) and verify its packet checksum.
    pub fn from_bytes_verified(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let resp = Self::from_bytes(bytes)?;
        resp.verify_checksum()?;
        Ok(resp)
    }

    /// Verify the 17-byte packet checksum (byte 16 against `0x55 - sum(bytes[0..16])`).
    pub fn verify_checksum(&self) -> Result<(), ProtocolError> {
        let expected = Command::checksum(&self.data[..16]);
        let actual = self.data[16];
        if expected != actual {
            return Err(ProtocolError::BadChecksum { expected, actual });
        }
        Ok(())
    }

    /// Verify that the response opcode matches `expected`.
    pub fn verify_opcode(&self, expected: u8) -> Result<(), ProtocolError> {
        let actual = self.data[1];
        if expected != actual {
            return Err(ProtocolError::UnexpectedOpcode(actual));
        }
        Ok(())
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
    #[must_use]
    pub const fn is_read_response(&self) -> bool {
        self.data[0] == REPORT_ID && (self.data[1] == CMD_READ || self.data[1] == CMD_RESPONSE)
    }

    /// Check if this is a response to a battery query (opcode 0x04).
    #[must_use]
    pub const fn is_battery_response(&self) -> bool {
        self.data[0] == REPORT_ID && self.data[1] == CMD_QUERY_BATTERY
    }

    /// Get battery level percentage (0..100).
    #[must_use]
    pub const fn battery_level(&self) -> u8 {
        if self.data[6] > 0 {
            self.data[6]
        } else {
            self.data[2]
        }
    }

    /// Check if the mouse is currently charging.
    #[must_use]
    pub const fn is_charging(&self) -> bool {
        self.data[7] != 0
    }

    /// Get battery voltage in millivolts.
    #[must_use]
    pub const fn battery_voltage_mv(&self) -> u16 {
        u16::from_be_bytes([self.data[8], self.data[9]])
    }

    /// Check if this is a response to an online query (opcode 0x03).
    #[must_use]
    pub const fn is_online_response(&self) -> bool {
        self.data[0] == REPORT_ID && self.data[1] == CMD_QUERY_ONLINE
    }

    /// Check if mouse device is actively online.
    #[must_use]
    pub const fn is_device_online(&self) -> bool {
        // Opcode 0x03 response:
        // byte 2 = 1 (device online/linked on 2.4G dongle)
        // byte 6 != 0 (device online on direct USB)
        self.data[1] == CMD_QUERY_ONLINE && (self.data[2] != 0 || self.data[6] != 0)
    }

    /// Get the address this response is for.
    #[must_use]
    pub const fn addr(&self) -> (u8, u8) {
        (self.data[3], self.data[4])
    }

    /// Get the data length field.
    #[must_use]
    pub const fn data_length(&self) -> u8 {
        self.data[5]
    }

    /// Get the payload bytes (up to 10 bytes at offset 6).
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        let len = (self.data[5] as usize).min(10);
        &self.data[6..6 + len]
    }
}
