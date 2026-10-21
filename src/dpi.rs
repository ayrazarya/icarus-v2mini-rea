//! DPI encoding/decoding for PAW3395 sensor.
//!
//! The ICARUS V2 MINI uses the PAW3395 sensor. DPI values are encoded
//! as register values according to the SENSOR_3335 table in `driver_sensor.h`.
//! The encoding uses the "3335" table (PAW3395 is compatible with PAW3335).
//!
//! Range: 100–26,000 DPI
//! - 100–16,000: mapped via the official SENSOR_3335 lookup table (step 100 DPI)
//! - Above 16,000: extrapolated linearly from the table pattern

/// Complete SENSOR_3335 DPI lookup table extracted from `driver_sensor.h`.
/// Maps (DPI, register_value) for every 100-DPI step from 100 to 16000.
const DPI_TABLE_3395: &[(u16, u8)] = &[
    (100,   0x00),
    (200,   0x02),
    (300,   0x03),
    (400,   0x04),
    (500,   0x05),
    (600,   0x06),
    (700,   0x07),
    (800,   0x09),
    (900,   0x0A),
    (1000,  0x0B),
    (1100,  0x0C),
    (1200,  0x0D),
    (1300,  0x0E),
    (1400,  0x10),
    (1500,  0x11),
    (1600,  0x12),
    (1700,  0x13),
    (1800,  0x14),
    (1900,  0x16),
    (2000,  0x17),
    (2100,  0x18),
    (2200,  0x19),
    (2300,  0x1A),
    (2400,  0x1B),
    (2500,  0x1D),
    (2600,  0x1E),
    (2700,  0x1F),
    (2800,  0x20),
    (2900,  0x21),
    (3000,  0x23),
    (3100,  0x25),
    (3200,  0x26),
    (3300,  0x27),
    (3400,  0x28),
    (3500,  0x29),
    (3600,  0x2A),
    (3700,  0x2C),
    (3800,  0x2D),
    (3900,  0x2E),
    (4000,  0x2F),
    (4100,  0x30),
    (4200,  0x32),
    (4300,  0x33),
    (4400,  0x34),
    (4500,  0x35),
    (4600,  0x36),
    (4700,  0x38),
    (4800,  0x39),
    (4900,  0x3A),
    (5000,  0x3B),
    (5100,  0x3C),
    (5200,  0x3E),
    (5300,  0x3F),
    (5400,  0x40),
    (5500,  0x41),
    (5600,  0x42),
    (5700,  0x44),
    (5800,  0x45),
    (5900,  0x46),
    (6000,  0x47),
    (6100,  0x48),
    (6200,  0x4A),
    (6300,  0x4B),
    (6400,  0x4C),
    (6500,  0x4D),
    (6600,  0x4E),
    (6700,  0x50),
    (6800,  0x51),
    (6900,  0x52),
    (7000,  0x53),
    (7100,  0x54),
    (7200,  0x56),
    (7300,  0x57),
    (7400,  0x58),
    (7500,  0x59),
    (7600,  0x5A),
    (7700,  0x5C),
    (7800,  0x5D),
    (7900,  0x5E),
    (8000,  0x5F),
    (8100,  0x60),
    (8200,  0x62),
    (8300,  0x63),
    (8400,  0x64),
    (8500,  0x65),
    (8600,  0x66),
    (8700,  0x68),
    (8800,  0x69),
    (8900,  0x6A),
    (9000,  0x6B),
    (9100,  0x6C),
    (9200,  0x6E),
    (9300,  0x6F),
    (9400,  0x70),
    (9500,  0x71),
    (9600,  0x72),
    (9700,  0x74),
    (9800,  0x75),
    (9900,  0x76),
    (10000, 0x77),
    (10100, 0x78),
    (10200, 0x7A),
    (10300, 0x7B),
    (10400, 0x7C),
    (10500, 0x7D),
    (10600, 0x7E),
    (10700, 0x80),
    (10800, 0x81),
    (10900, 0x82),
    (11000, 0x83),
    (11100, 0x84),
    (11200, 0x86),
    (11300, 0x87),
    (11400, 0x88),
    (11500, 0x89),
    (11600, 0x8A),
    (11700, 0x8C),
    (11800, 0x8D),
    (11900, 0x8E),
    (12000, 0x8F),
    (12100, 0x90),
    (12200, 0x92),
    (12300, 0x93),
    (12400, 0x94),
    (12500, 0x95),
    (12600, 0x96),
    (12700, 0x98),
    (12800, 0x99),
    (12900, 0x9A),
    (13000, 0x9B),
    (13100, 0x9C),
    (13200, 0x9E),
    (13300, 0x9F),
    (13400, 0xA0),
    (13500, 0xA1),
    (13600, 0xA2),
    (13700, 0xA4),
    (13800, 0xA5),
    (13900, 0xA6),
    (14000, 0xA7),
    (14100, 0xA8),
    (14200, 0xAA),
    (14300, 0xAB),
    (14400, 0xAC),
    (14500, 0xAD),
    (14600, 0xAE),
    (14700, 0xB0),
    (14800, 0xB1),
    (14900, 0xB2),
    (15000, 0xB3),
    (15100, 0xB4),
    (15200, 0xB5),
    (15300, 0xB6),
    (15400, 0xB7),
    (15500, 0xB8),
    (15600, 0xB9),
    (15700, 0xBA),
    (15800, 0xBB),
    (15900, 0xBC),
    (16000, 0xBD),
];

/// Encode a DPI value to a sensor register value.
///
/// Snaps to the nearest valid DPI value in the table.
/// Returns `(snapped_dpi, register_value)`.
pub fn encode_dpi(dpi: u16) -> (u16, u8) {
    // Clamp to valid range
    let dpi = dpi.clamp(100, 26000);

    // For DPI values in the table range, find the nearest entry
    if dpi <= 16000 {
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

    // For DPI values above 16000 (table ends at 16000=0xBD),
    // the pattern from 15000-16000 shows ~1 reg step per 100 DPI.
    // Extrapolate linearly: reg = 0xBD + (dpi - 16000) / 100
    let extra = ((dpi - 16000) as u32 + 50) / 100; // round
    let reg = (0xBDu16 + extra as u16).min(0xFF) as u8;
    let snapped = 16000 + (extra as u16) * 100;
    (snapped.min(26000), reg)
}

/// Decode a sensor register value back to a DPI value.
///
/// Uses the complete SENSOR_3335 table for exact matches and interpolates
/// for values between table entries.
pub fn decode_dpi(reg: u8) -> u16 {
    // Check the lookup table first for exact match
    for &(dpi, r) in DPI_TABLE_3395 {
        if r == reg {
            return dpi;
        }
    }

    // For values beyond the table (> 0xBD = 16000 DPI),
    // extrapolate: DPI = 16000 + (reg - 0xBD) * 100
    if reg > 0xBD {
        let extra = (reg - 0xBD) as u16;
        return (16000 + extra * 100).min(26000);
    }

    // For values between table entries, find the surrounding entries and interpolate
    let mut prev = DPI_TABLE_3395[0];
    for &(dpi, r) in DPI_TABLE_3395.iter().skip(1) {
        if r > reg {
            // Linear interpolation between prev and current
            let reg_span = (r - prev.1) as u32;
            let dpi_span = (dpi - prev.0) as u32;
            let offset = (reg - prev.1) as u32;
            if reg_span > 0 {
                let frac = offset * dpi_span / reg_span;
                return prev.0 + frac as u16;
            }
            return prev.0;
        }
        prev = (dpi, r);
    }

    prev.0
}

/// Validate that a DPI value is within the supported range.
pub fn validate_dpi(dpi: u16) -> Result<(), String> {
    if dpi < 100 {
        return Err(format!("DPI {} is below minimum (100)", dpi));
    }
    if dpi > 26000 {
        return Err(format!("DPI {} is above maximum (26000)", dpi));
    }
    Ok(())
}

/// Snap a DPI value to the nearest valid step (100 DPI increments).
///
/// The SENSOR_3335 table uses 100 DPI steps throughout.
pub fn snap_dpi(dpi: u16) -> u16 {
    let dpi = dpi.clamp(100, 26000);
    // Snap to nearest 100
    ((dpi + 50) / 100) * 100
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
    fn test_encode_high_dpi() {
        assert_eq!(encode_dpi(6500), (6500, 0x4D));
        assert_eq!(encode_dpi(9400), (9400, 0x70));
        assert_eq!(encode_dpi(12800), (12800, 0x99));
        assert_eq!(encode_dpi(16000), (16000, 0xBD));
    }

    #[test]
    fn test_decode_known_values() {
        assert_eq!(decode_dpi(0x04), 400);
        assert_eq!(decode_dpi(0x09), 800);
        assert_eq!(decode_dpi(0x12), 1600);
        assert_eq!(decode_dpi(0x26), 3200);
    }

    #[test]
    fn test_decode_high_dpi() {
        assert_eq!(decode_dpi(0x4D), 6500);
        assert_eq!(decode_dpi(0x70), 9400);
        assert_eq!(decode_dpi(0x99), 12800);
        assert_eq!(decode_dpi(0xBD), 16000);
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
        assert_eq!(snap_dpi(150), 200);
        assert_eq!(snap_dpi(149), 100);
        assert_eq!(snap_dpi(3250), 3300);
        assert_eq!(snap_dpi(6500), 6500);
        assert_eq!(snap_dpi(26001), 26000);
    }

    #[test]
    fn test_validate_dpi() {
        assert!(validate_dpi(100).is_ok());
        assert!(validate_dpi(26000).is_ok());
        assert!(validate_dpi(99).is_err());
        assert!(validate_dpi(26001).is_err());
    }
}
