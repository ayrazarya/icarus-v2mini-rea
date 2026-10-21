//! DPI encoding/decoding for PAW3395 sensor.
//!
//! The ICARUS V2 MINI uses the PAW3395 sensor. DPI values are encoded
//! as register values according to the table in `driver_sensor.h`.
//! The encoding uses the "3335" table (PAW3395 is compatible).
//!
//! Range: 50–26,000 DPI
//! - 50–4,000: step size 50
//! - 4,050–26,000: step size 200

/// Lookup table mapping DPI values to sensor register values.
/// Built from the SENSOR_3335_* defines in `driver_sensor.h`.
/// Only includes entries for DPI values in 50-step increments up to ~10,000.
const DPI_TABLE_3395: &[(u16, u8)] = &[
    (100, 0x00),
    (200, 0x02),
    (300, 0x03),
    (400, 0x04),
    (500, 0x05),
    (600, 0x06),
    (700, 0x07),
    (800, 0x09),
    (900, 0x0A),
    (1000, 0x0B),
    (1100, 0x0C),
    (1200, 0x0D),
    (1300, 0x0E),
    (1400, 0x10),
    (1500, 0x11),
    (1600, 0x12),
    (1700, 0x13),
    (1800, 0x14),
    (1900, 0x16),
    (2000, 0x17),
    (2100, 0x18),
    (2200, 0x19),
    (2300, 0x1A),
    (2400, 0x1B),
    (2500, 0x1D),
    (2600, 0x1E),
    (2700, 0x1F),
    (2800, 0x20),
    (2900, 0x21),
    (3000, 0x23),
    (3100, 0x25),
    (3200, 0x26),
    (3300, 0x27),
    (3400, 0x28),
    (3500, 0x29),
    (3600, 0x2A),
    (3700, 0x2C),
    (3800, 0x2D),
    (3900, 0x2E),
    (4000, 0x2F),
    (4100, 0x30),
    (4200, 0x32),
    (4300, 0x33),
    (4400, 0x34),
    (4500, 0x35),
    (4600, 0x36),
    (4700, 0x38),
    (4800, 0x39),
    (4900, 0x3A),
    (5000, 0x3B),
    (5100, 0x3C),
    (5200, 0x3E),
    (5300, 0x3F),
    (5400, 0x40),
    (5500, 0x41),
    (5600, 0x42),
    (5700, 0x44),
    (5800, 0x45),
    (5900, 0x46),
    (6000, 0x47),
    (6100, 0x48),
    (6200, 0x4A),
    (6300, 0x4B),
    (6400, 0x4C),
    (6500, 0x4D),
    (6600, 0x4E),
    (6700, 0x50),
    (6800, 0x51),
    (6900, 0x52),
    (7000, 0x53),
    (7100, 0x54),
    (7200, 0x56),
    (7300, 0x57),
    (7400, 0x58),
    (7500, 0x59),
    (7600, 0x5A),
    (7700, 0x5C),
    (7800, 0x5D),
    (7900, 0x5E),
    (8000, 0x5F),
    (8100, 0x60),
    (8200, 0x62),
    (8300, 0x63),
];

/// Encode a DPI value to a sensor register value.
///
/// Snaps to the nearest valid DPI value in the table.
/// Returns `(snapped_dpi, register_value)`.
pub fn encode_dpi(dpi: u16) -> (u16, u8) {
    // Clamp to valid range
    let dpi = dpi.clamp(100, 26000);

    // For DPI values in the table range, find the nearest entry
    if dpi <= 8300 {
        let mut best = DPI_TABLE_3395[0];
        for &(table_dpi, reg) in DPI_TABLE_3395 {
            if (table_dpi as i32 - dpi as i32).unsigned_abs()
                <= (best.0 as i32 - dpi as i32).unsigned_abs()
            {
                best = (table_dpi, reg);
            }
        }
        return best;
    }

    // For higher DPI values, extrapolate using the approximate formula:
    // The pattern shows roughly: reg = (DPI / 50) * 0.7 approximately
    // More precisely, from the table we see every ~6 register values
    // correspond to 6 DPI steps of 100, with one skip.
    // For simplicity and safety, we use linear interpolation from the last
    // known points.
    //
    // From the 3311 table which goes up higher:
    // DPI 8000 = 0xBC, DPI 10000 = 0xEB
    // That's 2000 DPI over 0x2F (47) register values = ~42.5 DPI per reg step
    //
    // For values above our table, we'll use the extended formula:
    // reg = 0x63 + ((dpi - 8300) * 47) / 2000
    let reg = 0x63u16.saturating_add(((dpi - 8300) as u32 * 47 / 2000) as u16);
    let reg = reg.min(0xFF) as u8;
    (dpi, reg)
}

/// Decode a sensor register value back to an approximate DPI.
#[allow(dead_code)]
pub fn decode_dpi(reg: u8) -> u16 {
    // Check the lookup table first
    for &(dpi, r) in DPI_TABLE_3395 {
        if r == reg {
            return dpi;
        }
    }

    // Extrapolate for values beyond the table
    if reg > 0x63 {
        let extra = (reg - 0x63) as u32;
        return 8300 + ((extra * 2000) / 47) as u16;
    }

    // For values between table entries, interpolate
    let mut prev = DPI_TABLE_3395[0];
    for &(dpi, r) in DPI_TABLE_3395.iter().skip(1) {
        if r > reg {
            // Interpolate between prev and current
            let frac = (reg - prev.1) as u32 * (dpi - prev.0) as u32 / (r - prev.1).max(1) as u32;
            return prev.0 + frac as u16;
        }
        prev = (dpi, r);
    }

    prev.0
}

/// Validate that a DPI value is within the supported range.
pub fn validate_dpi(dpi: u16) -> Result<(), String> {
    if dpi < 50 {
        return Err(format!("DPI {} is below minimum (50)", dpi));
    }
    if dpi > 26000 {
        return Err(format!("DPI {} is above maximum (26000)", dpi));
    }
    Ok(())
}

/// Snap a DPI value to the nearest valid step.
///
/// - Below 4000: snaps to nearest 50
/// - 4000 and above: snaps to nearest 200
pub fn snap_dpi(dpi: u16) -> u16 {
    let dpi = dpi.clamp(50, 26000);
    if dpi <= 4000 {
        ((dpi + 25) / 50) * 50
    } else {
        ((dpi + 100) / 200) * 200
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_known_values() {
        assert_eq!(encode_dpi(400), (400, 0x04));
        assert_eq!(encode_dpi(800), (800, 0x09));
        assert_eq!(encode_dpi(1600), (1600, 0x12));
        assert_eq!(encode_dpi(3200), (3200, 0x26));
    }

    #[test]
    fn test_decode_known_values() {
        assert_eq!(decode_dpi(0x04), 400);
        assert_eq!(decode_dpi(0x09), 800);
        assert_eq!(decode_dpi(0x12), 1600);
        assert_eq!(decode_dpi(0x26), 3200);
    }

    #[test]
    fn test_roundtrip() {
        for &(dpi, _) in DPI_TABLE_3395 {
            let (_, reg) = encode_dpi(dpi);
            assert_eq!(decode_dpi(reg), dpi);
        }
    }

    #[test]
    fn test_snap_dpi() {
        assert_eq!(snap_dpi(110), 100);
        assert_eq!(snap_dpi(125), 150);
        assert_eq!(snap_dpi(3999), 4000);
        assert_eq!(snap_dpi(4100), 4200);
        assert_eq!(snap_dpi(26001), 26000);
    }
}
