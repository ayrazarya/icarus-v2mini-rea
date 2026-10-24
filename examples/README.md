# Examples & Hardware Diagnostic Utilities

The `examples/` directory contains sample configuration profiles and low-level hardware diagnostic tools for the **ICARUS V2 MINI** gaming mouse.

---

## Contents

### 1. [`profile-default.toml`](profile-default.toml)
A complete reference TOML configuration profile covering:
- PixArt PAW3395 sensor parameters (`polling_rate`, `lod`, `angle_snapping`, `motion_sync`, `ripple_control`, `debounce_ms`).
- 5 DPI stages with per-stage RGB indicator colors.
- LED lighting effect configuration (`mode`, `brightness`, `speed`, `color`).
- 6 physical mouse button assignments (`LeftClick`, `RightClick`, `MiddleClick`, `Forward`, `Back`, `DpiSwitch`).

**How to apply or save profiles via CLI:**
```bash
# Apply the example profile directly to your mouse
icarus-v2mini apply examples/profile-default.toml

# Save your mouse's current hardware settings to a new TOML file
icarus-v2mini save ~/my-esports-profile.toml
```

---

### 2. [`dpi_probe.rs`](dpi_probe.rs)
A low-level hardware diagnostic probe used to inspect how the mouse and wireless receiver firmware read and write DPI EEPROM registers (`0x000C`..`0x002B`) and to test raw 17-byte HID command/response behavior directly.

**How to run `dpi_probe`:**
```bash
cargo run --example dpi_probe
```

> [!NOTE]
> Make sure your ICARUS V2 MINI mouse or 2.4G/4K wireless dongle is connected and the udev rule (`99-icarus-mouse.rules`) is installed before running `dpi_probe`.
