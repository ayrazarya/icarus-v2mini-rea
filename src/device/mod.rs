//! Device discovery and HID communication for ICARUS V2 MINI.
//!
//! Uses `hidapi` to find the mouse on Interface 1 (vendor-specific HID)
//! and send/receive 17-byte command packets.

pub mod composite;
pub mod connection;
pub mod discovery;
pub mod firmware;
pub mod macros;
pub mod registers;

pub use discovery::{DeviceInfo, list_devices};

/// USB Vendor ID for ICARUS V2 MINI
pub const VID: u16 = 0x3554;

/// USB Product IDs
pub const PID_USB: u16 = 0xF511;
pub const PID_WIRELESS: u16 = 0xF512;
pub const PID_WIRELESS_4G: u16 = 0xF510;

/// The vendor-specific HID interface index used for configuration
pub const CONFIG_INTERFACE: i32 = 1;

/// Read timeout in milliseconds
pub const READ_TIMEOUT_MS: i32 = 500;

/// Comprehensive battery and power reporting status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BatteryStatus {
    /// Battery level percentage (0–100)
    pub level: u8,
    /// Whether the mouse is currently charging or powered via USB
    pub is_charging: bool,
    /// Battery voltage in millivolts
    pub voltage_mv: u16,
}

impl std::fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}% ({} mV){}",
            self.level,
            self.voltage_mv,
            if self.is_charging { " [charging]" } else { "" }
        )
    }
}

/// Represents an open connection to an ICARUS V2 MINI mouse.
pub struct IcarusDevice {
    pub(crate) device: hidapi::HidDevice,
    /// Which PID was matched
    pub product_id: u16,

    /// Connection type description
    pub connection: String,
}

impl std::fmt::Debug for IcarusDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcarusDevice")
            .field("product_id", &format_args!("0x{:04X}", self.product_id))
            .field("connection", &self.connection)
            .finish()
    }
}
