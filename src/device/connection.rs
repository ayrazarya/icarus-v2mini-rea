//! Core HID transport connection, command transmission, and pacing.

use super::{
    CONFIG_INTERFACE, IcarusDevice, PID_USB, PID_WIRELESS, PID_WIRELESS_4G, READ_TIMEOUT_MS, VID,
};
use crate::protocol::{
    CMD_READ, CMD_RESPONSE, CMD_WRITE, Command, ProtocolError, REPORT_ID, Response,
};
use anyhow::{Context, Result, bail};
use std::time::Duration;

impl IcarusDevice {
    /// Finds and opens an active HID connection to an ICARUS V2 MINI mouse.
    ///
    /// Probes connected devices in priority order: USB wired (`0xF511`), 2.4G wireless (`0xF512`),
    /// and 4K wireless dongle (`0xF510`) on configuration interface 1.
    ///
    /// # Errors
    /// Returns an error if HIDAPI initialization fails or no supported mouse is found.
    pub fn open() -> Result<Self> {
        let api = hidapi::HidApi::new().context("Failed to initialize HIDAPI")?;

        // Try each PID in order of preference
        let candidates = [
            (PID_USB, "USB"),
            (PID_WIRELESS, "2.4G Wireless"),
            (PID_WIRELESS_4G, "4G Wireless"),
        ];

        for &(pid, connection_name) in &candidates {
            // Find the vendor-specific configuration interface (interface 1) without intermediate allocation
            for dev_info in api.device_list().filter(|d| {
                d.vendor_id() == VID
                    && d.product_id() == pid
                    && d.interface_number() == CONFIG_INTERFACE
            }) {
                match dev_info.open_device(&api) {
                    Ok(device) => {
                        // Set blocking mode for reads with explicit timeout
                        device
                            .set_blocking_mode(true)
                            .context("Failed to set blocking mode")?;

                        let dev = IcarusDevice {
                            device,
                            product_id: pid,
                            connection: connection_name.to_string(),
                        };
                        let _ = dev.send_pc_driver_status(true);
                        return Ok(dev);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Found device on {} but failed to open: {}",
                            connection_name, e
                        );
                        eprintln!("Hint: You may need to set up a udev rule. See README.md.");
                    }
                }
            }
        }

        bail!(
            "ICARUS V2 MINI not found.\n\
             Make sure the mouse is connected and you have permission to access HID devices.\n\
             \n\
             Searched for:\n\
             - USB:          VID=0x{:04X} PID=0x{:04X}\n\
             - 2.4G Wireless: VID=0x{:04X} PID=0x{:04X}\n\
             - 4G Wireless:  VID=0x{:04X} PID=0x{:04X}\n\
             \n\
             To grant access without root, create a udev rule:\n\
             sudo cp 99-icarus-mouse.rules /etc/udev/rules.d/\n\
             sudo udevadm control --reload-rules\n\
             sudo udevadm trigger",
            VID,
            PID_USB,
            VID,
            PID_WIRELESS,
            VID,
            PID_WIRELESS_4G
        );
    }

    /// Sends a command to the mouse hardware and waits for a matching response packet.
    ///
    /// # Errors
    /// Returns `ProtocolError::Hid` if HID communication fails, or `ProtocolError::Timeout`
    /// if no matching response arrives within the timeout period.
    pub fn send_command(&self, cmd: &Command) -> Result<Response> {
        let packet = cmd.to_packet();
        let expected_cmd = cmd.data[1];

        // Drain any stale pending packets to keep the buffer synchronized without blocking
        let mut drain_buf = [0u8; 64];
        while let Ok(n) = self.device.read_timeout(&mut drain_buf, 0) {
            if n == 0 {
                break;
            }
        }

        // Write the command
        self.device
            .write(&packet)
            .map_err(|e| ProtocolError::Hid(e.to_string()))
            .context("Failed to write HID report")?;

        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(READ_TIMEOUT_MS as u64);

        while start.elapsed() < timeout {
            let mut buf = [0u8; 64];
            let remaining_ms = (timeout.saturating_sub(start.elapsed()).as_millis() as i32).max(10);
            match self.device.read_timeout(&mut buf, remaining_ms) {
                Ok(n) if n >= 17 => {
                    if let Ok(resp) = Response::from_packet(&buf[..n])
                        && resp.data[0] == REPORT_ID
                    {
                        let matches = if expected_cmd == CMD_READ {
                            (resp.data[1] == CMD_READ || resp.data[1] == CMD_RESPONSE)
                                && resp.data[3] == cmd.data[3]
                                && resp.data[4] == cmd.data[4]
                        } else {
                            resp.data[1] == expected_cmd
                        };
                        if matches {
                            return Ok(resp);
                        }
                    }
                    // Non-matching packet (e.g. async 0x0A or stale ACK) - continue waiting
                }
                Ok(_) => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => {
                    return Err(ProtocolError::Hid(e.to_string()).into());
                }
            }
        }

        Err(ProtocolError::Timeout.into())
    }

    /// Sends a write command packet and consumes the hardware ACK response.
    ///
    /// # Errors
    /// Returns an error if the write fails or the hardware reports an I/O error.
    pub fn send_write(&self, cmd: &Command) -> Result<()> {
        let packet = cmd.to_packet();
        let expected_cmd = cmd.data[1];

        self.device
            .write(&packet)
            .map_err(|e| ProtocolError::Hid(e.to_string()))
            .context("Failed to write HID report")?;

        // Read ACK packet with short timeout so input buffer remains clean
        let mut ack_buf = [0u8; 64];
        let start = std::time::Instant::now();
        let ack_timeout = Duration::from_millis(60);

        while start.elapsed() < ack_timeout {
            match self.device.read_timeout(&mut ack_buf, 20) {
                Ok(n)
                    if n >= 17
                        && ack_buf[0] == REPORT_ID
                        && (ack_buf[1] == expected_cmd || ack_buf[1] == CMD_WRITE) =>
                {
                    break;
                }
                Ok(_) => {}
                Err(e) => return Err(ProtocolError::Hid(e.to_string()).into()),
            }
        }

        // 8ms pacing delay between writes matching sub_1000c690 in HIDUsb.dll
        std::thread::sleep(Duration::from_millis(8));
        Ok(())
    }

    /// Flashes and commits configuration changes for the given profile (1–5) to hardware memory.
    ///
    /// # Errors
    /// Returns an error if transmitting the apply command fails.
    pub fn commit_config(&self, profile: u8) -> Result<()> {
        self.send_write(&Command::apply_config(profile))
    }

    // ── Read operations ──────────────────────────────────────────────

    /// Informs the mouse firmware whether the PC driver or configuration GUI is active.
    ///
    /// # Errors
    /// Returns an error if transmitting the status packet fails.
    pub fn send_pc_driver_status(&self, active: bool) -> Result<()> {
        self.send_write(&Command::pc_driver_status(active))
    }
}
