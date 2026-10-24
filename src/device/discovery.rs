//! Enumeration and discovery of connected ICARUS V2 MINI devices via HIDAPI.

use super::{CONFIG_INTERFACE, PID_USB, PID_WIRELESS, PID_WIRELESS_4G, VID};
use anyhow::{Context, Result};

/// Enumerates all connected ICARUS V2 MINI devices across USB and 2.4G/4K wireless interfaces.
///
/// # Errors
/// Returns an error if initializing the system HIDAPI subsystem fails.
pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    let api = hidapi::HidApi::new().context("Failed to initialize HIDAPI")?;
    let mut found = Vec::new();

    for dev in api.device_list() {
        if dev.vendor_id() != VID || dev.interface_number() != CONFIG_INTERFACE {
            continue;
        }
        let pid = dev.product_id();
        let conn = match pid {
            PID_USB => "USB",
            PID_WIRELESS => "2.4G Wireless",
            PID_WIRELESS_4G => "4G Wireless",
            _ => continue,
        };
        found.push(DeviceInfo {
            vendor_id: VID,
            product_id: pid,
            connection: conn.to_string(),
            path: dev.path().to_string_lossy().into_owned(),
            manufacturer: dev
                .manufacturer_string()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            product: dev
                .product_string()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        });
    }

    found.sort_by_key(|d| match d.product_id {
        PID_USB => 0u8,
        PID_WIRELESS => 1u8,
        PID_WIRELESS_4G => 2u8,
        _ => 3u8,
    });

    Ok(found)
}

/// Information about a discovered device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub connection: String,
    pub path: String,
    pub manufacturer: String,
    pub product: String,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({}) [{:04X}:{:04X}]",
            self.manufacturer, self.product, self.connection, self.vendor_id, self.product_id
        )
    }
}
