// dpi_probe.rs - Quick raw DPI register dump
// Run: cargo run --example dpi_probe
//
// Reads all 8 DPI stage registers and prints raw hex bytes to identify
// encoding mismatch between Windows driver and our decode logic.

use icarus_v2mini::protocol::addr;
use hidapi::HidApi;
use std::time::Duration;

const VID: u16 = 0x3554;
const PIDS: [(u16, &str); 3] = [
    (0xF511, "USB"),
    (0xF512, "2.4G Wireless"),
    (0xF510, "4G Wireless"),
];

fn main() {
    let api = HidApi::new().expect("Failed to init HIDAPI");
    
    let mut device = None;
    for &(pid, name) in &PIDS {
        for dev in api.device_list() {
            if dev.vendor_id() == VID && dev.product_id() == pid && dev.interface_number() == 1 {
                match dev.open_device(&api) {
                    Ok(d) => {
                        d.set_blocking_mode(true).ok();
                        println!("Opened {} (PID=0x{:04X})", name, pid);
                        device = Some(d);
                        break;
                    }
                    Err(e) => eprintln!("Failed to open {}: {}", name, e),
                }
            }
        }
        if device.is_some() { break; }
    }
    
    let dev = device.expect("No ICARUS device found!");
    
    // Send PC Driver Status
    let mut cmd = [0u8; 17];
    cmd[0] = 0x08;
    cmd[1] = 0x02;
    cmd[5] = 0x01;
    cmd[6] = 0x01;
    cmd[16] = checksum(&cmd[..16]);
    let _ = dev.write(&cmd);
    std::thread::sleep(Duration::from_millis(20));
    let mut buf = [0u8; 64];
    let _ = dev.read_timeout(&mut buf, 100);
    
    println!("\n=== RAW DPI STAGE REGISTERS ===");
    for stage in 0..8u8 {
        let addr_lo = addr::DPI_STAGE_BASE + stage * 4;
        let resp = read_register(&dev, 0x00, addr_lo, 4);
        match resp {
            Some(data) => {
                let x_reg = data[0];
                let _y_reg = data[1];
                
                // Try different decode formulas:
                let dpi_3335 = decode_3335(x_reg);
                let dpi_linear_50 = (x_reg as u16 + 1) * 50;
                let dpi_linear_100 = (x_reg as u16) * 100;
                
                println!(
                    "Stage {}: raw=[{:02X} {:02X} {:02X} {:02X}] | x_reg=0x{:02X}({}) | 3335_decode={} | linear_50={} | linear_100={}",
                    stage, data[0], data[1], data[2], data[3],
                    x_reg, x_reg,
                    dpi_3335, dpi_linear_50, dpi_linear_100
                );
            }
            None => println!("Stage {}: READ TIMEOUT", stage),
        }
    }
    
    println!("\n=== RAW DPI COLOR REGISTERS ===");
    for stage in 0..8u8 {
        let addr_lo = addr::DPI_COLOR_BASE + stage * 4;
        let resp = read_register(&dev, 0x00, addr_lo, 4);
        match resp {
            Some(data) => {
                println!(
                    "Stage {} color: R={} G={} B={} chk=0x{:02X}",
                    stage, data[0], data[1], data[2], data[3]
                );
            }
            None => println!("Stage {} color: READ TIMEOUT", stage),
        }
    }
    
    println!("\n=== CURRENT DPI STAGE INDEX ===");
    if let Some(data) = read_register(&dev, 0x00, 0x04, 2) {
        println!("Active stage: {} (raw bytes: {:02X} {:02X})", data[0], data[0], data[1]);
    }
    
    println!("\n=== DPI LED COMPOSITE (0xA0) ===");
    if let Some(data) = read_register(&dev, 0x00, 0xA0, 7) {
        println!(
            "Mode={} Bright={} Speed={} R={} G={} B={} chk=0x{:02X}",
            data[0], data[1], data[2], data[3], data[4], data[5], data[6]
        );
    }
    
    println!("\n=== RAW 0x4C (HIDUsb.dll UsbServer_ReadDPILed) ===");
    if let Some(data) = read_register(&dev, 0x00, 0x4C, 8) {
        println!("0x4C raw bytes: {:02X?}", data);
        if data.len() >= 8 {
            println!("  byte 0 (mode?): 0x{:02X}, chk: 0x{:02X} (valid: {})", data[0], data[1], data[1] == (0x55u8.wrapping_sub(data[0])));
            println!("  byte 2 (bright?): 0x{:02X}, chk: 0x{:02X} (valid: {})", data[2], data[3], data[3] == (0x55u8.wrapping_sub(data[2])));
            println!("  byte 4 (speed?): 0x{:02X}, chk: 0x{:02X} (valid: {})", data[4], data[5], data[5] == (0x55u8.wrapping_sub(data[4])));
            println!("  byte 6 (unknown?): 0x{:02X}, chk: 0x{:02X} (valid: {})", data[6], data[7], data[7] == (0x55u8.wrapping_sub(data[6])));
        }
    }

    println!("\n=== LIVE WRITE TEST: Setting Stage 1 (active) Color to Green & Mode to Solid ===");
    // Stage 1 color register is 0x2C + 1*4 = 0x30
    // Write 3 bytes: R=0, G=255, B=0, chk = 0x55 - (0+255+0)
    write_reg(&dev, 0x00, 0x30, &[0, 255, 0]);
    // Write 0x4C (mode = 1: Solid)
    write_reg(&dev, 0x00, 0x4C, &[0x01]);
    // Write 0x4E (brightness = 10 / 0x80)
    write_reg(&dev, 0x00, 0x4E, &[0x80]);
    // Write 0x50 (speed = 3)
    write_reg(&dev, 0x00, 0x50, &[0x03]);
    // Write 0xA0 composite
    let mut a0 = [0x01, 10, 3, 0, 255, 0, 0];
    let sum: u8 = [0x01, 10, 3, 0, 255, 0].iter().fold(0u8, |a, &b| a.wrapping_add(b));
    a0[6] = 0x55u8.wrapping_sub(sum);
    write_reg_raw(&dev, 0x00, 0xA0, &a0);
    // Commit config (opcode 0x0F, profile 1)
    commit_config(&dev, 1);
    
    std::thread::sleep(Duration::from_millis(100));
    
    // Read back Stage 1 color
    if let Some(data) = read_register(&dev, 0x00, 0x30, 4) {
        println!("Stage 1 readback color: R={} G={} B={} chk=0x{:02X}", data[0], data[1], data[2], data[3]);
    }
    if let Some(data) = read_register(&dev, 0x00, 0x4C, 8) {
        println!("0x4C readback: mode=0x{:02X} bright=0x{:02X} speed=0x{:02X}", data[0], data[2], data[4]);
    }
    
    println!("\n=== POLLING RATE ===");
    if let Some(data) = read_register(&dev, 0x00, 0x00, 2) {
        println!("Report rate code: 0x{:02X} ({})", data[0], data[1]);
    }
    
    println!("\n=== SENSOR_3335 TABLE VERIFICATION ===");
    println!("Some key mappings from driver_sensor.h:");
    let known = [
        (400u16, 0x04u8), (800, 0x09), (1600, 0x12), (3200, 0x26),
        (6500, 0x4D), (9400, 0x70), (12800, 0x99), (16000, 0xBD),
    ];
    for &(dpi, expected_reg) in &known {
        let decoded = decode_3335(expected_reg);
        println!("  {} DPI → reg=0x{:02X} → decoded={} DPI {}", 
            dpi, expected_reg, decoded, 
            if decoded == dpi { "✓" } else { "✗ MISMATCH" }
        );
    }
}

fn read_register(dev: &hidapi::HidDevice, addr_hi: u8, addr_lo: u8, len: u8) -> Option<Vec<u8>> {
    let mut cmd = [0u8; 17];
    cmd[0] = 0x08; // Report ID
    cmd[1] = 0x08; // CMD_READ
    cmd[2] = 0x00;
    cmd[3] = addr_hi;
    cmd[4] = addr_lo;
    cmd[5] = len;
    cmd[16] = checksum(&cmd[..16]);
    
    // Drain
    let mut drain = [0u8; 64];
    while let Ok(n) = dev.read_timeout(&mut drain, 5) {
        if n == 0 { break; }
    }
    
    dev.write(&cmd).ok()?;
    
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(500) {
        let mut buf = [0u8; 64];
        if let Ok(n) = dev.read_timeout(&mut buf, 100) {
            if n >= 17 && buf[0] == 0x08 
                && (buf[1] == 0x08 || buf[1] == 0x09)
                && buf[3] == addr_hi && buf[4] == addr_lo 
            {
                let data_len = (buf[5] as usize).min(10);
                return Some(buf[6..6+data_len].to_vec());
            }
        }
    }
    None
}

fn checksum(bytes: &[u8]) -> u8 {
    let sum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    0x55u8.wrapping_sub(sum)
}

fn decode_3335(reg: u8) -> u16 {
    // Complete SENSOR_3335 table from driver_sensor.h
    let table: &[(u8, u16)] = &[
        (0x00, 100), (0x02, 200), (0x03, 300), (0x04, 400), (0x05, 500),
        (0x06, 600), (0x07, 700), (0x09, 800), (0x0A, 900), (0x0B, 1000),
        (0x0C, 1100), (0x0D, 1200), (0x0E, 1300), (0x10, 1400), (0x11, 1500),
        (0x12, 1600), (0x13, 1700), (0x14, 1800), (0x16, 1900), (0x17, 2000),
        (0x18, 2100), (0x19, 2200), (0x1A, 2300), (0x1B, 2400), (0x1D, 2500),
        (0x1E, 2600), (0x1F, 2700), (0x20, 2800), (0x21, 2900), (0x23, 3000),
        (0x25, 3100), (0x26, 3200), (0x27, 3300), (0x28, 3400), (0x29, 3500),
        (0x2A, 3600), (0x2C, 3700), (0x2D, 3800), (0x2E, 3900), (0x2F, 4000),
        (0x30, 4100), (0x32, 4200), (0x33, 4300), (0x34, 4400), (0x35, 4500),
        (0x36, 4600), (0x38, 4700), (0x39, 4800), (0x3A, 4900), (0x3B, 5000),
        (0x3C, 5100), (0x3E, 5200), (0x3F, 5300), (0x40, 5400), (0x41, 5500),
        (0x42, 5600), (0x44, 5700), (0x45, 5800), (0x46, 5900), (0x47, 6000),
        (0x48, 6100), (0x4A, 6200), (0x4B, 6300), (0x4C, 6400), (0x4D, 6500),
        (0x4E, 6600), (0x50, 6700), (0x51, 6800), (0x52, 6900), (0x53, 7000),
        (0x54, 7100), (0x56, 7200), (0x57, 7300), (0x58, 7400), (0x59, 7500),
        (0x5A, 7600), (0x5C, 7700), (0x5D, 7800), (0x5E, 7900), (0x5F, 8000),
        (0x60, 8100), (0x62, 8200), (0x63, 8300), (0x64, 8400), (0x65, 8500),
        (0x66, 8600), (0x68, 8700), (0x69, 8800), (0x6A, 8900), (0x6B, 9000),
        (0x6C, 9100), (0x6E, 9200), (0x6F, 9300), (0x70, 9400), (0x71, 9500),
        (0x72, 9600), (0x74, 9700), (0x75, 9800), (0x76, 9900), (0x77, 10000),
        (0x78, 10100), (0x7A, 10200), (0x7B, 10300), (0x7C, 10400), (0x7D, 10500),
        (0x7E, 10600), (0x80, 10700), (0x81, 10800), (0x82, 10900), (0x83, 11000),
        (0x84, 11100), (0x86, 11200), (0x87, 11300), (0x88, 11400), (0x89, 11500),
        (0x8A, 11600), (0x8C, 11700), (0x8D, 11800), (0x8E, 11900), (0x8F, 12000),
        (0x90, 12100), (0x92, 12200), (0x93, 12300), (0x94, 12400), (0x95, 12500),
        (0x96, 12600), (0x98, 12700), (0x99, 12800), (0x9A, 12900), (0x9B, 13000),
        (0x9C, 13100), (0x9E, 13200), (0x9F, 13300), (0xA0, 13400), (0xA1, 13500),
        (0xA2, 13600), (0xA4, 13700), (0xA5, 13800), (0xA6, 13900), (0xA7, 14000),
        (0xA8, 14100), (0xAA, 14200), (0xAB, 14300), (0xAC, 14400), (0xAD, 14500),
        (0xAE, 14600), (0xB0, 14700), (0xB1, 14800), (0xB2, 14900), (0xB3, 15000),
        (0xB4, 15100), (0xB5, 15200), (0xB6, 15300), (0xB7, 15400), (0xB8, 15500),
        (0xB9, 15600), (0xBA, 15700), (0xBB, 15800), (0xBC, 15900), (0xBD, 16000),
    ];
    
    for &(r, dpi) in table {
        if r == reg { return dpi; }
    }
    
    // Interpolate for unknown values
    let mut prev = table[0];
    for &(r, dpi) in table.iter().skip(1) {
        if r > reg {
            let frac = (reg - prev.0) as u32 * (dpi - prev.1) as u32 / (r - prev.0).max(1) as u32;
            return prev.1 + frac as u16;
        }
        prev = (r, dpi);
    }
    prev.1
}

fn write_reg(dev: &hidapi::HidDevice, addr_hi: u8, addr_lo: u8, payload: &[u8]) {
    let mut cmd = [0u8; 17];
    cmd[0] = 0x08; // Report ID
    cmd[1] = 0x07; // CMD_WRITE
    cmd[2] = 0x00;
    cmd[3] = addr_hi;
    cmd[4] = addr_lo;
    
    if payload.len() == 1 {
        cmd[5] = 2;
        cmd[6] = payload[0];
        cmd[7] = checksum(&payload);
    } else {
        cmd[5] = (payload.len() + 1) as u8;
        cmd[6..6+payload.len()].copy_from_slice(payload);
        cmd[6+payload.len()] = checksum(payload);
    }
    cmd[16] = checksum(&cmd[..16]);
    
    let _ = dev.write(&cmd);
    std::thread::sleep(Duration::from_millis(15));
}

fn write_reg_raw(dev: &hidapi::HidDevice, addr_hi: u8, addr_lo: u8, payload: &[u8]) {
    let mut cmd = [0u8; 17];
    cmd[0] = 0x08;
    cmd[1] = 0x07;
    cmd[2] = 0x00;
    cmd[3] = addr_hi;
    cmd[4] = addr_lo;
    cmd[5] = payload.len() as u8;
    cmd[6..6+payload.len()].copy_from_slice(payload);
    cmd[16] = checksum(&cmd[..16]);
    let _ = dev.write(&cmd);
    std::thread::sleep(Duration::from_millis(15));
}

fn commit_config(dev: &hidapi::HidDevice, profile: u8) {
    let mut cmd = [0u8; 17];
    cmd[0] = 0x08;
    cmd[1] = 0x0F;
    cmd[5] = 0x01;
    cmd[6] = profile;
    cmd[16] = checksum(&cmd[..16]);
    let _ = dev.write(&cmd);
    std::thread::sleep(Duration::from_millis(30));
}

