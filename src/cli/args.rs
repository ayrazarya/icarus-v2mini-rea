//! Command line argument parser structures.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "icarus-v2mini",
    about = "Native Linux configuration tool for ICARUS V2 MINI gaming mouse",
    version,
    long_about = "Configure your ICARUS V2 MINI gaming mouse on Linux.\n\
                  Supports DPI, polling rate, LED effects, LOD, angle snapping, \n\
                  motion sync, debounce, and button remapping.\n\n\
                  Run without arguments to launch the Desktop GUI.\n\
                  Hardware: PAW3395 sensor, NRF52840 MCU."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Launch the Desktop Graphical User Interface (GUI)
    Gui,

    /// Show device information and current settings
    Info,

    /// List connected ICARUS V2 MINI devices
    List,

    /// Get or set the polling rate
    Rate {
        /// Set polling rate (125, 250, 500, 1000)
        #[arg(value_name = "HZ")]
        rate: Option<u16>,
    },

    /// Get or set DPI settings
    Dpi {
        #[command(subcommand)]
        subcmd: Option<DpiCommands>,
    },

    /// Configure LED effects
    Led {
        #[command(subcommand)]
        subcmd: LedCommands,
    },

    /// Get or set lift-off distance
    Lod {
        /// Set LOD: "1mm" or "2mm" (or "low"/"high")
        #[arg(value_name = "DISTANCE")]
        distance: Option<String>,
    },

    /// Toggle angle snapping (line straightening)
    #[command(name = "angle-snap")]
    AngleSnap {
        /// "on" or "off"
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Toggle motion sync
    #[command(name = "motion-sync")]
    MotionSync {
        /// "on" or "off"
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Toggle sensor performance mode (performance vs office)
    Performance {
        /// "on"/"perf" for performance mode, "off"/"office" for power-saving office mode
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Toggle ripple control (high DPI smoothing)
    Ripple {
        /// "on" or "off"
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Set debounce time in milliseconds
    Debounce {
        /// Debounce time in ms (0, 2, 4, 6, 8, 10, 12, 16, 20)
        #[arg(value_name = "MS")]
        ms: Option<u8>,
    },

    /// Read battery level (wireless only)
    Battery,

    /// Apply a saved configuration profile (TOML file)
    Apply {
        /// Path to the TOML configuration file
        #[arg(value_name = "FILE")]
        file: String,
    },

    /// Save current device configuration to a TOML file
    Save {
        /// Path to save the configuration
        #[arg(value_name = "FILE")]
        file: String,
    },

    /// Trigger 2.4G wireless dongle pairing mode (Match)
    Pair {
        /// Optional Client ID (CID)
        #[arg(long)]
        cid: Option<u8>,
        /// Optional Manufacturer ID (MID)
        #[arg(long)]
        mid: Option<u8>,
    },

    /// Toggle Long Range wireless mode (RF power boost)
    #[command(name = "long-range")]
    LongRange {
        /// "on" or "off"
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /// Enter USB firmware upgrade mode (ISP/IAP bootloader)
    Dfu,

    /// Generate or install udev rules for non-root access
    #[command(name = "udev-rule")]
    UdevRule {
        /// Install rules directly to /etc/udev/rules.d/99-icarus-mouse.rules using sudo
        #[arg(long)]
        install: bool,
    },

    /// Run background daemon for Waybar/desktop battery integration and auto-profile switching
    Daemon {
        /// Battery polling interval in seconds (default: 10)
        #[arg(long, default_value = "10")]
        interval: u64,

        /// Disable desktop notifications
        #[arg(long)]
        no_notify: bool,

        /// Disable auto-profile switching
        #[arg(long)]
        no_auto_profile: bool,

        /// Install and enable background systemd user service (~/.config/systemd/user/icarus-v2mini.service)
        #[arg(long)]
        install_service: bool,
    },

    /// Configure 4K wireless receiver dongle RGB lighting
    #[command(name = "dongle-rgb")]
    DongleRgb {
        /// Mode: "off", "solid", "breathing", "neon"
        #[arg(long)]
        mode: Option<String>,

        /// Color hex (e.g. "00F0FF")
        #[arg(long)]
        color: Option<String>,

        /// Brightness level (0–10)
        #[arg(long)]
        brightness: Option<u8>,

        /// Breathing / cycle speed (1–10)
        #[arg(long)]
        speed: Option<u8>,

        /// Indicator bar mode (0 = sync with RGB, 1 = battery status, 2 = fixed color)
        #[arg(long)]
        bar_mode: Option<u8>,
    },

    /// Flash firmware (.bin / .hex) image to device via ISP bootloader
    Flash {
        /// Path to firmware binary file (.bin)
        #[arg(value_name = "FILE")]
        file: String,

        /// Flash to 4K wireless receiver dongle MCU instead of mouse MCU
        #[arg(long)]
        dongle: bool,
    },

    /// Manage and flash hardware macro sequences to on-board EEPROM
    Macro {
        #[command(subcommand)]
        subcmd: MacroCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DpiCommands {
    /// Show current DPI settings
    Get,

    /// Set DPI for a specific stage (1-indexed)
    Set {
        /// DPI stage number (1–8)
        #[arg(value_name = "STAGE")]
        stage: u8,

        /// DPI value (50–26000)
        #[arg(value_name = "DPI")]
        value: u16,

        /// Optional Y-axis DPI value (for independent X/Y sensitivity)
        #[arg(long, value_name = "DPI_Y")]
        dpi_y: Option<u16>,
    },

    /// Set the active DPI stage (1-indexed)
    Active {
        /// Stage number (1–8)
        #[arg(value_name = "STAGE")]
        stage: u8,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum MacroCommands {
    /// Flash a macro to an onboard EEPROM slot
    Flash {
        /// Onboard macro slot index (1–16)
        #[arg(value_name = "SLOT")]
        slot: u8,

        /// Macro name
        #[arg(long, default_value = "Macro")]
        name: String,

        /// Sequence of keys or delays (comma-separated, e.g. "Space,20ms,Space")
        #[arg(value_name = "SEQUENCE")]
        sequence: String,
    },

    /// Flash all macros assigned to mouse buttons from a TOML configuration profile
    #[command(name = "flash-all")]
    FlashAll {
        /// Path to TOML configuration profile
        #[arg(value_name = "FILE")]
        file: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum LedCommands {
    /// Set LED mode (off, solid, breathing)
    Mode {
        #[arg(value_name = "MODE")]
        mode: String,
    },

    /// Set LED brightness (0–10)
    Brightness {
        #[arg(value_name = "LEVEL")]
        level: u8,
    },

    /// Set LED color as hex (e.g., "FF0000" for red)
    Color {
        #[arg(value_name = "HEX")]
        hex: String,
    },

    /// Set breathing speed (1–10)
    Speed {
        #[arg(value_name = "SPEED")]
        speed: u8,
    },

    /// Set a DPI stage indicator color
    #[command(name = "stage-color")]
    StageColor {
        /// DPI stage number (1–8)
        #[arg(value_name = "STAGE")]
        stage: u8,

        /// Hex color (e.g., "00FF00")
        #[arg(value_name = "HEX")]
        hex: String,
    },
}
