//! Automatic per-application profile switching rules.

use serde::{Deserialize, Serialize};

/// Rule for automatically switching mouse profiles based on active application or game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoProfileRule {
    /// Target application or game name/binary/class pattern (e.g. "cs2", "blender", "dota2")
    pub app_pattern: String,
    /// Profile index to switch to (1 to 5)
    pub profile_index: u8,
    /// Optional human-readable description
    pub description: String,
}

/// Returns standard default per-application auto-profile rules (CS2, Dota 2, Blender).
pub fn default_auto_rules() -> Vec<AutoProfileRule> {
    vec![
        AutoProfileRule {
            app_pattern: "cs2".into(),
            profile_index: 3, // FPS / Esports
            description: "Counter-Strike 2 (Esports 400 DPI)".into(),
        },
        AutoProfileRule {
            app_pattern: "dota2".into(),
            profile_index: 4, // MOBA
            description: "Dota 2 (MOBA Profile)".into(),
        },
        AutoProfileRule {
            app_pattern: "blender".into(),
            profile_index: 5, // Work / Daily
            description: "Blender 3D (Productivity)".into(),
        },
    ]
}
