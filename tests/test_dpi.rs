//! Tests for DPI encoding and decoding (PixArt PAW3395 10-bit linear register mapping).

use icarus_v2mini::dpi::{
    MAX_DPI, MIN_DPI, decode_dpi, decode_dpi_10bit, decode_dpi_xy_10bit, encode_dpi,
    encode_dpi_10bit, encode_dpi_xy_10bit, snap_dpi, validate_dpi,
};

#[test]
fn test_known_driver_defaults() {
    // Expected default DPI grades from vendor Config.ini: 400, 800, 1600, 3200, 6400, 26000
    assert_eq!(encode_dpi(400), (400, 0x07));
    assert_eq!(decode_dpi(0x07), 400);

    assert_eq!(encode_dpi(800), (800, 0x0F));
    assert_eq!(decode_dpi(0x0F), 800);

    // Stage 2: read from physical ICARUS mouse as register 0x1F
    assert_eq!(encode_dpi(1600), (1600, 0x1F));
    assert_eq!(decode_dpi(0x1F), 1600);

    assert_eq!(encode_dpi(3200), (3200, 0x3F));
    assert_eq!(decode_dpi(0x3F), 3200);

    // Stage 5: read from physical ICARUS mouse as register 0x7F
    assert_eq!(encode_dpi(6400), (6400, 0x7F));
    assert_eq!(decode_dpi(0x7F), 6400);
}

#[test]
fn test_user_6500_dpi() {
    // 6500 DPI in Windows driver: (6500 / 50) - 1 = 129 = 0x81
    let (snapped, low, flag) = encode_dpi_10bit(6500);
    assert_eq!(snapped, 6500);
    assert_eq!(low, 0x81);
    assert_eq!(flag, 0x00);
    assert_eq!(decode_dpi_10bit(low, flag), 6500);
}

#[test]
fn test_high_dpi_10bit() {
    // 26,000 DPI: (26000 / 50) - 1 = 519 = 0x207
    let (snapped, low, flag) = encode_dpi_10bit(26000);
    assert_eq!(snapped, 26000);
    assert_eq!(low, 0x07);
    assert_eq!(flag, 0x80); // (2 << 6) = 0x80
    assert_eq!(decode_dpi_10bit(low, flag), 26000);

    // 16,000 DPI: (16000 / 50) - 1 = 319 = 0x13F
    let (snapped, low, flag) = encode_dpi_10bit(16000);
    assert_eq!(snapped, 16000);
    assert_eq!(low, 0x3F);
    assert_eq!(flag, 0x40); // (1 << 6) = 0x40
    assert_eq!(decode_dpi_10bit(low, flag), 16000);
}

#[test]
fn test_roundtrip_all_steps() {
    for dpi in (MIN_DPI..=MAX_DPI).step_by(50) {
        let (snapped, low, flag) = encode_dpi_10bit(dpi);
        assert_eq!(snapped, dpi);
        assert_eq!(decode_dpi_10bit(low, flag), dpi);
    }
}

#[test]
fn test_snap_dpi() {
    assert_eq!(snap_dpi(50), 50);
    assert_eq!(snap_dpi(75), 100);
    assert_eq!(snap_dpi(3990), 4000);
    assert_eq!(snap_dpi(4040), 4050);
    assert_eq!(snap_dpi(4140), 4150);
    assert_eq!(snap_dpi(6500), 6500);
    assert_eq!(snap_dpi(26001), 26000);
}

#[test]
fn test_validate_dpi() {
    assert!(validate_dpi(50).is_ok());
    assert!(validate_dpi(26000).is_ok());
    assert!(validate_dpi(49).is_err());
    assert!(validate_dpi(26001).is_err());
}

#[test]
fn test_independent_xy_dpi_10bit_roundtrip() {
    let pairs = [
        (800, 26000),
        (26000, 800),
        (16000, 26000),
        (26000, 16000),
        (26000, 26000),
        (400, 1600),
        (12850, 50),
        (50, 25950),
    ];

    for (x, y) in pairs {
        let (low_x, low_y, flag) = encode_dpi_xy_10bit(x, y);
        let (dec_x, dec_y) = decode_dpi_xy_10bit(low_x, low_y, flag);
        assert_eq!((dec_x, dec_y), (x, y), "Failed XY roundtrip for ({x}, {y})");
    }

    // Legacy uniform stage packet where only bits 7..6 are populated (low_x == low_y)
    let (_, low, flag) = encode_dpi_10bit(26000);
    assert_eq!(decode_dpi_xy_10bit(low, low, flag), (26000, 26000));
}
