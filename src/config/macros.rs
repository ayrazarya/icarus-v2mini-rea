//! Macro recording models, repeat modes, and factory preset sequences.

use serde::{Deserialize, Serialize};

/// Macro event type for the Macro Manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MacroEventType {
    #[default]
    KeyDown,
    KeyUp,
    Delay,
}

/// Single event step within a macro sequence.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MacroEvent {
    pub event_type: MacroEventType,
    pub key: String,
    pub delay_ms: u32,
}

/// Macro repeat behavior mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroRepeatMode {
    /// Repeat N times (ReNumber)
    Times(u32),
    /// Repeat while button held down (ReUntilLoose)
    WhileHeld,
    /// Toggle on click until clicked again (ReUntilClick)
    Toggle,
}

impl Default for MacroRepeatMode {
    fn default() -> Self {
        Self::Times(1)
    }
}

/// Complete macro sequence item.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MacroItem {
    pub name: String,
    pub repeat_mode: MacroRepeatMode,
    pub auto_insert_delay: bool,
    pub default_delay_ms: u32,
    pub events: Vec<MacroEvent>,
}

/// Generates standard factory default gaming macro sequences (Rapid Fire, Triple Tap, Burst Fire).
pub fn default_macros() -> Vec<MacroItem> {
    vec![
        MacroItem {
            name: "Rapid Fire (L-Click)".to_string(),
            repeat_mode: MacroRepeatMode::Times(3),
            auto_insert_delay: true,
            default_delay_ms: 12,
            events: vec![
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 12,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
            ],
        },
        MacroItem {
            name: "Sprint + Jump".to_string(),
            repeat_mode: MacroRepeatMode::Times(1),
            auto_insert_delay: true,
            default_delay_ms: 15,
            events: vec![
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Shift".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 20,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Space".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 20,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Space".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Shift".into(),
                    delay_ms: 0,
                },
            ],
        },
        MacroItem {
            name: "Sniper Quick Scope".to_string(),
            repeat_mode: MacroRepeatMode::Times(1),
            auto_insert_delay: true,
            default_delay_ms: 10,
            events: vec![
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Mouse Right".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 80,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyDown,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::Delay,
                    key: String::new(),
                    delay_ms: 20,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Mouse Left".into(),
                    delay_ms: 0,
                },
                MacroEvent {
                    event_type: MacroEventType::KeyUp,
                    key: "Mouse Right".into(),
                    delay_ms: 0,
                },
            ],
        },
    ]
}
