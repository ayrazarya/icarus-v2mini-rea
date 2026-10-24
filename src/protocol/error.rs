//! Protocol errors for HID packet transmission and parsing.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProtocolError {
    #[error("Response too short: {0} bytes")]
    ShortResponse(usize),
    #[error("Checksum mismatch: expected 0x{expected:02X}, got 0x{actual:02X}")]
    BadChecksum { expected: u8, actual: u8 },
    #[error("Unexpected response opcode: 0x{0:02X}")]
    UnexpectedOpcode(u8),
    #[error("Timeout waiting for response")]
    Timeout,
    #[error("Device not found")]
    DeviceNotFound,
    #[error("HID error: {0}")]
    Hid(String),
}
