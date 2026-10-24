//! Hardware On-Board Macro Buffer Serialization (MacroKeyToBuffer / BufferToMacroKey).

use super::Command;

/// Macro on-board memory layout constants decompiled from `sub_100055b0` and `sub_10005460` in `HIDUsb.dll`.
pub const MACRO_SLOT_BASE_ADDR: u16 = 0x0300;
pub const MACRO_SLOT_SIZE: u16 = 0x0180; // 384 bytes per macro slot
pub const MAX_MACRO_SLOTS: usize = 16;

/// Map a human-readable key string (e.g. from egui or config) to standard USB HID usage code.
#[must_use]
pub fn key_to_hid_usage(key: &str) -> u8 {
    let clean = key.trim();
    if clean.is_empty() {
        return 0x00;
    }

    // Fast O(1) path for single ASCII alphanumeric characters
    if clean.len() == 1 {
        let b = clean.as_bytes()[0];
        if b.is_ascii_alphabetic() {
            return 0x04 + (b.to_ascii_uppercase() - b'A');
        }
        if b.is_ascii_digit() {
            return if b == b'0' { 0x27 } else { 0x1E + (b - b'1') };
        }
    }

    let lower = clean.to_ascii_lowercase();
    if let Some(hex_str) = lower
        .strip_prefix("key_0x")
        .or_else(|| lower.strip_prefix("0x"))
        && let Ok(val) = u8::from_str_radix(hex_str, 16)
    {
        return val;
    }

    if let Some(letter) = lower.strip_prefix("key")
        && letter.len() == 1
        && letter.as_bytes()[0].is_ascii_alphabetic()
    {
        return 0x04 + (letter.as_bytes()[0] - b'a');
    }

    if let Some(digit) = lower.strip_prefix("num")
        && digit.len() == 1
        && digit.as_bytes()[0].is_ascii_digit()
    {
        let d = digit.as_bytes()[0];
        return if d == b'0' { 0x27 } else { 0x1E + (d - b'1') };
    }

    match lower.as_str() {
        "enter" | "return" => 0x28,
        "escape" | "esc" => 0x29,
        "backspace" => 0x2A,
        "tab" => 0x2B,
        "space" => 0x2C,
        "minus" | "-" => 0x2D,
        "equals" | "plus" | "=" | "+" => 0x2E,
        "openbracket" | "[" => 0x2F,
        "closebracket" | "]" => 0x30,
        "backslash" | "pipe" | "\\" | "|" => 0x31,
        "semicolon" | "colon" | ";" | ":" => 0x33,
        "quote" | "'" => 0x34,
        "grave" | "backtick" | "`" => 0x35,
        "comma" | "," => 0x36,
        "period" | "." => 0x37,
        "slash" | "questionmark" | "/" | "?" => 0x38,
        "capslock" => 0x39,
        "f1" => 0x3A,
        "f2" => 0x3B,
        "f3" => 0x3C,
        "f4" => 0x3D,
        "f5" => 0x3E,
        "f6" => 0x3F,
        "f7" => 0x40,
        "f8" => 0x41,
        "f9" => 0x42,
        "f10" => 0x43,
        "f11" => 0x44,
        "f12" => 0x45,
        "f13" => 0x68,
        "f14" => 0x69,
        "f15" => 0x6A,
        "f16" => 0x6B,
        "f17" => 0x6C,
        "f18" => 0x6D,
        "f19" => 0x6E,
        "f20" => 0x6F,
        "f21" => 0x70,
        "f22" => 0x71,
        "f23" => 0x72,
        "f24" => 0x73,
        "printscreen" => 0x46,
        "scrolllock" => 0x47,
        "pause" => 0x48,
        "insert" => 0x49,
        "home" => 0x4A,
        "pageup" => 0x4B,
        "delete" => 0x4C,
        "end" => 0x4D,
        "pagedown" => 0x4E,
        "arrowright" | "right" => 0x4F,
        "arrowleft" | "left" => 0x50,
        "arrowdown" | "down" => 0x51,
        "arrowup" | "up" => 0x52,
        "control" | "ctrl" | "leftcontrol" | "leftctrl" => 0xE0,
        "shift" | "leftshift" => 0xE1,
        "alt" | "leftalt" => 0xE2,
        "command" | "super" | "meta" | "win" => 0xE3,
        "rightcontrol" | "rightctrl" => 0xE4,
        "rightshift" => 0xE5,
        "rightalt" | "altgr" => 0xE6,
        "rightsuper" | "rightmeta" | "rightcommand" => 0xE7,
        "mouse left" | "mouseleft" => 0xF0,
        "mouse right" | "mouseright" => 0xF1,
        "mouse middle" | "mousemiddle" => 0xF2,
        "mouse forward" | "mouseforward" => 0xF3,
        "mouse back" | "mouseback" => 0xF4,
        _ => clean.parse::<u8>().unwrap_or(0x00),
    }
}

/// Convert a USB HID usage code into a readable key name without allocating for known keys.
#[must_use]
pub fn hid_usage_to_key_name(usage: u8) -> std::borrow::Cow<'static, str> {
    use std::borrow::Cow;
    const LETTERS: [&str; 26] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];
    const DIGITS: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
    const F_KEYS: [&str; 12] = [
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
    ];
    const EXT_F_KEYS: [&str; 12] = [
        "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
    ];
    let s = match usage {
        0x04..=0x1D => LETTERS[(usage - 0x04) as usize],
        0x1E..=0x26 => DIGITS[(usage - 0x1E) as usize],
        0x27 => "0",
        0x28 => "Enter",
        0x29 => "Escape",
        0x2A => "Backspace",
        0x2B => "Tab",
        0x2C => "Space",
        0x2D => "-",
        0x2E => "=",
        0x2F => "[",
        0x30 => "]",
        0x31 => "\\",
        0x33 => ";",
        0x34 => "'",
        0x35 => "`",
        0x36 => ",",
        0x37 => ".",
        0x38 => "/",
        0x39 => "CapsLock",
        0x3A..=0x45 => F_KEYS[(usage - 0x3A) as usize],
        0x68..=0x73 => EXT_F_KEYS[(usage - 0x68) as usize],
        0x46 => "PrintScreen",
        0x47 => "ScrollLock",
        0x48 => "Pause",
        0x49 => "Insert",
        0x4A => "Home",
        0x4B => "PageUp",
        0x4C => "Delete",
        0x4D => "End",
        0x4E => "PageDown",
        0x4F => "Right",
        0x50 => "Left",
        0x51 => "Down",
        0x52 => "Up",
        0xE0 => "Ctrl",
        0xE1 => "Shift",
        0xE2 => "Alt",
        0xE3 => "Win",
        0xE4 => "RightCtrl",
        0xE5 => "RightShift",
        0xE6 => "RightAlt",
        0xE7 => "RightSuper",
        0xF0 => "Mouse Left",
        0xF1 => "Mouse Right",
        0xF2 => "Mouse Middle",
        0xF3 => "Mouse Forward",
        0xF4 => "Mouse Back",
        _ => return Cow::Owned(format!("Key_0x{:02X}", usage)),
    };
    Cow::Borrowed(s)
}

/// Serialize a MacroItem into the exact on-board binary layout decompiled from `MacroKeyToBuffer` in `HIDUsb.dll`.
#[must_use]
pub fn macro_to_hardware_buffer(name: &str, events: &[crate::config::MacroEvent]) -> Vec<u8> {
    let event_count = events.len().min(64);
    let mut buf = Vec::with_capacity(32 + event_count * 5 + 1);
    buf.resize(32, 0u8);

    // Find the largest valid UTF-8 character boundary within 30 bytes
    let valid_prefix = if name.len() <= 30 {
        name
    } else {
        match name
            .char_indices()
            .take_while(|(i, c)| i + c.len_utf8() <= 30)
            .last()
        {
            Some((i, c)) => &name[..i + c.len_utf8()],
            None => "",
        }
    };
    let name_bytes = valid_prefix.as_bytes();
    let name_len = name_bytes.len();
    buf[0] = name_len as u8;
    buf[1..1 + name_len].copy_from_slice(name_bytes);

    buf[31] = event_count as u8;

    for event in &events[..event_count] {
        let mut b0 = 0u8;
        match event.event_type {
            crate::config::MacroEventType::KeyDown => b0 |= 0x80,
            crate::config::MacroEventType::KeyUp => b0 |= 0x40,
            crate::config::MacroEventType::Delay => {}
        }
        let usage = if event.event_type == crate::config::MacroEventType::Delay {
            0x00
        } else {
            key_to_hid_usage(&event.key)
        };
        let clamped_delay = event.delay_ms.min(u16::MAX as u32) as u16;
        let delay = clamped_delay.to_le_bytes();
        buf.extend_from_slice(&[b0, 0x00, usage, delay[0], delay[1]]);
    }

    let chk = Command::checksum(&buf);
    buf.push(chk);
    buf
}

/// Deserialize an on-board binary macro buffer into a MacroItem (decompiled from `BufferToMacroKey`).
pub fn hardware_buffer_to_macro(buf: &[u8]) -> Option<crate::config::MacroItem> {
    if buf.len() < 32 {
        return None;
    }
    let name_len = (buf[0] as usize).min(30);
    let name = String::from_utf8_lossy(&buf[1..1 + name_len])
        .trim_matches('\0')
        .to_string();
    let event_count = buf[31] as usize;
    if event_count > 64 {
        return None;
    }
    let payload_end = 32 + event_count * 5;
    if buf.len() < payload_end {
        return None;
    }
    if buf.len() > payload_end {
        let expected_chk = Command::checksum(&buf[..payload_end]);
        if buf[payload_end] != expected_chk {
            return None;
        }
    }

    let mut events = Vec::with_capacity(event_count);
    for i in 0..event_count {
        let offset = 32 + i * 5;
        let b0 = buf[offset];
        let usage = buf[offset + 2];
        let delay_ms = u16::from_le_bytes([buf[offset + 3], buf[offset + 4]]) as u32;

        let event_type = if (b0 & 0x80) != 0 {
            crate::config::MacroEventType::KeyDown
        } else if (b0 & 0x40) != 0 {
            crate::config::MacroEventType::KeyUp
        } else {
            crate::config::MacroEventType::Delay
        };

        events.push(crate::config::MacroEvent {
            event_type,
            key: if event_type == crate::config::MacroEventType::Delay {
                String::new()
            } else {
                hid_usage_to_key_name(usage).into_owned()
            },
            delay_ms,
        });
    }

    Some(crate::config::MacroItem {
        name: if name.is_empty() {
            "Imported_Macro".to_string()
        } else {
            name
        },
        repeat_mode: crate::config::MacroRepeatMode::Times(1),
        auto_insert_delay: true,
        default_delay_ms: 10,
        events,
    })
}
