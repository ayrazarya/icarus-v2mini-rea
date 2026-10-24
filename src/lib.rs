//! # icarus-v2mini
//!
//! Native Linux configuration driver and desktop GUI for the **ICARUS V2 MINI** gaming mouse.
//!
//! Reverse-engineered directly from the official Windows OEM software (`ICARUS V2 MINI.exe`,
//! `HIDUsb.dll`, and `functionset.obj`). Communicates directly with the PixArt PAW3395 sensor
//! and onboard Compx microcontroller via raw 17-byte vendor HID reports on Interface 1.
//!
//! ## Modules
//! - [`config`]: Configuration structs, data models, serialization (TOML/JSON), and safety validators.
//! - [`daemon`]: Background telemetry daemon, auto-profile switching engine, and Waybar/Polybar export.
//! - [`device`]: Low-level HID device discovery, register read/write operations, and firmware flashing.
//! - [`dpi`]: PixArt PAW3395 10-bit linear register encoding, decoding, snapping, and validation.
//! - [`gui`]: Native egui/eframe desktop application with dark/light themes, macro editor, and real-time canvas.
//! - [`i18n`]: Internationalization engine providing dual English and Indonesian language support.
//! - [`protocol`]: Raw 17-byte command packet construction, opcode constants, and checksum algorithms.
//!
//! ## External Dependencies
//! - `hidapi`: Direct cross-platform USB HID communication without requiring root permissions (via udev rules).
//! - `eframe` / `egui`: Immediate-mode GUI framework rendering smoothly via OpenGL / X11 / Wayland.
//! - `serde` / `toml` / `serde_json`: Strongly typed serialization and deserialization of configuration profiles.
//! - `thiserror` / `anyhow`: Robust and idiomatic error handling across protocol and device operations.

pub mod cli;
pub mod config;
pub mod daemon;
pub mod device;
pub mod dpi;
pub mod gui;
pub mod i18n;
pub mod protocol;
