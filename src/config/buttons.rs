//! Mouse button actions, key remapping, and multimedia assignments.

use crate::i18n::AppLanguage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ButtonAction {
    /// Standard mouse button (button number)
    MouseButton(u8),
    /// Keyboard key (HID usage code, modifier)
    Keyboard { usage: u8, modifier: u8 },
    /// Media key
    Media(u8),
    /// DPI switch action
    DpiSwitch,
    /// Macro playback
    Macro(u8),
    /// Fire key / rapid fire (clicks count, delay in ms)
    FireKey { clicks: u8, delay_ms: u8 },
    /// Disabled
    #[default]
    Disabled,
}

impl ButtonAction {
    /// Encode to the 3-byte wire format.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 3] {
        match self {
            ButtonAction::MouseButton(btn) => [0x01, btn, 0x00],
            ButtonAction::Keyboard { usage, modifier } => [0x02, usage, modifier],
            ButtonAction::Media(code) => [0x03, code, 0x00],
            ButtonAction::DpiSwitch => [0x04, 0x00, 0x00],
            ButtonAction::Macro(id) => [0x05, id, 0x00],
            ButtonAction::FireKey { clicks, delay_ms } => [0x06, clicks, delay_ms],
            ButtonAction::Disabled => [0x00, 0x00, 0x00],
        }
    }

    /// Parse from the 3-byte wire format.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 3]) -> Self {
        match bytes[0] {
            0x01 => ButtonAction::MouseButton(bytes[1]),
            0x02 => ButtonAction::Keyboard {
                usage: bytes[1],
                modifier: bytes[2],
            },
            0x03 => ButtonAction::Media(bytes[1]),
            0x04 => ButtonAction::DpiSwitch,
            0x05 => ButtonAction::Macro(bytes[1]),
            0x06 => ButtonAction::FireKey {
                clicks: bytes[1],
                delay_ms: bytes[2],
            },
            _ => ButtonAction::Disabled,
        }
    }
}

impl std::fmt::Display for ButtonAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ButtonAction::MouseButton(1) => write!(f, "Left Click"),
            ButtonAction::MouseButton(2) => write!(f, "Right Click"),
            ButtonAction::MouseButton(4) => write!(f, "Middle Click"),
            ButtonAction::MouseButton(8) => write!(f, "Back"),
            ButtonAction::MouseButton(16) => write!(f, "Forward"),
            ButtonAction::MouseButton(b) => write!(f, "Mouse Button {}", b),
            ButtonAction::Keyboard { usage, modifier } => {
                if modifier & 0x01 != 0 {
                    f.write_str("Ctrl+")?;
                }
                if modifier & 0x02 != 0 {
                    f.write_str("Shift+")?;
                }
                if modifier & 0x04 != 0 {
                    f.write_str("Alt+")?;
                }
                if modifier & 0x08 != 0 {
                    f.write_str("Win+")?;
                }
                match crate::protocol::hid_usage_to_key_name(*usage) {
                    std::borrow::Cow::Borrowed(name) => {
                        f.write_str(if *usage == 0x29 { "Esc" } else { name })
                    }
                    std::borrow::Cow::Owned(_) => write!(f, "0x{:02X}", usage),
                }
            }
            ButtonAction::Media(code) => {
                let name = match code {
                    0xCD => "Play / Pause",
                    0xE9 => "Volume Up",
                    0xEA => "Volume Down",
                    0xE2 => "Mute",
                    0xB5 => "Next Track",
                    0xB6 => "Prev Track",
                    0xB7 => "Stop",
                    0x83 => "Media Player",
                    0x92 => "Calculator",
                    0x23 => "Web Home",
                    0x24 => "Browser Back",
                    0x25 => "Browser Forward",
                    0x8A => "Mail",
                    0x94 => "My Computer",
                    _ => "Media",
                };
                write!(f, "{}", name)
            }
            ButtonAction::DpiSwitch => write!(f, "DPI Cycle"),
            ButtonAction::Macro(id) => write!(f, "Macro #{}", id),
            ButtonAction::FireKey { clicks, delay_ms } => {
                write!(f, "Fire Key ({}x, {}ms)", clicks, delay_ms)
            }
            ButtonAction::Disabled => write!(f, "Disabled"),
        }
    }
}

impl ButtonAction {
    #[must_use]
    pub fn display_localized(&self, lang: AppLanguage) -> String {
        match lang {
            AppLanguage::English => self.to_string(),
            AppLanguage::Indonesian => match self {
                ButtonAction::MouseButton(1) => "Klik Kiri".to_string(),
                ButtonAction::MouseButton(2) => "Klik Kanan".to_string(),
                ButtonAction::MouseButton(4) => "Klik Tengah".to_string(),
                ButtonAction::MouseButton(8) => "Mundur (Back)".to_string(),
                ButtonAction::MouseButton(16) => "Maju (Forward)".to_string(),
                ButtonAction::MouseButton(b) => format!("Tombol Mouse {}", b),
                ButtonAction::DpiSwitch => "Siklus DPI".to_string(),
                ButtonAction::Macro(id) => format!("Makro #{}", id),
                ButtonAction::FireKey { clicks, delay_ms } => {
                    format!("Klik Cepat ({}x, {}ms)", clicks, delay_ms)
                }
                ButtonAction::Disabled => "Nonaktif".to_string(),
                ButtonAction::Media(code) => {
                    let name = match code {
                        0xCD => "Putar / Jeda",
                        0xE9 => "Volume Naik",
                        0xEA => "Volume Turun",
                        0xE2 => "Bisu (Mute)",
                        0xB5 => "Lagu Selanjutnya",
                        0xB6 => "Lagu Sebelumnya",
                        0xB7 => "Berhenti",
                        0x83 => "Pemutar Media",
                        0x92 => "Kalkulator",
                        0x23 => "Beranda Web",
                        0x24 => "Web Mundur (Back)",
                        0x25 => "Web Maju (Forward)",
                        0x8A => "Email",
                        0x94 => "Komputer Saya",
                        _ => "Media",
                    };
                    name.to_string()
                }
                ButtonAction::Keyboard { .. } => self.to_string(),
            },
        }
    }
}
