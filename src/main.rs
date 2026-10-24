//! icarus-v2mini — Native Linux configuration tool for ICARUS V2 MINI gaming mouse
//!
//! Reverse-engineered from the Windows-only ICARUS V2 MINI configuration
//! software. Communicates directly with the mouse over HID to configure
//! DPI, polling rate, LED effects, button mappings, and sensor features.

use anyhow::Result;

fn main() -> Result<()> {
    icarus_v2mini::cli::run()
}
