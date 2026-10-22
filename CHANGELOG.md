# Changelog

All notable changes to the **ICARUS V2 MINI Linux Driver & Utility** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.0] - 2026-10-22

### Added (Beyond OEM Windows Capabilities)
- **Background Daemon (`icarus-v2mini daemon`)**:
  - Export real-time battery telemetry in Waybar/Polybar-compliant JSON schema to `~/.cache/icarus-v2mini/battery.json`.
  - Desktop notification alerts (`libnotify` / `notify-send`) when battery drops to $\le 15\%$, $10\%$, or $5\%$ (with 5-minute cooldown) and when fully charged to 100%.
  - Multi-tier foreground app detection:
    - Wayland Hyprland (`hyprctl activewindow -j`)
    - Wayland Sway (`swaymsg -t get_tree`)
    - X11 / XWayland (`xprop -root _NET_ACTIVE_WINDOW`)
    - Linux `/proc` process scanner fallback for game detection on desktop environments with restricted window inspection.
  - Automatic on-the-fly hardware profile switching on game launch:
    - `cs2` $\rightarrow$ Profile 3 (*400 DPI, 1000Hz, LOD 1mm*)
    - `dota2` $\rightarrow$ Profile 4 (*1600 DPI, 1000Hz, LOD 1mm*)
    - `blender` $\rightarrow$ Profile 5 (*1600 DPI, 500Hz, LOD 2mm*)
  - One-click systemd user service generator and installer:
    - `icarus-v2mini daemon --install-service` creating `~/.config/systemd/user/icarus-v2mini.service`.
  - GUI Settings tab integration:
    - "Linux Desktop & Battery Integration" card with cache path and systemd installer button.
    - "Auto-Profile Switching (Per-Game)" card with toggle, interactive rule table, rule removal, and app pattern editing.

### Added (Hardware Parity with OEM Driver)
- **2.4G Dongle Re-Pairing Tool**:
  - Direct hardware opcode `0x0B` (`CS_UsbServer_EnterDonglePair`, `CS_UsbServer_ReadDonglePairStatus`) enabling resynchronization between mouse and 2.4G dongle without Windows.
  - Supported in both CLI (`icarus-v2mini pair [--cid] [--mid]`) and GUI Settings tab.
- **Long Range RF Power Boost**:
  - Direct register toggling (`CS_UsbFinder_SetLongRangeMode`, opcode `0x01, 0x11`) for maximum wireless stability through obstacles.
  - Supported in both CLI (`icarus-v2mini long-range [on|off]`) and GUI Settings tab.
- **Hardware On-Board Macro Buffer Conversion**:
  - Complete binary encoder replicating `MacroKeyToBuffer` and `BufferToMacroKey`, converting keystroke timeline sequences into 5-byte event packets for on-board EEPROM buffer serialization.
- **Left Click Safety Lock**:
  - Replicated OEM constraint (`leftbuttononly.obj`) ensuring Button 1 or at least one button is permanently locked to Left Click, preventing accidental lockout.

### Optimized & Quality Improvements
- **Release Compilation Profile**:
  - Full Link-Time Optimization (`lto = "fat"`), `opt-level = 3`, `codegen-units = 1`, `panic = "abort"`, and `strip = true`.
  - Reduced final binary footprint from **22 MB $\rightarrow$ 12 MB** (~45% reduction).
- **Rust Architecture & Coding Standards**:
  - Refactored daemon with type-safe `DaemonConfig` builder pattern.
  - Implemented eager trait derivations (`Debug`, `Clone`, `PartialEq`, `Eq`, `Default`, `Serialize`, `Deserialize`).
  - Strict zero-warning compliance on `cargo clippy --all-targets -- -D warnings` and `cargo fmt`.
  - Added 5 new unit tests in `tests/test_daemon.rs` (bringing total test suite to 29 passing tests).

---

## [0.2.0] - 2026-10-22

### Added
- **Click & Debounce Tester**: Built-in test pad with CPS counter, interval measurement in milliseconds, and hardware switch double-click detector.
- **Single DPI Lock**: 1-click toggle to lock active DPI stage and disable others for competitive FPS tournaments.
- **Custom Milestone Ticks**: Clickable ticks on spectrum slider (`400`, `800`, `1.6k`, `3.2k`, `6.4k`, `12k`, `26k`).
- **Profile Renaming**: Custom onboard profile names integrated into top header dropdown.
- **Hardware Manual Guide Preview**: Embedded hardware layout and quick start guide from the official user manual.

---

## [0.1.0] - 2026-10-21

### Added
- Initial native Linux driver and GUI implementation for Press Play ICARUS V2 MINI mouse.
- Exact PixArt PAW3395 register lookup table (`driver_sensor.h` / `SENSOR_3335`) from 50 to 26,000 DPI.
- Real-time battery status reading (percentage, charging state, millivolt telemetry).
- RGB Lighting control: Solid, Breathing, Neon, Off modes with 14 reverse-engineered ColorBoard swatches.
- 6-Button remapping and macro timeline sequencer.
- Sensor performance tuning: Polling Rate (125–1000Hz), LOD (1mm/2mm), Angle Snapping, Motion Sync, Ripple Control, and Debounce Time.
- Non-root USB access via `99-icarus-mouse.rules`.
