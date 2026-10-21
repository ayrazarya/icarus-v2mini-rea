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
pub fn encode_dpi_10bit(dpi: u16) -> (u16, u8, u8) {
    let snapped = snap_dpi(dpi);
    let reg_10bit = ((snapped / 50).saturating_sub(1)).min(519); // 26000 / 50 - 1 = 519
    let low = (reg_10bit & 0xFF) as u8;
    let flag = (((reg_10bit >> 8) & 0x03) << 6) as u8;
    (snapped, low, flag)
}

/// Backwards-compatible encode returning `(snapped_dpi, low_byte)`.
#[allow(dead_code)]
pub fn encode_dpi(dpi: u16) -> (u16, u8) {
    let (snapped, low, _) = encode_dpi_10bit(dpi);
    (snapped, low)
}

/// Decode a 10-bit PAW3395 register back to DPI value.
///
/// Disassembly from `ICARUS V2 MINI.exe` (0x14000eb21):
///   combined = low | ((flag & 0xC0) << 2)
///   DPI = (combined + 1) * 50
pub fn decode_dpi_10bit(low: u8, flag: u8) -> u16 {
    let high_bits = ((flag as u16) & 0xC0) << 2;
    let reg_10bit = (low as u16) | high_bits;
    ((reg_10bit + 1) * 50).clamp(MIN_DPI, MAX_DPI)
}

/// Decode when only low byte is provided (assumes flag = 0, valid up to 12,800 DPI).
#[allow(dead_code)]
pub fn decode_dpi(low: u8) -> u16 {
    decode_dpi_10bit(low, 0)
}

/// Validate that a DPI value is within the supported PAW3395 range.
pub fn validate_dpi(dpi: u16) -> Result<(), String> {
    if dpi < MIN_DPI {
        return Err(format!("DPI {} is below minimum ({})", dpi, MIN_DPI));
    }
    if dpi > MAX_DPI {
        return Err(format!("DPI {} is above maximum ({})", dpi, MAX_DPI));
    }
    Ok(())
}

/// Snap a DPI value to the nearest valid hardware step (50 DPI increments).
pub fn snap_dpi(dpi: u16) -> u16 {
    let clamped = dpi.clamp(MIN_DPI, MAX_DPI);
    ((clamped + 25) / 50) * 50
}
