//! DPI encoding and decoding for PixArt PAW3395 sensor.
//!
//! Reverse-engineered directly from `ICARUS V2 MINI.exe` disassembly:
//! (VAs: `0x14000eb21`, `0x14003e54a`, `0x14004595c`).
//!
//! When `Sensor == "3395"` or `"3950"`, the driver bypasses any table lookup
//! and uses the native PixArt PAW3395 10-bit linear step of 50 DPI:
//!
//!   DPI = (register_10bit + 1) * 50
//!   register_10bit = (DPI / 50) - 1
//!
//! Register layout in 4-byte DPI stage payload:
//!   byte 0: X register low byte (bits 0..7)
//!   byte 1: Y register low byte (bits 0..7)
//!   byte 2: flags / high bits: ((register_10bit >> 8) & 0x03) << 6
//!   byte 3: checksum (XOR of bytes 0..2)
//!
//! Supported Range: 50 – 26,000 DPI (step size: 50 DPI below 4000, 200 DPI above 4000)

pub const MIN_DPI: u16 = 50;
pub const MAX_DPI: u16 = 26000;

/// Encode a DPI value to the PAW3395 sensor register value.
///
/// Returns `(snapped_dpi, low_byte, flag_byte)`.
#[must_use]
pub const fn encode_dpi_10bit(dpi: u16) -> (u16, u8, u8) {
    let snapped = snap_dpi(dpi);
    let sub = (snapped / 50).saturating_sub(1);
    let reg_10bit = if sub > 519 { 519 } else { sub }; // 26000 / 50 - 1 = 519
    let low = (reg_10bit & 0xFF) as u8;
    let flag = (((reg_10bit >> 8) & 0x03) << 6) as u8;
    (snapped, low, flag)
}

/// Backwards-compatible encode returning `(snapped_dpi, low_byte)`.
#[must_use]
pub const fn encode_dpi(dpi: u16) -> (u16, u8) {
    let (snapped, low, _) = encode_dpi_10bit(dpi);
    (snapped, low)
}

/// Decode a 10-bit PAW3395 register back to DPI value.
///
/// Disassembly from `ICARUS V2 MINI.exe` (0x14000eb21):
///   combined = low | ((flag & 0xC0) << 2)
///   DPI = (combined + 1) * 50
#[must_use]
pub const fn decode_dpi_10bit(low: u8, flag: u8) -> u16 {
    let high_bits = ((flag as u16) & 0xC0) << 2;
    let reg_10bit = (low as u16) | high_bits;
    let raw = (reg_10bit + 1) * 50;
    if raw < MIN_DPI {
        MIN_DPI
    } else if raw > MAX_DPI {
        MAX_DPI
    } else {
        raw
    }
}

/// Decode when only low byte is provided (assumes flag = 0, valid up to 12,800 DPI).
#[must_use]
pub const fn decode_dpi(low: u8) -> u16 {
    decode_dpi_10bit(low, 0)
}

/// Encode independent X and Y DPI values into `(low_x, low_y, combined_flag)` for 4-byte stage registers.
///
/// Bits 7..6 of `combined_flag` hold the upper 2 bits of the 10-bit X register,
/// bits 5..4 hold the upper 2 bits of the 10-bit Y register, and bit 3 (`0x08`) marks
/// independent X/Y encoding so collisions where `low_x == low_y` (e.g. 12850 vs 50) decode unambiguously.
#[must_use]
pub const fn encode_dpi_xy_10bit(dpi_x: u16, dpi_y: u16) -> (u8, u8, u8) {
    let (snapped_x, low_x, flag_x) = encode_dpi_10bit(dpi_x);
    let (snapped_y, low_y, flag_y) = encode_dpi_10bit(dpi_y);
    let xy_marker = if snapped_x != snapped_y { 0x08 } else { 0x00 };
    let combined_flag = (flag_x & 0xC0) | ((flag_y & 0xC0) >> 2) | xy_marker;
    (low_x, low_y, combined_flag)
}

/// Decode a 4-byte DPI stage register triple `(low_x, low_y, flag)` into `(dpi_x, dpi_y)`.
#[must_use]
pub const fn decode_dpi_xy_10bit(low_x: u8, low_y: u8, flag: u8) -> (u16, u16) {
    let dpi_x = decode_dpi_10bit(low_x, flag & 0xC0);
    let dpi_y = if low_y == low_x && (flag & 0x38) == 0 {
        dpi_x
    } else {
        let flag_y = (flag << 2) & 0xC0;
        decode_dpi_10bit(low_y, flag_y)
    };
    (dpi_x, dpi_y)
}

/// Errors that can occur during DPI validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DpiError {
    /// DPI value is below the hardware minimum limit.
    #[error("DPI {dpi} is below minimum ({min})")]
    BelowMinimum { dpi: u16, min: u16 },
    /// DPI value is above the hardware maximum limit.
    #[error("DPI {dpi} is above maximum ({max})")]
    AboveMaximum { dpi: u16, max: u16 },
}

/// Validate that a DPI value is within the supported PAW3395 range.
pub const fn validate_dpi(dpi: u16) -> Result<(), DpiError> {
    if dpi < MIN_DPI {
        return Err(DpiError::BelowMinimum { dpi, min: MIN_DPI });
    }
    if dpi > MAX_DPI {
        return Err(DpiError::AboveMaximum { dpi, max: MAX_DPI });
    }
    Ok(())
}

/// Snap a DPI value to the nearest valid hardware step (50 DPI increments).
#[must_use]
pub const fn snap_dpi(dpi: u16) -> u16 {
    let clamped = if dpi < MIN_DPI {
        MIN_DPI
    } else if dpi > MAX_DPI {
        MAX_DPI
    } else {
        dpi
    };
    ((clamped + 25) / 50) * 50
}
