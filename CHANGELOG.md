# Changelog

All notable changes to the **ICARUS V2 MINI Linux Driver & Utility** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.0] - 2026-10-24

### Added
- **Modular Domain-Driven Architecture (`src/*`)**:
  - Refactored the entire codebase from monolithic files into 7 cohesive domain modules (`cli`, `config`, `daemon`, `device`, `dpi`, `gui`, `i18n`, `protocol`) with 30+ focused submodules.
- **Full Bilingual Localization Subsystem (`src/i18n/`)**:
  - Dedicated zero-allocation localization engine (`AppLanguage::English` and `AppLanguage::Indonesian`) covering all 5 GUI tabs, status banners, safety lock warnings, and interactive dialogs.
- **Bijective 10-Bit Independent X/Y DPI Encoding (`src/dpi.rs`)**:
  - Added `encode_dpi_xy_10bit` and `decode_dpi_xy_10bit` to pack independent 10-bit X and Y PAW3395 register indices (`0x000`–`0x3FF`) into the 4-byte EEPROM stage packet (`[val_x_low, val_y_low, high_bits_xy, checksum]`), eliminating wrap-around corruption above 12,750 DPI when Separate X/Y DPI is enabled.
- **Cryptographic/Frame Verification on HID Responses (`src/protocol/response.rs`)**:
  - Added `Response::from_bytes_verified`, `verify_checksum`, and `verify_opcode` to validate the OEM `0x55 - sum` frame checksum and command opcodes on incoming 17-byte HID packets.
- **Extended Keyboard & Media Key Coverage (`src/protocol/macro_codec.rs`, `src/config/buttons.rs`)**:
  - Added full support for `F13`–`F24` (`0x68`–`0x73`), `egui::Key` debug aliases (`ArrowUp`, `OpenBracket`, `Semicolon`, etc.), and extended media usage codes (`0x83` Media Player, `0x24` Browser Back, `0x25` Browser Forward).
- **Automatic USB Hot-Plug Detection (`src/gui/app.rs`)**:
  - GUI background poll now automatically detects newly connected ICARUS mice/dongles and refreshes firmware version when a sleeping wireless mouse wakes up.
- **Comprehensive Project Documentation (`docs/`, `CONTRIBUTING.md`, `examples/README.md`)**:
  - Added `docs/ARCHITECTURE.md`, `docs/PROTOCOL.md`, `CONTRIBUTING.md`, and `examples/README.md`.
- **Release Packaging (`.AppImage` & `.tar.gz`)**:
  - Added portable `x86_64.AppImage` bundle alongside the standard `.tar.gz` binary archive in GitHub Releases.

### Fixed
- **Wireless Offline/Sleeping Multi-Second Freeze & False-Positive Success (`src/device/`, `src/cli/`, `src/gui/`)**:
  - Fixed `IcarusDevice::read_config` stalling for **17.5 seconds** (`35 * 500ms` timeouts) and returning a silent `Ok(MouseConfig::default())` when the wireless mouse is asleep.
  - Fixed `IcarusDevice::apply_config` and `flash_macro_slot` stalling for 1.3–2.5 seconds and falsely reporting `Ok(())` when the wireless mouse is offline.
  - Fixed `icarus-v2mini info` stalling for **8.5 seconds** when querying an offline wireless dongle.
- **CLI Standalone Register Persistence (`src/cli/commands/`)**:
  - Added `dev.commit_config(1)?` after standalone CLI writes (`rate`, `lod`, `angle-snap`, `motion-sync`, `ripple`, `debounce`, `sleep`, `dpi`, `led`) so changes take effect immediately on the physical mouse MCU.
- **Hardware `DPI_LED_MODE` Read Address & Atomic Profile Commit (`src/device/`)**:
  - Fixed `read_config` reading `DPI_LED_MODE` from `0x00A1` instead of `addr::DPI_LED_MODE` (`0x00A0`).
  - Updated `commit_config` to write `[profile_id, 0x55 - profile_id]` atomically in a single 2-byte command.
- **Daemon 100% Battery Notification State Machine (`src/daemon/notification.rs`)**:
  - Fixed a state tracking bug where plugging in the mouse at `< 100%` immediately set `last_charging_state = true`, preventing the "Battery Full (100%)" desktop notification from firing when charging reached `100%`.
- **Daemon Auto-Profile Rule Priority & Live Config Reload (`src/daemon/scanner.rs`)**:
  - Fixed rule evaluation so higher-priority rules matching the current profile no longer fall through to lower-priority rules.
  - Added `Sway` `floating_nodes` recursive inspection and immediate rule re-evaluation when `config.toml` modification time (`mtime`) changes.
- **GUI Input Capture, Safety Lock & Macro Recording (`src/gui/`)**:
  - Suspended global shortcuts (`1..5`, `Ctrl+S`, `Ctrl+R`) while binding a button key or recording a live macro.
  - Enforced the Left-Click Safety Lock during interactive `[Press Any Key]` button binding.
  - Prevented off-by-one capacity overflow (> 64 events) and orphan `KeyDown` events during live macro recording.
  - Preserved host-persisted settings (`language`, `macros`, `auto_profile_rules`, `dongle_rgb`, `led_sleep_seconds`) when reading hardware registers.

### Performance & Quality
- **Zero-Allocation Hot Paths**:
  - `hid_usage_to_key_name` now returns `Cow<'static, str>`, eliminating heap allocations for all known keyboard/mouse keys.
  - Replaced `fs::read_to_string` in `/proc/<pid>/comm` scanning with a 64-byte stack-allocated buffer and environment-guarded compositor subprocess execution (`HYPRLAND_INSTANCE_SIGNATURE`, `SWAYSOCK`, `DISPLAY`).
  - Added `AutoProfileCache` backed by `config.toml` `mtime` and atomic `.tmp` + `rename` writes for `~/.cache/icarus-v2mini/battery.json`.
  - Throttled GUI live lighting/dongle slider HID writes to `120ms` (~8 Hz) and 2.4G pairing status polls to `250ms` (4 Hz).
- **Compile-Time Evaluation & Strict Lints**:
  - Enforced `unsafe_code = "forbid"` and strict Clippy lints (`redundant_clone`, `clone_on_copy`, `needless_collect`, `manual_while_let_some`, `or_fun_call`).
  - Converted all pure functions in `src/dpi.rs` and all 38 packet constructors in `src/protocol/command.rs` to `#[must_use] pub const fn`.
  - Expanded automated test suite from 29 to **51 unit & integration tests**.

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
  - Direct hardware opcode `0x06` / `0x09` (`CS_UsbServer_EnterDonglePair`, `CS_UsbServer_ReadDonglePairStatus`) enabling resynchronization between mouse and 2.4G dongle without Windows.
  - Supported in both CLI (`icarus-v2mini pair [--cid] [--mid]`) and GUI Settings tab.
- **Long Range RF Power Boost**:
  - Direct register toggling (`CS_UsbFinder_SetLongRangeMode`, opcode `0x16`) for maximum wireless stability through obstacles.
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
