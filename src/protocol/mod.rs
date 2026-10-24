//! HID protocol implementation for ICARUS V2 MINI.
//!
//! Implements the 17-byte command/response protocol used to communicate
//! with the mouse over the vendor-specific HID interface (Interface 1).

pub mod addr;
pub mod command;
pub mod error;
pub mod macro_codec;
pub mod response;

pub use command::Command;
pub use error::ProtocolError;
pub use macro_codec::{
    MACRO_SLOT_BASE_ADDR, MACRO_SLOT_SIZE, MAX_MACRO_SLOTS, hardware_buffer_to_macro,
    hid_usage_to_key_name, key_to_hid_usage, macro_to_hardware_buffer,
};
pub use response::Response;

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

// ── Extended OEM Opcodes (Decompiled from HIDUsb.dll) ───────────────
pub const CMD_DONGLE_PAIR_CID_MID: u8 = 0x05;
pub const CMD_ENTER_DONGLE_PAIR: u8 = 0x06;
pub const CMD_READ_DONGLE_PAIR_STATUS: u8 = 0x09;
pub const CMD_ENTER_UPDATE_MODE: u8 = 0x0E;
pub const CMD_SET_4K_DONGLE_RGB: u8 = 0x14;
pub const CMD_GET_4K_DONGLE_RGB: u8 = 0x15;
pub const CMD_SET_LONGRANGE_MODE: u8 = 0x16;
pub const CMD_GET_LONGRANGE_MODE: u8 = 0x17;
pub const CMD_SET_DONGLE_RGB_BAR: u8 = 0x18;
pub const CMD_GET_DONGLE_RGB_BAR: u8 = 0x19;
