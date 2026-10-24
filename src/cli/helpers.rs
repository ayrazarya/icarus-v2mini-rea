//! Common CLI parsing helpers.

use crate::config;
use anyhow::Result;

pub fn parse_hex_color(hex: &str) -> Result<config::Color> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 || !hex.is_ascii() {
        anyhow::bail!(
            "Color must be 6 ASCII hex digits (e.g., FF0000), got '{}'",
            hex
        );
    }
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    Ok(config::Color::new(r, g, b))
}

pub fn parse_on_off(s: &str) -> Result<bool> {
    match s.to_lowercase().as_str() {
        "on" | "true" | "1" | "enable" | "enabled" | "yes" | "perf" | "performance" => Ok(true),
        "off" | "false" | "0" | "disable" | "disabled" | "no" | "office" | "eco" => Ok(false),
        _ => anyhow::bail!("Invalid value: '{}'. Use 'on' or 'off'", s),
    }
}
